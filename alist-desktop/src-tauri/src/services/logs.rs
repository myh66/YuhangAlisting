use serde::Serialize;
use std::{
    collections::VecDeque,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

pub type SharedLogBuffer = Arc<Mutex<LogBuffer>>;

const MAX_LOGS: usize = 2_000;

#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub timestamp: u64,
    pub source: String,
    pub level: String,
    pub message: String,
}

pub struct LogBuffer {
    entries: VecDeque<LogEntry>,
}

impl LogBuffer {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::with_capacity(MAX_LOGS),
        }
    }

    pub fn list(&self) -> Vec<LogEntry> {
        self.entries.iter().cloned().collect()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    fn push(&mut self, entry: LogEntry) {
        if self.entries.len() >= MAX_LOGS {
            self.entries.pop_front();
        }

        self.entries.push_back(entry);
    }
}

pub fn shared_log_buffer() -> SharedLogBuffer {
    Arc::new(Mutex::new(LogBuffer::new()))
}

pub async fn emit_log(
    app: &AppHandle,
    logs: &SharedLogBuffer,
    source: impl Into<String>,
    level: impl Into<String>,
    message: impl Into<String>,
) {
    let entry = LogEntry {
        timestamp: now_millis(),
        source: source.into(),
        level: level.into(),
        message: message.into(),
    };

    logs.lock().await.push(entry.clone());
    let _ = app.emit("service-log", entry);
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}
