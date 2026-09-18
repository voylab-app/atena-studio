use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerStatus {
    Stopped,
    Starting,
    Running { port: u16, active_model_id: String },
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerRequestLog {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub method: String,
    pub path: String,
    pub status_code: u16,
    pub latency_ms: u64,
    pub tokens_prompt: usize,
    pub tokens_completion: usize,
    pub model: Option<String>,
    pub body_preview: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeveloperLogEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub level: String, // "INFO", "DEBUG", "WARN", "ERROR"
    pub tag: Option<String>,
    pub message: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_enabled: bool,
    pub api_key: Option<String>,
    pub max_parallel_requests: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 1234,
            cors_enabled: true,
            api_key: None,
            max_parallel_requests: 4,
        }
    }
}
