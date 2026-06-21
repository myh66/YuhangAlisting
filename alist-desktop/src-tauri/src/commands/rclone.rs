use crate::{
    services::rclone_manager::{MountConfig, MountInfo},
    AppState,
};
use tauri::State;

#[tauri::command]
pub async fn list_mounts(state: State<'_, AppState>) -> Result<Vec<MountInfo>, String> {
    let mut rclone = state.rclone.lock().await;
    rclone.list().await
}

#[tauri::command]
pub async fn create_mount(
    state: State<'_, AppState>,
    config: MountConfig,
) -> Result<MountInfo, String> {
    let mut rclone = state.rclone.lock().await;
    rclone.create(config)
}

#[tauri::command]
pub async fn update_mount(
    state: State<'_, AppState>,
    config: MountConfig,
) -> Result<MountInfo, String> {
    let mut rclone = state.rclone.lock().await;
    rclone.update(config).await
}

#[tauri::command]
pub async fn delete_mount(
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<MountInfo>, String> {
    let mut rclone = state.rclone.lock().await;
    rclone.delete(&id).await
}

#[tauri::command]
pub async fn mount_rclone(
    state: State<'_, AppState>,
    id: String,
    password: Option<String>,
) -> Result<Vec<MountInfo>, String> {
    let password = match password {
        Some(password) if !password.trim().is_empty() => password,
        _ => {
            let alist = state.alist.lock().await;
            alist.get_admin_password().await?
        }
    };

    let mut rclone = state.rclone.lock().await;
    rclone.mount(&id, &password).await
}

#[tauri::command]
pub async fn unmount_rclone(
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<MountInfo>, String> {
    let mut rclone = state.rclone.lock().await;
    rclone.unmount(&id).await
}

#[tauri::command]
pub async fn unmount_all_rclone(state: State<'_, AppState>) -> Result<Vec<MountInfo>, String> {
    let mut rclone = state.rclone.lock().await;
    rclone.unmount_all().await
}

#[tauri::command]
pub async fn open_mount_path(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let rclone = state.rclone.lock().await;
    rclone.open_mount(&id)
}

#[tauri::command]
pub async fn mount_auto_rclone(
    state: State<'_, AppState>,
    password: Option<String>,
) -> Result<Vec<MountInfo>, String> {
    let password = match password {
        Some(password) if !password.trim().is_empty() => password,
        _ => {
            let alist = state.alist.lock().await;
            alist.get_admin_password().await?
        }
    };

    let mut rclone = state.rclone.lock().await;
    rclone.mount_auto_configs(&password).await
}
