use crate::{config::AppConfig, services::logs::emit_log, AppState};
use serde::{Deserialize, Serialize};
#[cfg(any(target_os = "macos", all(unix, not(target_os = "macos"))))]
use std::{fs, path::PathBuf};
use tauri::State;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheckResult {
    pub app_version: String,
    pub alist: ReleaseInfo,
    pub rclone: ReleaseInfo,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseInfo {
    pub name: String,
    pub tag_name: String,
    pub html_url: String,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    name: Option<String>,
    tag_name: String,
    html_url: String,
}

#[tauri::command]
pub async fn get_app_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    Ok(state.config.lock().await.get())
}

#[tauri::command]
pub async fn save_app_config(
    state: State<'_, AppState>,
    config: AppConfig,
) -> Result<AppConfig, String> {
    let saved = {
        let mut store = state.config.lock().await;
        store.save(config)?
    };

    state.alist.lock().await.apply_config(&saved);
    state.rclone.lock().await.apply_config(&saved);

    Ok(saved)
}

#[tauri::command]
pub async fn set_autostart(state: State<'_, AppState>, enabled: bool) -> Result<AppConfig, String> {
    set_autostart_enabled(enabled)?;

    let saved = {
        let store = state.config.lock().await;
        store.get()
    };

    emit_log(
        &state.app,
        &state.logs,
        "system",
        "info",
        format!("Autostart {}", if enabled { "enabled" } else { "disabled" }),
    )
    .await;

    Ok(saved)
}

#[tauri::command]
pub async fn get_autostart_enabled() -> Result<bool, String> {
    autostart_enabled()
}

#[tauri::command]
pub async fn check_updates() -> Result<UpdateCheckResult, String> {
    let client = reqwest::Client::new();
    let alist = fetch_latest_release(&client, "AlistGo/alist").await?;
    let rclone = fetch_latest_release(&client, "rclone/rclone").await?;

    Ok(UpdateCheckResult {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        alist,
        rclone,
    })
}

async fn fetch_latest_release(client: &reqwest::Client, repo: &str) -> Result<ReleaseInfo, String> {
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let release = client
        .get(url)
        .header("User-Agent", "alist-desktop")
        .send()
        .await
        .map_err(|err| format!("request latest release failed: {err}"))?
        .error_for_status()
        .map_err(|err| format!("latest release response failed: {err}"))?
        .json::<GithubRelease>()
        .await
        .map_err(|err| format!("parse latest release failed: {err}"))?;

    Ok(ReleaseInfo {
        name: release.name.unwrap_or_else(|| release.tag_name.clone()),
        tag_name: release.tag_name,
        html_url: release.html_url,
    })
}

fn set_autostart_enabled(enabled: bool) -> Result<(), String> {
    let exe =
        std::env::current_exe().map_err(|err| format!("resolve current exe failed: {err}"))?;

    #[cfg(target_os = "windows")]
    {
        let key = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
        let value = "AListDesktop";

        if enabled {
            let data = format!("\"{}\"", exe.display());
            run_command(
                "reg",
                &["add", key, "/v", value, "/t", "REG_SZ", "/d", &data, "/f"],
            )
        } else {
            let _ = run_command("reg", &["delete", key, "/v", value, "/f"]);
            Ok(())
        }
    }

    #[cfg(target_os = "macos")]
    {
        let plist = launch_agent_path()?;

        if enabled {
            if let Some(parent) = plist.parent() {
                fs::create_dir_all(parent)
                    .map_err(|err| format!("create launch agent dir failed: {err}"))?;
            }

            let content = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
"http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>com.alist.desktop</string>
  <key>ProgramArguments</key>
  <array><string>{}</string></array>
  <key>RunAtLoad</key><true/>
</dict>
</plist>
"#,
                exe.display()
            );
            fs::write(plist, content).map_err(|err| format!("write launch agent failed: {err}"))
        } else {
            let _ = fs::remove_file(plist);
            Ok(())
        }
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let desktop = autostart_desktop_path()?;

        if enabled {
            if let Some(parent) = desktop.parent() {
                fs::create_dir_all(parent)
                    .map_err(|err| format!("create autostart dir failed: {err}"))?;
            }

            let content = format!(
                "[Desktop Entry]\nType=Application\nName=AList Desktop\nExec={}\nX-GNOME-Autostart-enabled=true\n",
                exe.display()
            );
            fs::write(desktop, content).map_err(|err| format!("write autostart failed: {err}"))
        } else {
            let _ = fs::remove_file(desktop);
            Ok(())
        }
    }
}

fn autostart_enabled() -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        let key = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
        let output = Command::new("reg")
            .args(["query", key, "/v", "AListDesktop"])
            .output()
            .map_err(|err| format!("query autostart failed: {err}"))?;
        return Ok(output.status.success());
    }

    #[cfg(target_os = "macos")]
    {
        return Ok(launch_agent_path()?.exists());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        return Ok(autostart_desktop_path()?.exists());
    }
}

#[cfg(target_os = "windows")]
fn run_command(program: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|err| format!("run {program} failed: {err}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("{program} exited with status {status}"))
    }
}

#[cfg(target_os = "macos")]
fn launch_agent_path() -> Result<PathBuf, String> {
    home_dir()
        .map(|home| home.join("Library/LaunchAgents/com.alist.desktop.plist"))
        .ok_or_else(|| "home dir not found".to_string())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn autostart_desktop_path() -> Result<PathBuf, String> {
    home_dir()
        .map(|home| home.join(".config/autostart/alist-desktop.desktop"))
        .ok_or_else(|| "home dir not found".to_string())
}

#[cfg(any(target_os = "macos", all(unix, not(target_os = "macos"))))]
fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}
