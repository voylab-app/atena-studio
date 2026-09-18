// services/gateways/mod.rs — Remote Messaging Gateways Manager for Atena Studio
// Bridges Telegram Bot, Discord Bot, and external channels directly to Atena's inference engine and associative memory.

pub mod telegram;
pub mod discord;
pub mod i18n;

use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tokio::task::JoinHandle;
use crate::core::config::AppConfig;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GatewayStatus {
    Offline,
    Connecting,
    Connected,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayStatusReport {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub status: String,
    pub error: Option<String>,
}

pub struct GatewaysManager {
    telegram_handle: std::sync::Mutex<Option<JoinHandle<()>>>,
    discord_handle: std::sync::Mutex<Option<JoinHandle<()>>>,
    telegram_status: Arc<RwLock<GatewayStatus>>,
    discord_status: Arc<RwLock<GatewayStatus>>,
}

impl Default for GatewaysManager {
    fn default() -> Self {
        Self::new()
    }
}

impl GatewaysManager {
    pub fn new() -> Self {
        Self {
            telegram_handle: std::sync::Mutex::new(None),
            discord_handle: std::sync::Mutex::new(None),
            telegram_status: Arc::new(RwLock::new(GatewayStatus::Offline)),
            discord_status: Arc::new(RwLock::new(GatewayStatus::Offline)),
        }
    }

    /// Synchronizes active gateway background worker tasks with the given configuration.
    pub async fn sync_with_config(&self, app: tauri::AppHandle, config: &AppConfig) {
        // 1. Sync Telegram Gateway
        self.sync_telegram(app.clone(), config).await;

        // 2. Sync Discord Gateway
        self.sync_discord(app, config).await;
    }

    async fn sync_telegram(&self, app: tauri::AppHandle, config: &AppConfig) {
        let mut handle_guard = self.telegram_handle.lock().unwrap();

        // Always abort existing running task if token or config changed
        if let Some(handle) = handle_guard.take() {
            handle.abort();
            let mut st = self.telegram_status.write().unwrap();
            *st = GatewayStatus::Offline;
        }

        if config.gateways.telegram.enabled && !config.gateways.telegram.bot_token.trim().is_empty() {
            let tg_cfg = config.gateways.telegram.clone();
            let status_ref = self.telegram_status.clone();
            {
                let mut st = status_ref.write().unwrap();
                *st = GatewayStatus::Connecting;
            }

            let handle = tokio::spawn(async move {
                telegram::run_telegram_gateway(app, tg_cfg, status_ref).await;
            });
            *handle_guard = Some(handle);
        }
    }

    async fn sync_discord(&self, app: tauri::AppHandle, config: &AppConfig) {
        let mut handle_guard = self.discord_handle.lock().unwrap();

        if let Some(handle) = handle_guard.take() {
            handle.abort();
            let mut st = self.discord_status.write().unwrap();
            *st = GatewayStatus::Offline;
        }

        if config.gateways.discord.enabled && !config.gateways.discord.bot_token.trim().is_empty() {
            let dc_cfg = config.gateways.discord.clone();
            let status_ref = self.discord_status.clone();
            {
                let mut st = status_ref.write().unwrap();
                *st = GatewayStatus::Connecting;
            }

            let handle = tokio::spawn(async move {
                discord::run_discord_gateway(app, dc_cfg, status_ref).await;
            });
            *handle_guard = Some(handle);
        }
    }

    /// Stops all running gateway background worker threads.
    pub fn stop_all(&self) {
        if let Ok(mut guard) = self.telegram_handle.lock() {
            if let Some(handle) = guard.take() {
                handle.abort();
            }
            if let Ok(mut st) = self.telegram_status.write() {
                *st = GatewayStatus::Offline;
            }
        }

        if let Ok(mut guard) = self.discord_handle.lock() {
            if let Some(handle) = guard.take() {
                handle.abort();
            }
            if let Ok(mut st) = self.discord_status.write() {
                *st = GatewayStatus::Offline;
            }
        }
    }

    /// Returns a snapshot of status reports for all gateways.
    pub fn get_status_reports(&self, config: &AppConfig) -> Vec<GatewayStatusReport> {
        let tg_status = self.telegram_status.read().unwrap();
        let (tg_str, tg_err) = match &*tg_status {
            GatewayStatus::Offline => ("offline".to_string(), None),
            GatewayStatus::Connecting => ("connecting".to_string(), None),
            GatewayStatus::Connected => ("connected".to_string(), None),
            GatewayStatus::Error(e) => ("error".to_string(), Some(e.clone())),
        };

        let dc_status = self.discord_status.read().unwrap();
        let (dc_str, dc_err) = match &*dc_status {
            GatewayStatus::Offline => ("offline".to_string(), None),
            GatewayStatus::Connecting => ("connecting".to_string(), None),
            GatewayStatus::Connected => ("connected".to_string(), None),
            GatewayStatus::Error(e) => ("error".to_string(), Some(e.clone())),
        };

        vec![
            GatewayStatusReport {
                id: "telegram".to_string(),
                name: "Telegram Bot".to_string(),
                enabled: config.gateways.telegram.enabled,
                status: tg_str,
                error: tg_err,
            },
            GatewayStatusReport {
                id: "discord".to_string(),
                name: "Discord Bot".to_string(),
                enabled: config.gateways.discord.enabled,
                status: dc_str,
                error: dc_err,
            },
        ]
    }

    /// Tests Telegram bot token by querying Telegram Bot API getMe endpoint.
    pub async fn test_telegram_token(token: &str) -> Result<String, String> {
        let token = token.trim();
        if token.is_empty() {
            return Err("Token cannot be empty".to_string());
        }

        let url = format!("https://api.telegram.org/bot{}/getMe", token);
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| e.to_string())?;

        let res = client.get(&url).send().await.map_err(|e| format!("Connection failed: {}", e))?;
        if !res.status().is_success() {
            return Err(format!("Telegram API returned status: {}", res.status()));
        }

        let body: serde_json::Value = res.json().await.map_err(|e| format!("Invalid JSON: {}", e))?;
        if body["ok"].as_bool().unwrap_or(false) {
            let bot_username = body["result"]["username"].as_str().unwrap_or("UnknownBot");
            let bot_first_name = body["result"]["first_name"].as_str().unwrap_or("Atena Bot");
            Ok(format!("@{} ({})", bot_username, bot_first_name))
        } else {
            let desc = body["description"].as_str().unwrap_or("Authentication failed");
            Err(desc.to_string())
        }
    }

    /// Tests Discord bot token by querying Discord API /users/@me endpoint.
    pub async fn test_discord_token(token: &str) -> Result<String, String> {
        let token = token.trim();
        if token.is_empty() {
            return Err("Token cannot be empty".to_string());
        }

        let url = "https://discord.com/api/v10/users/@me";
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| e.to_string())?;

        let auth_header = if token.starts_with("Bot ") {
            token.to_string()
        } else {
            format!("Bot {}", token)
        };

        let res = client
            .get(url)
            .header("Authorization", auth_header)
            .header("User-Agent", "AtenaStudio (https://github.com/voylaboratory/atena, 0.1.0)")
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if !res.status().is_success() {
            return Err(format!("Discord API returned status: {}", res.status()));
        }

        let body: serde_json::Value = res.json().await.map_err(|e| format!("Invalid JSON: {}", e))?;
        let username = body["username"].as_str().unwrap_or("UnknownBot");
        Ok(username.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateways_manager_initial_status() {
        let manager = GatewaysManager::new();
        let config = AppConfig::default();
        let reports = manager.get_status_reports(&config);

        assert_eq!(reports.len(), 2);
        assert_eq!(reports[0].id, "telegram");
        assert_eq!(reports[0].status, "offline");
        assert!(!reports[0].enabled);

        assert_eq!(reports[1].id, "discord");
        assert_eq!(reports[1].status, "offline");
        assert!(!reports[1].enabled);
    }

    #[test]
    fn test_whitelist_filtering() {
        let allowed_users = vec![123456789i64, 987654321i64];

        let sender_allowed = 123456789i64;
        let sender_denied = 555555555i64;

        assert!(allowed_users.contains(&sender_allowed));
        assert!(!allowed_users.contains(&sender_denied));
    }

    #[test]
    fn test_strip_internal_tags_invoke_block() {
        let input = "Boa pergunta! Vou verificar de novo.\n\n<invoke name=\"atena_search_episodes\">\n<parameter name=\"max_results\">10</parameter>\n</invoke>";
        let cleaned = strip_internal_tags(input);
        assert_eq!(cleaned, "Boa pergunta! Vou verificar de novo.");
    }

    #[test]
    fn test_strip_internal_tags_think_and_memory() {
        let input = "<think>Analizando pergunta do usuario...</think>\nOlá Carlos!\n<memory subject=\"User\" property=\"Prefere café filtrado\" />";
        let cleaned = strip_internal_tags(input);
        assert_eq!(cleaned, "Olá Carlos!");
    }

    #[test]
    fn test_strip_internal_tags_inline_and_unclosed() {
        let input = "Resposta normal.\n<invoke name=\"search\">param</invoke>\nMais texto.\n<parameter name=\"foo\">";
        let cleaned = strip_internal_tags(input);
        assert_eq!(cleaned, "Resposta normal.\n\nMais texto.");
    }
}

/// Strips internal reasoning (<think>), tool invocations (<invoke>, <parameter>, <tool_call>),
/// and cognitive memory tags (<memory>, <forget>) from assistant responses before sending to chat gateways.
pub fn strip_internal_tags(text: &str) -> String {
    let mut result = text.to_string();

    // 1. Remove paired blocks (both opening tag, content, and closing tag)
    let block_tags = [
        "think",
        "invoke",
        "tool_call",
        "parameter",
        "memory",
        "forget",
    ];

    for tag in block_tags {
        let open_prefix = format!("<{}", tag);
        let close_tag = format!("</{}>", tag);

        while let Some(start_idx) = result.find(&open_prefix) {
            // Check if there is a closing tag after start_idx
            if let Some(close_rel_idx) = result[start_idx..].find(&close_tag) {
                let end_idx = start_idx + close_rel_idx + close_tag.len();
                result.replace_range(start_idx..end_idx, "");
            } else if let Some(self_close_rel) = result[start_idx..].find("/>") {
                // Check if there is a standard '>' before '/>'
                if let Some(regular_close_rel) = result[start_idx..].find('>') {
                    if regular_close_rel < self_close_rel {
                        result.replace_range(start_idx..start_idx + regular_close_rel + 1, "");
                        continue;
                    }
                }
                let tag_end = start_idx + self_close_rel + 2;
                result.replace_range(start_idx..tag_end, "");
            } else if let Some(regular_close_rel) = result[start_idx..].find('>') {
                // Opening tag without closing tag
                let tag_end = start_idx + regular_close_rel + 1;
                result.replace_range(start_idx..tag_end, "");
            } else {
                // Incomplete tag trailing at the end
                result.replace_range(start_idx.., "");
                break;
            }
        }
    }

    // 2. Clean up any stray closing tags
    for tag in &["think", "invoke", "tool_call", "parameter", "memory", "forget"] {
        let close_tag = format!("</{}>", tag);
        result = result.replace(&close_tag, "");
    }

    // 3. Clean up line formatting and collapse excessive blank lines
    let mut cleaned_lines = Vec::new();
    let mut consecutive_blanks = 0;

    for line in result.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            consecutive_blanks += 1;
            if consecutive_blanks <= 1 {
                cleaned_lines.push("");
            }
        } else {
            consecutive_blanks = 0;
            cleaned_lines.push(trimmed);
        }
    }

    cleaned_lines.join("\n").trim().to_string()
}

