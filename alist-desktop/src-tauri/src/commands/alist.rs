use crate::{
    services::{alist_manager::ServiceInfo, process::hide_std_command_window},
    AppState,
};
use std::process::Command;
use tauri::State;

#[tauri::command]
pub async fn start_alist(state: State<'_, AppState>) -> Result<(), String> {
    let mut manager = state.alist.lock().await;
    manager.start().await
}

#[tauri::command]
pub async fn stop_alist(state: State<'_, AppState>) -> Result<(), String> {
    let mut manager = state.alist.lock().await;
    manager.stop().await
}

#[tauri::command]
pub async fn restart_alist(state: State<'_, AppState>) -> Result<(), String> {
    let mut manager = state.alist.lock().await;
    manager.restart().await
}

#[tauri::command]
pub async fn get_alist_status(state: State<'_, AppState>) -> Result<ServiceInfo, String> {
    let mut manager = state.alist.lock().await;
    manager.get_status().await
}

#[tauri::command]
pub async fn get_alist_password(state: State<'_, AppState>) -> Result<String, String> {
    let manager = state.alist.lock().await;
    manager.get_admin_password().await
}

#[tauri::command]
pub async fn reset_alist_password(state: State<'_, AppState>) -> Result<String, String> {
    let manager = state.alist.lock().await;
    manager.reset_admin_password().await
}

#[tauri::command]
pub async fn set_alist_password(
    state: State<'_, AppState>,
    password: String,
) -> Result<String, String> {
    let manager = state.alist.lock().await;
    manager.set_admin_password(&password).await
}

#[tauri::command]
pub async fn open_alist_web(state: State<'_, AppState>) -> Result<(), String> {
    let mut manager = state.alist.lock().await;
    let info = manager.get_status().await?;

    if info.status != "running" {
        return Err("AList is not running".to_string());
    }

    open_url(&info.web_url)
}

fn open_url(url: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let mut command = Command::new("cmd");
        command.args(["/C", "start", "", url]);
        hide_std_command_window(&mut command);
        command
            .spawn()
            .map_err(|err| format!("open browser failed: {err}"))?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|err| format!("open browser failed: {err}"))?;
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|err| format!("open browser failed: {err}"))?;
    }

    Ok(())
}
