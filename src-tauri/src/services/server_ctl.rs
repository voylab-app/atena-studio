use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use chrono::Utc;
use crate::core::server::ServerRequestLog;

static REQ_COUNTER: AtomicU64 = AtomicU64::new(100);
static SERVER_LOGS: Mutex<Vec<ServerRequestLog>> = Mutex::new(Vec::new());

pub struct LocalServerController;

impl LocalServerController {
    /// Records an HTTP request into the in-memory circular buffer (newest first, max 200 items).
    pub fn record_log(
        method: &str,
        path: &str,
        status_code: u16,
        latency_ms: u64,
        tokens_prompt: usize,
        tokens_completion: usize,
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
        };

        if let Ok(mut logs) = SERVER_LOGS.lock() {
            logs.insert(0, log_entry);
            if logs.len() > 200 {
                logs.truncate(200);
            }
        }
    }

    /// Returns all recorded logs (newest first).
    pub fn get_logs() -> Vec<ServerRequestLog> {
        SERVER_LOGS
            .lock()
            .map(|logs| logs.clone())
            .unwrap_or_default()
    }

    /// Clears all recorded logs from memory.
    pub fn clear_logs() {
        if let Ok(mut logs) = SERVER_LOGS.lock() {
            logs.clear();
        }
    }

    pub fn mock_initial_logs() -> Vec<ServerRequestLog> {
        Self::get_logs()
    }
}

