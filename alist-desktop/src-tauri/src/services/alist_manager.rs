use crate::{
    config::{app_data_dir, fallback_project_root, AppConfig},
    services::logs::{emit_log, SharedLogBuffer},
};
use serde::Serialize;
use std::{
    fs,
    path::PathBuf,
    process::Stdio,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, BufReader},
    process::{Child, Command},
};

const MAX_RESTART_ATTEMPTS: u8 = 3;
const DEFAULT_ADMIN_PASSWORD: &str = "root";

#[derive(Debug, Clone)]
pub enum ServiceStatus {
    Stopped,
    Starting,
    Running,
    Error(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceInfo {
    pub status: String,
    pub port: u16,
    pub uptime_seconds: u64,
    pub web_url: String,
    pub data_dir: String,
    pub binary_path: String,
    pub error: Option<String>,
    pub restart_attempts: u8,
}

pub struct AListManager {
    app: AppHandle,
    logs: SharedLogBuffer,
    process: Option<Child>,
    port: u16,
    data_dir: PathBuf,
    binary_path: PathBuf,
    status: ServiceStatus,
    start_time: Option<Instant>,
    restart_attempts: u8,
    http_client: reqwest::Client,
}

impl AListManager {
    pub fn new(app: AppHandle, logs: SharedLogBuffer, config: &AppConfig) -> Self {
        let data_dir = app_data_dir(&app).join("alist-data");
        let binary_path = resolve_alist_binary_path(&app, config);

        Self {
            app,
            logs,
            process: None,
            port: config.alist_port,
            data_dir,
            binary_path,
            status: ServiceStatus::Stopped,
            start_time: None,
            restart_attempts: 0,
            http_client: reqwest::Client::new(),
        }
    }

    pub fn apply_config(&mut self, config: &AppConfig) {
        self.port = config.alist_port;

        if self.process.is_none() {
            self.binary_path = resolve_alist_binary_path(&self.app, config);
        }
    }

    pub async fn start(&mut self) -> Result<(), String> {
        self.spawn_alist(true).await
    }

    pub async fn stop(&mut self) -> Result<(), String> {
        if let Some(mut child) = self.process.take() {
            emit_log(&self.app, &self.logs, "alist", "info", "Stopping AList").await;
            child
                .kill()
                .await
                .map_err(|err| format!("stop alist process failed: {err}"))?;
            let _ = child.wait().await;
        }

        self.status = ServiceStatus::Stopped;
        self.start_time = None;
        self.restart_attempts = 0;
        Ok(())
    }

    pub async fn restart(&mut self) -> Result<(), String> {
        self.stop().await?;
        self.spawn_alist(true).await
    }

    pub async fn health_check(&self) -> bool {
        let url = format!("http://127.0.0.1:{}/ping", self.port);

        self.http_client
            .get(url)
            .timeout(Duration::from_secs(2))
            .send()
            .await
            .map(|response| response.status().is_success())
            .unwrap_or(false)
    }

    pub async fn get_admin_password(&self) -> Result<String, String> {
        self.run_admin_command(&["admin"]).await
    }

    pub async fn reset_admin_password(&self) -> Result<String, String> {
        self.run_admin_command(&["admin", "random"]).await
    }

    pub async fn set_admin_password(&self, password: &str) -> Result<String, String> {
        self.run_admin_command(&["admin", "set", password]).await
    }

    pub async fn get_status(&mut self) -> Result<ServiceInfo, String> {
        self.refresh_process_state().await?;

        if matches!(
            self.status,
            ServiceStatus::Starting | ServiceStatus::Running
        ) {
            if self.health_check().await {
                self.status = ServiceStatus::Running;
            }
        }

        Ok(ServiceInfo {
            status: self.status_kind().to_string(),
            port: self.port,
            uptime_seconds: self.get_uptime().as_secs(),
            web_url: self.web_url(),
            data_dir: self.data_dir.to_string_lossy().to_string(),
            binary_path: self.binary_path.to_string_lossy().to_string(),
            error: self.status_error(),
            restart_attempts: self.restart_attempts,
        })
    }

    pub fn get_uptime(&self) -> Duration {
        self.start_time
            .map(|start| start.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0))
    }

    pub fn web_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    async fn spawn_alist(&mut self, reset_restart_attempts: bool) -> Result<(), String> {
        if self.process.is_some() {
            return Ok(());
        }

        self.ensure_binary_exists()?;
        std::fs::create_dir_all(&self.data_dir)
            .map_err(|err| format!("create alist data dir failed: {err}"))?;
        self.prepare_config().await?;

        if reset_restart_attempts {
            self.restart_attempts = 0;
        }

        self.status = ServiceStatus::Starting;
        emit_log(
            &self.app,
            &self.logs,
            "alist",
            "info",
            format!("Starting AList on port {}", self.port),
        )
        .await;

        let mut child = Command::new(&self.binary_path)
            .arg("server")
            .arg("--data")
            .arg(&self.data_dir)
            .arg("--log-std")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| format!("start alist process failed: {err}"))?;

        if let Some(stdout) = child.stdout.take() {
            spawn_log_reader(self.app.clone(), self.logs.clone(), "alist", "info", stdout);
        }

        if let Some(stderr) = child.stderr.take() {
            spawn_log_reader(
                self.app.clone(),
                self.logs.clone(),
                "alist",
                "error",
                stderr,
            );
        }

        self.process = Some(child);
        self.start_time = Some(Instant::now());
        Ok(())
    }

    async fn refresh_process_state(&mut self) -> Result<(), String> {
        let Some(child) = self.process.as_mut() else {
            return Ok(());
        };

        let Some(exit_status) = child
            .try_wait()
            .map_err(|err| format!("check alist process failed: {err}"))?
        else {
            return Ok(());
        };

        self.process = None;
        self.start_time = None;

        if exit_status.success() {
            self.status = ServiceStatus::Stopped;
            self.restart_attempts = 0;
            emit_log(&self.app, &self.logs, "alist", "info", "AList stopped").await;
            return Ok(());
        }

        emit_log(
            &self.app,
            &self.logs,
            "alist",
            "error",
            format!("AList crashed with code {:?}", exit_status.code()),
        )
        .await;

        if self.restart_attempts < MAX_RESTART_ATTEMPTS {
            self.restart_attempts += 1;
            emit_log(
                &self.app,
                &self.logs,
                "alist",
                "warn",
                format!(
                    "Restarting AList ({}/{MAX_RESTART_ATTEMPTS})",
                    self.restart_attempts
                ),
            )
            .await;
            self.spawn_alist(false).await?;
            return Ok(());
        }

        self.status = ServiceStatus::Error(format!(
            "AList crashed too many times, last code: {:?}",
            exit_status.code()
        ));
        Ok(())
    }

    async fn run_admin_command(&self, args: &[&str]) -> Result<String, String> {
        self.ensure_binary_exists()?;

        let output = Command::new(&self.binary_path)
            .args(args)
            .arg("--data")
            .arg(&self.data_dir)
            .output()
            .await
            .map_err(|err| format!("run alist admin command failed: {err}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("alist admin command failed: {}", stderr.trim()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined_output = format!("{stdout}\n{stderr}");
        let value = parse_password_from_admin_output(&combined_output);

        if let Some(value) = value {
            Ok(value)
        } else if args.len() == 3 && args[0] == "admin" && args[1] == "set" {
            Ok(args[2].to_string())
        } else if args == &["admin"] {
            Err("AList cannot reveal the current admin password after first startup. Reset it instead.".to_string())
        } else {
            Err("alist admin command did not return a password".to_string())
        }
    }

    async fn prepare_config(&self) -> Result<(), String> {
        let config_path = self.data_dir.join("config.json");
        let is_first_bootstrap = !config_path.exists();

        if is_first_bootstrap {
            self.bootstrap_default_config(&config_path).await?;
            self.set_default_admin_password().await?;
        }

        if !config_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&config_path)
            .map_err(|err| format!("read alist config failed: {err}"))?;
        let mut value = serde_json::from_str::<serde_json::Value>(&content)
            .map_err(|err| format!("parse alist config failed: {err}"))?;

        *value
            .pointer_mut("/scheme/http_port")
            .ok_or_else(|| "alist config missing scheme.http_port".to_string())? =
            serde_json::Value::from(self.port);

        if let Some(address) = value.pointer_mut("/scheme/address") {
            *address = serde_json::Value::from("0.0.0.0");
        }

        let json = serde_json::to_string_pretty(&value)
            .map_err(|err| format!("serialize alist config failed: {err}"))?;
        fs::write(config_path, json).map_err(|err| format!("write alist config failed: {err}"))
    }

    async fn set_default_admin_password(&self) -> Result<(), String> {
        self.set_admin_password(DEFAULT_ADMIN_PASSWORD).await?;
        emit_log(
            &self.app,
            &self.logs,
            "alist",
            "info",
            "Default AList admin login initialized: admin/root",
        )
        .await;
        Ok(())
    }

    async fn bootstrap_default_config(&self, config_path: &PathBuf) -> Result<(), String> {
        emit_log(
            &self.app,
            &self.logs,
            "alist",
            "info",
            "Generating default AList config",
        )
        .await;

        let mut child = Command::new(&self.binary_path)
            .arg("server")
            .arg("--data")
            .arg(&self.data_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|err| format!("bootstrap alist config failed: {err}"))?;

        for _ in 0..50 {
            if config_path.exists() {
                let _ = child.kill().await;
                let _ = child.wait().await;
                return Ok(());
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        let _ = child.kill().await;
        let _ = child.wait().await;
        Err("timed out while generating AList config".to_string())
    }

    fn ensure_binary_exists(&self) -> Result<(), String> {
        if self.binary_path.exists() {
            Ok(())
        } else {
            Err(format!(
                "AList binary not found: {}. Run yarn prebuild first.",
                self.binary_path.display()
            ))
        }
    }

    fn status_kind(&self) -> &'static str {
        match self.status {
            ServiceStatus::Stopped => "stopped",
            ServiceStatus::Starting => "starting",
            ServiceStatus::Running => "running",
            ServiceStatus::Error(_) => "error",
        }
    }

    fn status_error(&self) -> Option<String> {
        match &self.status {
            ServiceStatus::Error(message) => Some(message.clone()),
            _ => None,
        }
    }
}

fn spawn_log_reader<R>(
    app: AppHandle,
    logs: SharedLogBuffer,
    source: &'static str,
    level: &'static str,
    reader: R,
) where
    R: AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut lines = BufReader::new(reader).lines();

        while let Ok(Some(line)) = lines.next_line().await {
            emit_log(&app, &logs, source, level, line).await;
        }
    });
}

fn parse_password_from_admin_output(output: &str) -> Option<String> {
    output.lines().find_map(|line| {
        let sanitized = strip_ansi_codes(line);
        let (_, password) = sanitized.split_once("password:")?;
        let password = password.trim().trim_matches('"');

        if password.is_empty() {
            None
        } else {
            Some(password.to_string())
        }
    })
}

fn strip_ansi_codes(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            while let Some(next) = chars.next() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            output.push(ch);
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::parse_password_from_admin_output;

    #[test]
    fn parses_password_from_alist_logrus_output() {
        let output = r#"time="2026-06-21 11:37:46" level=info msg="password: q84N5MN2""#;

        assert_eq!(
            parse_password_from_admin_output(output),
            Some("q84N5MN2".to_string())
        );
    }
}

pub fn resolve_alist_binary_path(app: &AppHandle, config: &AppConfig) -> PathBuf {
    if let Some(path) = config
        .alist_binary_path
        .as_ref()
        .filter(|path| !path.is_empty())
    {
        return PathBuf::from(path);
    }

    let file_name = binary_name("alist");

    if let Some(resource_binary) = bundled_binary_path(app, &file_name) {
        return resource_binary;
    }

    fallback_project_root().join("binaries").join(file_name)
}

pub fn resolve_rclone_binary_path(app: &AppHandle, config: &AppConfig) -> PathBuf {
    if let Some(path) = config
        .rclone_binary_path
        .as_ref()
        .filter(|path| !path.is_empty())
    {
        return PathBuf::from(path);
    }

    let file_name = binary_name("rclone");

    if let Some(resource_binary) = bundled_binary_path(app, &file_name) {
        return resource_binary;
    }

    fallback_project_root().join("binaries").join(file_name)
}

pub fn bundled_binary_path(app: &AppHandle, file_name: &str) -> Option<PathBuf> {
    let resource_dir = app.path().resource_dir().ok()?;

    [
        resource_dir.join("_up_").join("binaries").join(file_name),
        resource_dir.join("binaries").join(file_name),
        resource_dir.join(file_name),
    ]
    .into_iter()
    .find(|candidate| candidate.exists())
}

fn binary_name(base: &str) -> String {
    if cfg!(windows) {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}
