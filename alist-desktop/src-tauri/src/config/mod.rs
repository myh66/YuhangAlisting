use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub alist_port: u16,
    pub auto_start_alist: bool,
    pub auto_mount: bool,
    pub theme: ThemeMode,
    pub language: Language,
    pub alist_binary_path: Option<String>,
    pub rclone_binary_path: Option<String>,
    pub check_updates: bool,
    pub start_minimized: bool,
    pub close_action: CloseAction,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Language {
    #[serde(rename = "zh-CN")]
    ZhCn,
    #[serde(rename = "en-US")]
    EnUs,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CloseAction {
    Ask,
    Minimize,
    Exit,
}

pub struct ConfigStore {
    path: PathBuf,
    config: AppConfig,
}

impl ConfigStore {
    pub fn new(app: &AppHandle) -> Self {
        let path = app_data_dir(app).join("config.json");
        let config = read_json::<AppConfig>(&path).unwrap_or_default();

        Self { path, config }
    }

    pub fn get(&self) -> AppConfig {
        self.config.clone()
    }

    pub fn save(&mut self, config: AppConfig) -> Result<AppConfig, String> {
        self.config = config;
        self.flush()?;
        Ok(self.config.clone())
    }

    fn flush(&self) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|err| format!("create config dir failed: {err}"))?;
        }

        let json = serde_json::to_string_pretty(&self.config)
            .map_err(|err| format!("serialize config failed: {err}"))?;
        fs::write(&self.path, json).map_err(|err| format!("write config failed: {err}"))
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            alist_port: 5244,
            auto_start_alist: false,
            auto_mount: false,
            theme: ThemeMode::System,
            language: Language::ZhCn,
            alist_binary_path: None,
            rclone_binary_path: None,
            check_updates: true,
            start_minimized: false,
            close_action: CloseAction::Ask,
        }
    }
}

pub fn app_data_dir(app: &AppHandle) -> PathBuf {
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| fallback_project_root().join(".alist-desktop"))
}

pub fn fallback_project_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    if cwd
        .file_name()
        .is_some_and(|name| name == std::ffi::OsStr::new("src-tauri"))
    {
        return cwd.parent().unwrap_or(&cwd).to_path_buf();
    }

    cwd
}

fn read_json<T>(path: &PathBuf) -> Option<T>
where
    T: for<'de> Deserialize<'de>,
{
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}
