use crate::{
    services::alist_manager::{
        bundled_binary_path, resolve_alist_binary_path, resolve_rclone_binary_path,
    },
    AppState,
};
use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::State;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
    pub family: String,
    pub default_mount_hint: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeReadiness {
    pub platform: PlatformInfo,
    pub alist_binary_path: String,
    pub alist_binary_ready: bool,
    pub rclone_binary_path: String,
    pub rclone_binary_ready: bool,
    pub winfsp_required: bool,
    pub winfsp_installed: bool,
    pub winfsp_installer_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WinFspStatus {
    pub required: bool,
    pub installed: bool,
    pub installer_path: Option<String>,
    pub download_url: String,
}

#[tauri::command]
pub async fn get_platform_info() -> Result<PlatformInfo, String> {
    Ok(platform_info())
}

#[tauri::command]
pub async fn get_runtime_readiness(state: State<'_, AppState>) -> Result<RuntimeReadiness, String> {
    let config = state.config.lock().await.get();
    let alist_path = resolve_alist_binary_path(&state.app, &config);
    let rclone_path = resolve_rclone_binary_path(&state.app, &config);

    Ok(RuntimeReadiness {
        platform: platform_info(),
        alist_binary_ready: alist_path.exists(),
        alist_binary_path: alist_path.to_string_lossy().to_string(),
        rclone_binary_ready: rclone_path.exists(),
        rclone_binary_path: rclone_path.to_string_lossy().to_string(),
        winfsp_required: cfg!(target_os = "windows"),
        winfsp_installed: winfsp_installed(),
        winfsp_installer_path: bundled_winfsp_installer(&state.app)
            .map(|path| path.to_string_lossy().to_string()),
    })
}

#[tauri::command]
pub async fn get_winfsp_status(state: State<'_, AppState>) -> Result<WinFspStatus, String> {
    Ok(WinFspStatus {
        required: cfg!(target_os = "windows"),
        installed: winfsp_installed(),
        installer_path: bundled_winfsp_installer(&state.app)
            .map(|path| path.to_string_lossy().to_string()),
        download_url: "https://github.com/winfsp/winfsp/releases/latest".to_string(),
    })
}

#[tauri::command]
pub async fn install_winfsp(state: State<'_, AppState>) -> Result<(), String> {
    if !cfg!(target_os = "windows") {
        return Ok(());
    }

    if winfsp_installed() {
        return Ok(());
    }

    let installer = bundled_winfsp_installer(&state.app)
        .ok_or_else(|| "WinFsp installer not bundled. Run yarn prebuild --force before building.".to_string())?;

    install_winfsp_msi(&installer)
}

fn platform_info() -> PlatformInfo {
    PlatformInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        family: std::env::consts::FAMILY.to_string(),
        default_mount_hint: default_mount_hint().to_string(),
    }
}

fn default_mount_hint() -> &'static str {
    if cfg!(target_os = "windows") {
        "Z:"
    } else if cfg!(target_os = "macos") {
        "/Volumes/AList"
    } else {
        "/mnt/alist"
    }
}

fn winfsp_installed() -> bool {
    if !cfg!(target_os = "windows") {
        return true;
    }

    [
        r"C:\Program Files (x86)\WinFsp\bin\winfsp-x64.dll",
        r"C:\Program Files\WinFsp\bin\winfsp-x64.dll",
        r"C:\Program Files (x86)\WinFsp\bin\winfsp-x86.dll",
        r"C:\Program Files\WinFsp\bin\winfsp-x86.dll",
    ]
    .iter()
    .any(|path| std::path::Path::new(path).exists())
}

fn bundled_winfsp_installer(app: &tauri::AppHandle) -> Option<PathBuf> {
    let file_name = "winfsp.msi";

    if let Some(installer) = bundled_binary_path(app, file_name) {
        return Some(installer);
    }

    let fallback = std::env::current_dir()
        .ok()
        .and_then(|dir| {
            for ancestor in dir.ancestors() {
                let candidate = ancestor.join("binaries").join(file_name);
                if candidate.exists() {
                    return Some(candidate);
                }
            }
            None
        });

    fallback
}

#[cfg(target_os = "windows")]
fn install_winfsp_msi(installer: &Path) -> Result<(), String> {
    let installer_arg = installer.to_string_lossy().replace('\'', "''");
    let command = format!(
        "Start-Process msiexec.exe -ArgumentList '/i','{}','/passive','/norestart' -Verb RunAs -Wait",
        installer_arg
    );

    std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &command])
        .spawn()
        .map_err(|err| format!("launch WinFsp installer failed: {err}"))?;

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn install_winfsp_msi(_installer: &Path) -> Result<(), String> {
    Ok(())
}
