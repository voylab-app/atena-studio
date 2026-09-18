use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use chrono::Utc;
use tauri::{AppHandle, Emitter};
use crate::core::server::{DeveloperLogEntry, ServerRequestLog};

static REQ_COUNTER: AtomicU64 = AtomicU64::new(100);
static DEV_LOG_COUNTER: AtomicU64 = AtomicU64::new(1000);

static SERVER_LOGS: Mutex<Vec<ServerRequestLog>> = Mutex::new(Vec::new());
static SERVER_DEV_LOGS: Mutex<Vec<DeveloperLogEntry>> = Mutex::new(Vec::new());
static APP_HANDLE: Mutex<Option<AppHandle>> = Mutex::new(None);

pub struct LocalServerController;

impl LocalServerController {
    /// Sets the application handle to allow emitting real-time log events to the frontend.
    pub fn init_handle(app: AppHandle) {
        if let Ok(mut h) = APP_HANDLE.lock() {
            *h = Some(app);
        }
        Self::seed_initial_developer_logs_if_empty();
    }

    fn seed_initial_developer_logs_if_empty() {
        if let Ok(mut dev_logs) = SERVER_DEV_LOGS.lock() {
            if dev_logs.is_empty() {
                let now = Utc::now();
                let entries = vec![
                    DeveloperLogEntry {
                        id: format!("dev-{}", DEV_LOG_COUNTER.fetch_add(1, Ordering::Relaxed)),
                        timestamp: now,
                        level: "INFO".to_string(),
                        tag: Some("ATENA SERVER".to_string()),
                        message: "Success! Cognitive Runtime & HTTP server active".to_string(),
                        details: None,
                    },
                    DeveloperLogEntry {
                        id: format!("dev-{}", DEV_LOG_COUNTER.fetch_add(1, Ordering::Relaxed)),
                        timestamp: now,
                        level: "INFO".to_string(),
                        tag: Some("ATENA SERVER".to_string()),
                        message: "Supported endpoints:\n  OpenAI-compatible:\n    -> GET  /v1/models\n    -> POST /v1/chat/completions\n    -> POST /v1/embeddings".to_string(),
                        details: None,
                    },
                    DeveloperLogEntry {
                        id: format!("dev-{}", DEV_LOG_COUNTER.fetch_add(1, Ordering::Relaxed)),
                        timestamp: now,
                        level: "INFO".to_string(),
                        tag: None,
                        message: "Server started. Just-in-time model loading active.".to_string(),
                        details: None,
                    },
                ];
                dev_logs.extend(entries);
            }
        }
    }

    /// Records an LM Studio-style developer log entry (terminal stream).
    pub fn record_dev_log(
        level: &str,
        tag: Option<&str>,
        message: &str,
        details: Option<&str>,
    ) {
        let entry = DeveloperLogEntry {
            id: format!("dev-{}", DEV_LOG_COUNTER.fetch_add(1, Ordering::Relaxed)),
            timestamp: Utc::now(),
            level: level.to_uppercase(),
            tag: tag.map(|s| s.to_string()),
            message: message.to_string(),
            details: details.map(|s| s.to_string()),
        };

        if let Ok(mut logs) = SERVER_DEV_LOGS.lock() {
            logs.push(entry.clone());
            if logs.len() > 1000 {
                let overflow = logs.len() - 1000;
                logs.drain(0..overflow);
            }
        }

        if let Ok(h) = APP_HANDLE.lock() {
            if let Some(ref app) = *h {
                let _ = app.emit("developer_log_entry", entry);
            }
        }
    }

    /// Records an HTTP request into the in-memory circular buffer (newest first, max 200 items).
    pub fn record_log(
        method: &str,
        path: &str,
        status_code: u16,
        latency_ms: u64,
        tokens_prompt: usize,
        tokens_completion: usize,
    ) {
        Self::record_log_detailed(
            method,
            path,
            status_code,
            latency_ms,
            tokens_prompt,
            tokens_completion,
            None,
            None,
        );
    }

    /// Records an HTTP request with extended model and body preview information.
    pub fn record_log_detailed(
        method: &str,
        path: &str,
        status_code: u16,
        latency_ms: u64,
        tokens_prompt: usize,
        tokens_completion: usize,
        model: Option<&str>,
        body_preview: Option<&str>,
    ) {
        let req_num = REQ_COUNTER.fetch_add(1, Ordering::Relaxed);
        let log_entry = ServerRequestLog {
            id: format!("req-{}", req_num),
            timestamp: Utc::now(),
            method: method.to_uppercase(),
            path: path.to_string(),
            status_code,
            latency_ms,
            tokens_prompt,
            tokens_completion,
            model: model.map(|s| s.to_string()),
            body_preview: body_preview.map(|s| s.to_string()),
        };

        if let Ok(mut logs) = SERVER_LOGS.lock() {
            logs.insert(0, log_entry);
            if logs.len() > 200 {
                logs.truncate(200);
            }
        }

        if let Ok(h) = APP_HANDLE.lock() {
            if let Some(ref app) = *h {
                let _ = app.emit("server_logs_updated", ());
            }
        }
    }

    /// Returns all recorded HTTP logs (newest first).
    pub fn get_logs() -> Vec<ServerRequestLog> {
        SERVER_LOGS
            .lock()
            .map(|logs| logs.clone())
            .unwrap_or_default()
    }

    /// Returns all recorded developer logs (chronological order, oldest to newest).
    pub fn get_developer_logs() -> Vec<DeveloperLogEntry> {
        Self::seed_initial_developer_logs_if_empty();
        SERVER_DEV_LOGS
            .lock()
            .map(|logs| logs.clone())
            .unwrap_or_default()
    }

    /// Clears all recorded HTTP logs from memory.
    pub fn clear_logs() {
        if let Ok(mut logs) = SERVER_LOGS.lock() {
            logs.clear();
        }
        if let Ok(h) = APP_HANDLE.lock() {
            if let Some(ref app) = *h {
                let _ = app.emit("server_logs_updated", ());
            }
        }
    }

    /// Clears all recorded developer logs from memory.
    pub fn clear_developer_logs() {
        if let Ok(mut logs) = SERVER_DEV_LOGS.lock() {
            logs.clear();
        }
        if let Ok(h) = APP_HANDLE.lock() {
            if let Some(ref app) = *h {
                let _ = app.emit("developer_logs_cleared", ());
            }
        }
    }

    pub fn mock_initial_logs() -> Vec<ServerRequestLog> {
        Self::get_logs()
    }
}
