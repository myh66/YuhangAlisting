mod commands;
mod config;
mod services;
mod tray;

use config::{CloseAction, ConfigStore};
use services::{
    alist_manager::AListManager,
    logs::{shared_log_buffer, SharedLogBuffer},
    rclone_manager::RcloneManager,
};
use tauri::{Emitter, Manager};
use tokio::{sync::Mutex, time::Duration};

pub struct AppState {
    pub app: tauri::AppHandle,
    pub config: Mutex<ConfigStore>,
    pub logs: SharedLogBuffer,
    pub alist: Mutex<AListManager>,
    pub rclone: Mutex<RcloneManager>,
}

pub async fn shutdown_app(app: tauri::AppHandle) {
    {
        let state = app.state::<AppState>();
        let mut rclone = state.rclone.lock().await;
        let _ = rclone.unmount_all().await;
        rclone.cleanup_stale_processes().await;
        let _ = state.alist.lock().await.stop().await;
    }

    app.exit(0);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let logs = shared_log_buffer();
            let config_store = ConfigStore::new(&app_handle);
            let config = config_store.get();
            let alist = AListManager::new(app_handle.clone(), logs.clone(), &config);
            let rclone = RcloneManager::new(app_handle.clone(), logs.clone(), &config);

            app.manage(AppState {
                app: app_handle.clone(),
                config: Mutex::new(config_store),
                logs,
                alist: Mutex::new(alist),
                rclone: Mutex::new(rclone),
            });

            tray::setup_tray(app)?;
            if config.start_minimized {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            spawn_health_monitor(app_handle.clone());
            spawn_auto_start(app_handle, config.auto_start_alist, config.auto_mount);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::alist::start_alist,
            commands::alist::stop_alist,
            commands::alist::restart_alist,
            commands::alist::get_alist_status,
            commands::alist::get_alist_password,
            commands::alist::reset_alist_password,
            commands::alist::set_alist_password,
            commands::alist::open_alist_web,
            commands::rclone::list_mounts,
            commands::rclone::create_mount,
            commands::rclone::update_mount,
            commands::rclone::delete_mount,
            commands::rclone::mount_rclone,
            commands::rclone::unmount_rclone,
            commands::rclone::unmount_all_rclone,
            commands::rclone::open_mount_path,
            commands::rclone::mount_auto_rclone,
            commands::logs::list_logs,
            commands::logs::clear_logs,
            commands::settings::get_app_config,
            commands::settings::save_app_config,
            commands::settings::set_autostart,
            commands::settings::get_autostart_enabled,
            commands::settings::check_updates,
            commands::system::get_platform_info,
            commands::system::get_runtime_readiness,
            commands::system::get_winfsp_status,
            commands::system::install_winfsp,
            commands::system::hide_main_window,
            commands::system::exit_app
        ])
        .on_window_event(|window, event| {
            if window.label() == "main" {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let app = window.app_handle().clone();
                    tauri::async_runtime::spawn(async move {
                        let state = app.state::<AppState>();
                        let close_action = state.config.lock().await.get().close_action;

                        match close_action {
                            CloseAction::Minimize => {
                                if let Some(window) = app.get_webview_window("main") {
                                    let _ = window.hide();
                                }
                            }
                            CloseAction::Exit => shutdown_app(app).await,
                            CloseAction::Ask => {
                                let _ = app.emit("app-close-requested", ());
                            }
                        }
                    });
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn spawn_health_monitor(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            interval.tick().await;
            let state = app.state::<AppState>();
            let status = state.alist.lock().await.get_status().await;

            if let Ok(info) = status {
                let _ = app.emit("alist-status", info);
            }
        }
    });
}

fn spawn_auto_start(app: tauri::AppHandle, auto_start_alist: bool, auto_mount: bool) {
    if !auto_start_alist {
        return;
    }

    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();

        if state.alist.lock().await.start().await.is_err() || !auto_mount {
            return;
        }

        tokio::time::sleep(Duration::from_secs(3)).await;

        let password = {
            let alist = state.alist.lock().await;
            alist.get_admin_password().await
        };

        if let Ok(password) = password {
            let _ = state
                .rclone
                .lock()
                .await
                .mount_auto_configs(&password)
                .await;
        }
    });
}
