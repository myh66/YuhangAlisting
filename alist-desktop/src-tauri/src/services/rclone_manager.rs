use crate::{
    config::{app_data_dir, AppConfig},
    services::{
        alist_manager::resolve_rclone_binary_path,
        logs::{emit_log, SharedLogBuffer},
        process::{hide_std_command_window, hide_tokio_command_window},
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    process::Stdio,
};
use tauri::AppHandle;
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, BufReader},
    process::{Child, Command},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MountConfig {
    pub id: String,
    pub name: String,
    pub remote_path: String,
    pub local_path: String,
    pub auto_mount: bool,
    pub cache_mode: CacheMode,
    pub buffer_size: String,
    pub vfs_cache_max_age: String,
    pub read_only: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CacheMode {
    Off,
    Minimal,
    Writes,
    Full,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MountInfo {
    pub id: String,
    pub name: String,
    pub remote_path: String,
    pub local_path: String,
    pub auto_mount: bool,
    pub cache_mode: CacheMode,
    pub buffer_size: String,
    pub vfs_cache_max_age: String,
    pub read_only: bool,
    pub status: MountStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MountStatus {
    Mounted,
    Mounting,
    Unmounted,
    Error,
}

pub struct RcloneManager {
    app: AppHandle,
    logs: SharedLogBuffer,
    configs: Vec<MountConfig>,
    mounts: HashMap<String, MountInstance>,
    rclone_path: PathBuf,
    mounts_path: PathBuf,
    recent_targets_path: PathBuf,
    recent_targets: Vec<String>,
    alist_port: u16,
}

struct MountInstance {
    process: Option<Child>,
    status: MountStatus,
    error: Option<String>,
}

impl RcloneManager {
    pub fn new(app: AppHandle, logs: SharedLogBuffer, config: &AppConfig) -> Self {
        let mounts_path = app_data_dir(&app).join("mounts.json");
        let recent_targets_path = app_data_dir(&app).join("recent_mount_targets.json");
        let configs = read_mounts(&mounts_path);
        let recent_targets = read_json_vec(&recent_targets_path);
        let rclone_path = resolve_rclone_binary_path(&app, config);

        Self {
            app,
            logs,
            configs,
            mounts: HashMap::new(),
            rclone_path,
            mounts_path,
            recent_targets_path,
            recent_targets,
            alist_port: config.alist_port,
        }
    }

    pub fn apply_config(&mut self, config: &AppConfig) {
        self.alist_port = config.alist_port;

        if self.mounts.values().all(|mount| mount.process.is_none()) {
            self.rclone_path = resolve_rclone_binary_path(&self.app, config);
        }
    }

    pub async fn list(&mut self) -> Result<Vec<MountInfo>, String> {
        self.refresh_mounts().await?;
        Ok(self.to_mount_infos())
    }

    pub fn create(&mut self, mut config: MountConfig) -> Result<MountInfo, String> {
        normalize_mount_config(&mut config);
        validate_mount_config(&config)?;

        if config.id.trim().is_empty() {
            config.id = new_mount_id();
        }

        if self.configs.iter().any(|item| item.id == config.id) {
            return Err(format!("mount id already exists: {}", config.id));
        }

        self.remember_mount_target(&config.local_path)?;
        self.configs.push(config.clone());
        self.flush()?;
        Ok(self.to_mount_info(&config))
    }

    pub async fn update(&mut self, mut config: MountConfig) -> Result<MountInfo, String> {
        normalize_mount_config(&mut config);
        validate_mount_config(&config)?;

        let Some(existing_index) = self.configs.iter().position(|item| item.id == config.id) else {
            return Err(format!("mount not found: {}", config.id));
        };

        let existing = self.configs[existing_index].clone();
        let needs_remount = existing.name != config.name
            || existing.remote_path != config.remote_path
            || existing.local_path != config.local_path
            || existing.cache_mode.as_rclone_arg() != config.cache_mode.as_rclone_arg()
            || existing.buffer_size != config.buffer_size
            || existing.vfs_cache_max_age != config.vfs_cache_max_age
            || existing.read_only != config.read_only;
        let was_mounted = self
            .mounts
            .get(&config.id)
            .is_some_and(|instance| matches!(instance.status, MountStatus::Mounted | MountStatus::Mounting));

        if was_mounted && needs_remount {
            let id = config.id.clone();
            self.unmount(&id).await?;
        }

        self.configs[existing_index] = config.clone();
        self.remember_mount_target(&config.local_path)?;
        self.flush()?;
        Ok(self.to_mount_info(&config))
    }

    pub async fn delete(&mut self, id: &str) -> Result<Vec<MountInfo>, String> {
        self.unmount(id).await?;
        let initial_len = self.configs.len();
        self.configs.retain(|config| config.id != id);

        if self.configs.len() == initial_len {
            return Err(format!("mount not found: {id}"));
        }

        self.flush()?;
        self.list().await
    }

    pub async fn mount(&mut self, id: &str, password: &str) -> Result<Vec<MountInfo>, String> {
        self.refresh_mounts().await?;

        let config = self
            .configs
            .iter()
            .find(|config| config.id == id)
            .cloned()
            .ok_or_else(|| format!("mount not found: {id}"))?;

        self.ensure_rclone_binary_exists()?;
        validate_mount_target(&config.local_path)?;
        self.remember_mount_target(&config.local_path)?;
        self.release_stale_mount_target(&config).await;

        if let Some(instance) = self.mounts.get(id) {
            if matches!(
                instance.status,
                MountStatus::Mounted | MountStatus::Mounting
            ) && instance.process.is_some()
            {
                return Ok(self.to_mount_infos());
            }
        }

        prepare_mount_target(&config.local_path)?;
        emit_log(
            &self.app,
            &self.logs,
            "rclone",
            "info",
            format!("Mounting {} to {}", config.remote_path, config.local_path),
        )
        .await;

        let obscured_password = self.obscure_password(password).await?;
        let mount_target = rclone_mount_target(&config.local_path);
        let mut command = Command::new(&self.rclone_path);
        command
            .arg("mount")
            .arg(format!(
                ":webdav:{}",
                normalized_remote_path(&config.remote_path)
            ))
            .arg(&mount_target)
            .arg("--webdav-url")
            .arg(format!("http://127.0.0.1:{}/dav", self.alist_port))
            .arg("--webdav-user")
            .arg("admin")
            .arg("--webdav-pass")
            .arg(obscured_password)
            .arg("--vfs-cache-mode")
            .arg(config.cache_mode.as_rclone_arg())
            .arg("--buffer-size")
            .arg(&config.buffer_size)
            .arg("--vfs-cache-max-age")
            .arg(&config.vfs_cache_max_age)
            .arg("--dir-cache-time")
            .arg("5m")
            .arg("--links")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if config.read_only {
            command.arg("--read-only");
        }

        #[cfg(target_os = "windows")]
        {
            if config.local_path.len() == 2 && config.local_path.ends_with(':') {
                command.arg("--network-mode");
                command.arg("--volname");
                command.arg(windows_volume_name(&config));
            }
        }

        hide_tokio_command_window(&mut command);

        let mut child = command
            .spawn()
            .map_err(|err| format!("start rclone mount failed: {err}"))?;

        if let Some(stdout) = child.stdout.take() {
            spawn_log_reader(
                self.app.clone(),
                self.logs.clone(),
                "rclone",
                "info",
                stdout,
            );
        }

        if let Some(stderr) = child.stderr.take() {
            spawn_log_reader(
                self.app.clone(),
                self.logs.clone(),
                "rclone",
                "error",
                stderr,
            );
        }

        self.mounts.insert(
            id.to_string(),
            MountInstance {
                process: Some(child),
                status: MountStatus::Mounting,
                error: None,
            },
        );

        tokio::time::sleep(std::time::Duration::from_millis(700)).await;
        self.refresh_mounts().await?;

        if let Some(instance) = self.mounts.get(id) {
            if matches!(instance.status, MountStatus::Error | MountStatus::Unmounted) {
                let detail = instance
                    .error
                    .clone()
                    .unwrap_or_else(|| "rclone exited before the mount became ready".to_string());
                self.cleanup_failed_mount_target(&config).await;
                return Err(format!(
                    "{detail}. Check that the AList path exists and the local drive letter is free."
                ));
            }
        }

        Ok(self.to_mount_infos())
    }

    pub async fn unmount(&mut self, id: &str) -> Result<Vec<MountInfo>, String> {
        let config = self.configs.iter().find(|config| config.id == id).cloned();

        self.unmount_without_target_cleanup(id).await?;

        if let Some(config) = config {
            self.cleanup_mount_target(&config).await;
        }

        self.list().await
    }

    async fn unmount_without_target_cleanup(&mut self, id: &str) -> Result<(), String> {
        if let Some(mut instance) = self.mounts.remove(id) {
            if let Some(mut child) = instance.process.take() {
                child
                    .kill()
                    .await
                    .map_err(|err| format!("stop rclone mount failed: {err}"))?;
                let _ = child.wait().await;
            }

            emit_log(
                &self.app,
                &self.logs,
                "rclone",
                "info",
                format!("Unmounted {id}"),
            )
            .await;
        }

        Ok(())
    }

    pub async fn unmount_all(&mut self) -> Result<Vec<MountInfo>, String> {
        let ids: Vec<String> = self.mounts.keys().cloned().collect();

        for id in ids {
            let _ = self.unmount_without_target_cleanup(&id).await;
        }

        self.cleanup_all_mount_targets().await;
        self.list().await
    }

    pub fn open_mount(&self, id: &str) -> Result<(), String> {
        let config = self
            .configs
            .iter()
            .find(|config| config.id == id)
            .ok_or_else(|| format!("mount not found: {id}"))?;

        open_path(&config.local_path)
    }

    pub async fn mount_auto_configs(&mut self, password: &str) -> Result<Vec<MountInfo>, String> {
        let ids: Vec<String> = self
            .configs
            .iter()
            .filter(|config| config.auto_mount)
            .map(|config| config.id.clone())
            .collect();

        for id in ids {
            let _ = self.mount(&id, password).await;
        }

        self.list().await
    }

    async fn refresh_mounts(&mut self) -> Result<(), String> {
        for (id, instance) in self.mounts.iter_mut() {
            let Some(child) = instance.process.as_mut() else {
                continue;
            };

            match child
                .try_wait()
                .map_err(|err| format!("check rclone process failed: {err}"))?
            {
                Some(status) if status.success() => {
                    instance.status = MountStatus::Unmounted;
                    instance.process = None;
                    instance.error = None;
                }
                Some(status) => {
                    instance.status = MountStatus::Error;
                    instance.process = None;
                    instance.error = Some(format!("rclone exited with code {:?}", status.code()));
                    emit_log(
                        &self.app,
                        &self.logs,
                        "rclone",
                        "error",
                        format!("Mount {id} exited with code {:?}", status.code()),
                    )
                    .await;
                }
                None => {
                    instance.status = MountStatus::Mounted;
                }
            }
        }

        Ok(())
    }

    fn to_mount_infos(&self) -> Vec<MountInfo> {
        self.configs
            .iter()
            .map(|config| self.to_mount_info(config))
            .collect()
    }

    fn to_mount_info(&self, config: &MountConfig) -> MountInfo {
        let status = self
            .mounts
            .get(&config.id)
            .map(|instance| instance.status.clone())
            .unwrap_or(MountStatus::Unmounted);
        let error = self
            .mounts
            .get(&config.id)
            .and_then(|instance| instance.error.clone());

        MountInfo {
            id: config.id.clone(),
            name: config.name.clone(),
            remote_path: config.remote_path.clone(),
            local_path: config.local_path.clone(),
            auto_mount: config.auto_mount,
            cache_mode: config.cache_mode.clone(),
            buffer_size: config.buffer_size.clone(),
            vfs_cache_max_age: config.vfs_cache_max_age.clone(),
            read_only: config.read_only,
            status,
            error,
        }
    }

    fn flush(&self) -> Result<(), String> {
        if let Some(parent) = self.mounts_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|err| format!("create mounts dir failed: {err}"))?;
        }

        let json = serde_json::to_string_pretty(&self.configs)
            .map_err(|err| format!("serialize mounts failed: {err}"))?;
        std::fs::write(&self.mounts_path, json).map_err(|err| format!("write mounts failed: {err}"))
    }

    fn remember_mount_target(&mut self, local_path: &str) -> Result<(), String> {
        let target = normalize_local_target(local_path);

        if target.is_empty() {
            return Ok(());
        }

        self.recent_targets.retain(|item| item != &target);
        self.recent_targets.insert(0, target);
        self.recent_targets.truncate(12);
        self.flush_recent_targets()
    }

    fn flush_recent_targets(&self) -> Result<(), String> {
        if let Some(parent) = self.recent_targets_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|err| format!("create mount target history dir failed: {err}"))?;
        }

        let json = serde_json::to_string_pretty(&self.recent_targets)
            .map_err(|err| format!("serialize mount target history failed: {err}"))?;
        std::fs::write(&self.recent_targets_path, json)
            .map_err(|err| format!("write mount target history failed: {err}"))
    }

    fn ensure_rclone_binary_exists(&self) -> Result<(), String> {
        if self.rclone_path.exists() {
            Ok(())
        } else {
            Err(format!(
                "Rclone binary not found: {}. Run yarn prebuild first.",
                self.rclone_path.display()
            ))
        }
    }

    async fn obscure_password(&self, password: &str) -> Result<String, String> {
        let mut command = Command::new(&self.rclone_path);
        command.arg("obscure").arg(password);
        hide_tokio_command_window(&mut command);

        let output = command
            .output()
            .await
            .map_err(|err| format!("rclone obscure failed: {err}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("rclone obscure failed: {}", stderr.trim()));
        }

        let obscured = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if obscured.is_empty() {
            Err("rclone obscure returned empty output".to_string())
        } else {
            Ok(obscured)
        }
    }

    async fn release_stale_mount_target(&self, config: &MountConfig) {
        #[cfg(target_os = "windows")]
        {
            if !is_windows_drive_letter(&config.local_path) {
                return;
            }

            let _ = run_std_command("net", &["use", &config.local_path, "/delete", "/y"]);
            self.kill_stale_rclone_processes().await;
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = config;
        }
    }

    async fn cleanup_failed_mount_target(&mut self, config: &MountConfig) {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let _ = self.unmount(&config.id).await;
        self.cleanup_mount_target(config).await;
        self.kill_stale_rclone_processes().await;

        #[cfg(target_os = "windows")]
        {
            let targets = self.cleanup_targets(Some(config));
            if targets.iter().any(|target| windows_drive_exists(target)) {
                cleanup_windows_targets(&targets, true);
            }
        }
    }

    async fn cleanup_mount_target(&self, config: &MountConfig) {
        #[cfg(target_os = "windows")]
        {
            cleanup_windows_targets(&self.cleanup_targets(Some(config)), false);
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = config;
        }
    }

    async fn cleanup_all_mount_targets(&self) {
        #[cfg(target_os = "windows")]
        {
            cleanup_windows_targets(&self.cleanup_targets(None), false);
        }

        #[cfg(not(target_os = "windows"))]
        {}
    }

    pub async fn cleanup_stale_processes(&self) {
        #[cfg(target_os = "windows")]
        {
            cleanup_windows_targets(&self.cleanup_targets(None), false);
        }

        self.kill_stale_rclone_processes().await;
    }

    async fn kill_stale_rclone_processes(&self) {
        #[cfg(target_os = "windows")]
        {
            kill_processes_by_exe_path(&self.rclone_path);
        }

        #[cfg(not(target_os = "windows"))]
        {}
    }

    #[cfg(target_os = "windows")]
    fn cleanup_targets(&self, config: Option<&MountConfig>) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut targets = Vec::new();

        let mut push_target = |local_path: &str| {
            let target = normalize_local_target(local_path);

            if is_windows_drive_letter(&target) && seen.insert(target.clone()) {
                targets.push(target);
            }
        };

        if let Some(config) = config {
            push_target(&config.local_path);
        }

        for config in &self.configs {
            push_target(&config.local_path);
        }

        for target in &self.recent_targets {
            push_target(target);
        }

        targets
    }
}

impl CacheMode {
    fn as_rclone_arg(&self) -> &'static str {
        match self {
            CacheMode::Off => "off",
            CacheMode::Minimal => "minimal",
            CacheMode::Writes => "writes",
            CacheMode::Full => "full",
        }
    }
}

fn read_mounts(path: &PathBuf) -> Vec<MountConfig> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

fn read_json_vec(path: &PathBuf) -> Vec<String> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str(&content).ok())
        .unwrap_or_default()
}

fn normalize_mount_config(config: &mut MountConfig) {
    config.name = config.name.trim().to_string();
    config.remote_path = normalized_remote_path(&config.remote_path);
    config.local_path = config.local_path.trim().to_string();

    if config.buffer_size.trim().is_empty() {
        config.buffer_size = "256M".to_string();
    }

    if config.vfs_cache_max_age.trim().is_empty() {
        config.vfs_cache_max_age = "1h".to_string();
    }
}

fn normalize_local_target(local_path: &str) -> String {
    let trimmed = local_path.trim();

    #[cfg(target_os = "windows")]
    {
        if is_windows_drive_letter(trimmed) {
            return format!("{}:", trimmed.chars().next().unwrap().to_ascii_uppercase());
        }
    }

    trimmed.to_string()
}

fn validate_mount_config(config: &MountConfig) -> Result<(), String> {
    if config.name.trim().is_empty() {
        return Err("mount name is required".to_string());
    }

    if config.remote_path.trim().is_empty() {
        return Err("remote path is required".to_string());
    }

    validate_mount_target(&config.local_path)
}

fn validate_mount_target(local_path: &str) -> Result<(), String> {
    let local_path = local_path.trim();

    if local_path.is_empty() {
        return Err("local path is required".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        let is_drive = local_path.len() == 2 && local_path.ends_with(':');
        let is_absolute = std::path::Path::new(local_path).is_absolute();

        if !is_drive && !is_absolute {
            return Err("Windows mount target must be a drive letter or absolute path".to_string());
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if !std::path::Path::new(local_path).is_absolute() {
            return Err("Unix mount target must be an absolute directory".to_string());
        }
    }

    Ok(())
}

fn prepare_mount_target(local_path: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if is_windows_drive_letter(local_path) {
            return Ok(());
        }
    }

    std::fs::create_dir_all(local_path).map_err(|err| format!("create mount target failed: {err}"))
}

fn rclone_mount_target(local_path: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        if is_windows_drive_letter(local_path) {
            return format!("{}\\", local_path.trim());
        }
    }

    local_path.to_string()
}

fn normalized_remote_path(remote_path: &str) -> String {
    let trimmed = remote_path.trim();

    let normalized = if trimmed.starts_with('/') {
        trimmed.to_string()
    } else {
        format!("/{trimmed}")
    };

    normalized.to_lowercase()
}

fn new_mount_id() -> String {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();

    format!("mount-{millis}")
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

fn open_path(path: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let mut command = std::process::Command::new("explorer");
        command.arg(path);
        hide_std_command_window(&mut command);
        command
            .spawn()
            .map_err(|err| format!("open path failed: {err}"))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|err| format!("open path failed: {err}"))?;
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|err| format!("open path failed: {err}"))?;
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn sanitize_volume_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .map(|ch| match ch {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            _ => ch,
        })
        .collect();

    if sanitized.trim().is_empty() {
        "Mount".to_string()
    } else {
        sanitized
    }
}

#[cfg(target_os = "windows")]
fn is_windows_drive_letter(path: &str) -> bool {
    let path = path.trim();
    path.len() == 2 && path.ends_with(':') && path.as_bytes()[0].is_ascii_alphabetic()
}

#[cfg(target_os = "windows")]
fn windows_volume_name(config: &MountConfig) -> String {
    format!("AList-{}", sanitize_volume_name(&config.name))
}

#[cfg(target_os = "windows")]
fn run_std_command(program: &str, args: &[&str]) -> Result<(), String> {
    let mut command = std::process::Command::new(program);
    command.args(args);
    hide_std_command_window(&mut command);

    let status = command
        .status()
        .map_err(|err| format!("run {program} failed: {err}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("{program} exited with status {status}"))
    }
}

#[cfg(target_os = "windows")]
fn kill_processes_by_exe_path(exe_path: &PathBuf) {
    let escaped_path = exe_path.to_string_lossy().replace('\'', "''");
    let script = format!(
        "$target = '{}'; Get-CimInstance Win32_Process -Filter \"name = 'rclone.exe'\" | Where-Object {{ $_.ExecutablePath -eq $target }} | ForEach-Object {{ Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }}",
        escaped_path
    );

    let _ = run_std_command(
        "powershell.exe",
        &[
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ],
    );
}

#[cfg(target_os = "windows")]
fn cleanup_windows_targets(targets: &[String], refresh_explorer: bool) {
    for target in targets {
        let _ = run_std_command("net", &["use", target, "/delete", "/y"]);
        let drive_root = format!("{target}\\");
        let _ = run_std_command("net", &["use", &drive_root, "/delete", "/y"]);
    }

    if refresh_explorer {
        restart_explorer();
    }
}

#[cfg(target_os = "windows")]
fn windows_drive_exists(target: &str) -> bool {
    std::path::Path::new(&format!("{target}\\")).exists()
}

#[cfg(target_os = "windows")]
pub fn restart_explorer() {
    let _ = run_std_command("taskkill", &["/F", "/IM", "explorer.exe"]);
    let mut command = std::process::Command::new("explorer.exe");
    hide_std_command_window(&mut command);
    let _ = command.spawn();
}

#[cfg(not(target_os = "windows"))]
pub fn restart_explorer() {}
