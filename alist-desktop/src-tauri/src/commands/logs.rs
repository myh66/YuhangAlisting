use crate::{services::logs::LogEntry, AppState};
use tauri::State;

#[tauri::command]
pub async fn list_logs(state: State<'_, AppState>) -> Result<Vec<LogEntry>, String> {
    Ok(state.logs.lock().await.list())
}

#[tauri::command]
pub async fn clear_logs(state: State<'_, AppState>) -> Result<(), String> {
    state.logs.lock().await.clear();
    Ok(())
}
