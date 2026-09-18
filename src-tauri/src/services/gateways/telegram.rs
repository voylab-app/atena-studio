// services/gateways/telegram.rs — Telegram Bot Gateway implementation
// Connects Telegram chats directly to Atena's internal streaming inference and associative memory.

use std::sync::{Arc, RwLock};
use std::time::Duration;
use serde_json::json;
use crate::core::config::TelegramConfig;
use crate::services::gateways::GatewayStatus;
use tauri::Manager;
use crate::AppState;

pub async fn run_telegram_gateway(
    app: tauri::AppHandle,
    config: TelegramConfig,
    status: Arc<RwLock<GatewayStatus>>,
) {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(35))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            log::error!("[Telegram Gateway] Failed to build HTTP client: {}", e);
            let mut st = status.write().unwrap();
            *st = GatewayStatus::Error(format!("HTTP client error: {}", e));
            return;
        }
    };

    let token = config.bot_token.trim().to_string();
    let get_me_url = format!("https://api.telegram.org/bot{}/getMe", token);

    // 1. Initial health check
    match client.get(&get_me_url).send().await {
        Ok(res) if res.status().is_success() => {
            if let Ok(body) = res.json::<serde_json::Value>().await {
                if body["ok"].as_bool().unwrap_or(false) {
                    let bot_username = body["result"]["username"].as_str().unwrap_or("AtenaBot");
                    log::info!("[Telegram Gateway] Connected successfully as @{}", bot_username);
                    let mut st = status.write().unwrap();
                    *st = GatewayStatus::Connected;
                } else {
                    let err = body["description"].as_str().unwrap_or("Invalid bot token");
                    log::error!("[Telegram Gateway] Auth failed: {}", err);
                    let mut st = status.write().unwrap();
                    *st = GatewayStatus::Error(err.to_string());
                    return;
                }
            }
        }
        Ok(res) => {
            let err = format!("API returned HTTP status {}", res.status());
            log::error!("[Telegram Gateway] Auth failed: {}", err);
            let mut st = status.write().unwrap();
            *st = GatewayStatus::Error(err);
            return;
        }
        Err(e) => {
            let err = format!("Connection failed: {}", e);
            log::error!("[Telegram Gateway] Auth failed: {}", err);
            let mut st = status.write().unwrap();
            *st = GatewayStatus::Error(err);
            return;
        }
    }

    // 2. Long Polling loop
    let mut offset: i64 = 0;
    let base_url = format!("https://api.telegram.org/bot{}", token);

    log::info!("[Telegram Gateway] Starting long polling loop (offset: {})...", offset);

    loop {
        let poll_url = format!("{}/getUpdates?offset={}&timeout=25", base_url, offset);
        match client.get(&poll_url).send().await {
            Ok(res) if res.status().is_success() => {
                if let Ok(body) = res.json::<serde_json::Value>().await {
                    if let Some(updates) = body["result"].as_array() {
                        for upd in updates {
                            if let Some(upd_id) = upd["update_id"].as_i64() {
                                offset = upd_id + 1;
                            }

                            if let Some(msg) = upd.get("message") {
                                let chat_id = msg["chat"]["id"].as_i64().unwrap_or(0);
                                let sender_id = msg["from"]["id"].as_i64().unwrap_or(0);
                                let text = msg["text"].as_str().unwrap_or("").trim().to_string();

                                if chat_id == 0 || text.is_empty() {
                                    continue;
                                }

                                // Security Whitelist Guardrail
                                if !config.allowed_user_ids.is_empty()
                                    && !config.allowed_user_ids.contains(&sender_id)
                                {
                                    log::warn!(
                                        "[Telegram Gateway] Unauthorized message from user ID {}",
                                        sender_id
                                    );
                                    let warning = format!(
                                        "⚠️ **Access Restricted**\n\nYour Telegram User ID is `{}`.\nTo authorize this account, add this ID in **Atena Studio → Settings → Gateways → Telegram**.",
                                        sender_id
                                    );
                                    let _ = send_telegram_message(&client, &base_url, chat_id, &warning).await;
                                    continue;
                                }

                                // Handle commands & prompt execution
                                handle_telegram_message(
                                    app.clone(),
                                    &client,
                                    &base_url,
                                    chat_id,
                                    sender_id,
                                    text,
                                    &config,
                                )
                                .await;
                            }
                        }
                    }
                }
            }
            Ok(res) => {
                log::warn!("[Telegram Gateway] getUpdates returned HTTP {}", res.status());
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
            Err(e) => {
                log::debug!("[Telegram Gateway] Network interruption during polling: {}", e);
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
}

async fn handle_telegram_message(
    app: tauri::AppHandle,
    client: &reqwest::Client,
    base_url: &str,
    chat_id: i64,
    sender_id: i64,
    text: String,
    config: &TelegramConfig,
) {
    let state = app.state::<AppState>();
    let app_cfg = crate::core::config::AppConfig::load();
    let lang = app_cfg.language.as_str();

    // 1. Slash commands
    if text.starts_with('/') {
        let parts: Vec<&str> = text.split_whitespace().collect();
        let cmd = parts[0].to_lowercase();

        match cmd.as_str() {
            "/start" | "/help" => {
                let msg = crate::services::gateways::i18n::get_help_msg(lang);
                let _ = send_telegram_message(client, base_url, chat_id, msg).await;
                return;
            }
            "/id" => {
                let msg = crate::services::gateways::i18n::get_user_id_msg(lang, &sender_id.to_string());
                let _ = send_telegram_message(client, base_url, chat_id, &msg).await;
                return;
            }
            "/model" => {
                let active_model = state.active_model.lock().await.clone();
                let model_name = active_model
                    .map(|m| format!("{} ({:?})", m.name, m.backend))
                    .unwrap_or_else(|| crate::services::gateways::i18n::get_no_model_msg(lang).to_string());
                let msg = crate::services::gateways::i18n::get_model_msg(lang, &model_name);
                let _ = send_telegram_message(client, base_url, chat_id, &msg).await;
                return;
            }
            "/new" => {
                // Clear active session messages in SQLite
                let session_id = format!("telegram_{}", chat_id);
                let now = chrono::Utc::now().to_rfc3339();
                let new_session = crate::services::db::DbChatSession {
                    id: session_id.clone(),
                    title: format!("Telegram Chat {}", chat_id),
                    model_id: None,
                    model_name: None,
                    project_id: None,
                    created_at: now.clone(),
                    updated_at: Some(now),
                    messages: Vec::new(),
                    is_private: Some(false),
                    pinned: Some(false),
                    archived: Some(false),
                    archived_at: None,
                    extra: std::collections::HashMap::new(),
                };
                let _ = state.db.save_session(&new_session);
                {
                    use tauri::Emitter;
                    let _ = app.emit("db_sessions_updated", ());
                }
                let msg = crate::services::gateways::i18n::get_fresh_conversation_msg(lang);
                let _ = send_telegram_message(client, base_url, chat_id, msg).await;
                return;
            }
            "/memory" => {
                let query = parts.get(1..).map(|p| p.join(" ")).unwrap_or_default();
                if query.trim().is_empty() {
                    let msg = crate::services::gateways::i18n::get_memory_usage_msg(lang);
                    let _ = send_telegram_message(client, base_url, chat_id, msg).await;
                    return;
                }

                let mut engine = state.memory_engine.lock().await;
                if let Some(context) = engine.search_active_context_for_query(&query) {
                    let report = crate::services::gateways::i18n::get_memory_found_msg(lang, &query, &context);
                    let _ = send_telegram_message(client, base_url, chat_id, &report).await;
                } else {
                    let report = crate::services::gateways::i18n::get_memory_not_found_msg(lang, &query);
                    let _ = send_telegram_message(client, base_url, chat_id, &report).await;
                }
                return;
            }
            _ => {}
        }
    }

    // 2. Chat Inference execution
    let session_id = format!("telegram_{}", chat_id);

    // Send initial typing action
    let _ = send_chat_action(client, base_url, chat_id, "typing").await;

    // Load recent conversation history from SQLite with sliding window
    let max_history = if config.sliding_window > 0 {
        config.sliding_window
    } else {
        4
    };

    let mut messages: Vec<crate::core::model::ChatMessage> = if let Ok(Some(sess)) = state.db.get_session(&session_id) {
        let total = sess.messages.len();
        let skip_count = total.saturating_sub(max_history);
        sess.messages
            .into_iter()
            .skip(skip_count)
            .map(|m| crate::core::model::ChatMessage {
                role: m.role,
                content: m.content,
                images: m.images,
                tool_calls: None,
                tool_call_id: m.tool_call_id,
            })
            .collect()
    } else {
        Vec::new()
    };
    messages.push(crate::core::model::ChatMessage {
        role: "user".to_string(),
        content: text.clone(),
        images: None,
        tool_calls: None,
        tool_call_id: None,
    });

    let active_model = state.active_model.lock().await.clone();
    let app_cfg = crate::core::config::AppConfig::load();

    let default_sys_prompt = crate::services::gateways::i18n::get_default_system_prompt(lang);
    let system_prompt = config
        .custom_system_prompt
        .clone()
        .unwrap_or_else(|| default_sys_prompt.to_string());

    let mut params = crate::core::model::InferenceParams::default();
    if config.enable_tools {
        params.mcp_tools = Some(state.mcp_manager.list_all_tools().await);
    }

    let req = crate::StreamChatRequest {
        model: active_model.clone(),
        system_prompt: system_prompt.clone(),
        messages: Some(messages.clone()),
        user_message: Some(text.clone()),
        session_id: Some(session_id.clone()),
        session_title: Some(format!("Telegram User {}", sender_id)),
        params: params.clone(),
        mlx_host: app_cfg.mlx_server_host.clone(),
        mlx_port: app_cfg.mlx_server_port,
        ollama_host: app_cfg.ollama_host.clone(),
        ollama_port: app_cfg.ollama_port,
        enable_memory: Some(config.enable_memory),
        enable_facts_memory: Some(config.enable_memory),
        enable_skills_memory: Some(config.enable_tools),
        enable_episodic_memory: Some(config.enable_memory),
    };

    let accumulated_text = Arc::new(std::sync::Mutex::new(String::new()));
    let acc_clone = accumulated_text.clone();
    let captured_tool_calls = Arc::new(std::sync::Mutex::new(Vec::new()));
    let tc_clone = captured_tool_calls.clone();

    let on_chunk = Arc::new(move |payload: crate::ChatChunkPayload| {
        if let Ok(mut text_guard) = acc_clone.lock() {
            *text_guard = payload.content;
        }
        if let Some(tcs) = payload.tool_calls {
            if let Ok(mut tc_guard) = tc_clone.lock() {
                *tc_guard = tcs;
            }
        }
    });

    let client_clone = client.clone();
    let base_url_str = base_url.to_string();

    // Spawn periodic typing action during inference
    let typing_task = tokio::spawn(async move {
        for _ in 0..60 {
            tokio::time::sleep(Duration::from_secs(4)).await;
            let _ = send_chat_action(&client_clone, &base_url_str, chat_id, "typing").await;
        }
    });

    let result = crate::execute_stream_chat_internal(&state, req, on_chunk).await;
    typing_task.abort();

    if let Err(e) = result {
        let err_msg = format!("⚠️ Inference Error: {}", e);
        let _ = send_telegram_message(client, base_url, chat_id, &err_msg).await;
        return;
    }

    // Check if the assistant requested tool calls to execute
    let tool_calls = captured_tool_calls.lock().unwrap().clone();
    if config.enable_tools && !tool_calls.is_empty() {
        let mut current_tool_calls = tool_calls;
        let mut loop_count = 0;
        while !current_tool_calls.is_empty() && loop_count < 3 {
            loop_count += 1;
            log::info!("[Telegram Gateway] Executing {} tool call(s) (loop {})", current_tool_calls.len(), loop_count);
            let _ = send_chat_action(client, base_url, chat_id, "typing").await;

            let mut executed_tools = Vec::new();
            for mut tc in current_tool_calls {
                let server_id = tc.server_id.clone().unwrap_or_else(|| {
                    if tc.name.starts_with("atena_") {
                        "atena_native".to_string()
                    } else {
                        "skills".to_string()
                    }
                });

                log::info!("[Telegram Gateway] Invoking tool '{}' on '{}' with args: {:?}", tc.name, server_id, tc.arguments);
                match crate::execute_tool_call_internal(&state, &server_id, &tc.name, tc.arguments.clone()).await {
                    Ok(res) => {
                        log::info!("[Telegram Gateway] Tool '{}' succeeded: {:?}", tc.name, res);
                        tc.status = Some("completed".to_string());
                        tc.result = Some(res);
                    }
                    Err(e) => {
                        log::error!("[Telegram Gateway] Tool '{}' failed: {}", tc.name, e);
                        tc.status = Some("error".to_string());
                        tc.result = Some(serde_json::json!({ "error": e }));
                    }
                }
                executed_tools.push(tc);
            }

            let tool_results_block = crate::services::backend::BackendManager::format_tool_results_block(&executed_tools);

            let interim_text = accumulated_text.lock().unwrap().clone();
            messages.push(crate::core::model::ChatMessage {
                role: "assistant".to_string(),
                content: interim_text,
                images: None,
                tool_calls: Some(executed_tools),
                tool_call_id: None,
            });

            messages.push(crate::core::model::ChatMessage {
                role: "user".to_string(),
                content: format!("[Retorno das Ferramentas Executadas]:\n{}\n\nPor favor, apresente o resultado final diretamente ao usuário de forma clara e objetiva.", tool_results_block),
                images: None,
                tool_calls: None,
                tool_call_id: None,
            });

            let fu_accumulated = Arc::new(std::sync::Mutex::new(String::new()));
            let fu_acc_clone = fu_accumulated.clone();
            let fu_captured_calls = Arc::new(std::sync::Mutex::new(Vec::new()));
            let fu_tc_clone = fu_captured_calls.clone();

            let on_fu_chunk = Arc::new(move |payload: crate::ChatChunkPayload| {
                if let Ok(mut text_guard) = fu_acc_clone.lock() {
                    *text_guard = payload.content;
                }
                if let Some(tcs) = payload.tool_calls {
                    if let Ok(mut tc_guard) = fu_tc_clone.lock() {
                        *tc_guard = tcs;
                    }
                }
            });

            let fu_typing_task = {
                let cl = client.clone();
                let b_url = base_url.to_string();
                tokio::spawn(async move {
                    for _ in 0..60 {
                        tokio::time::sleep(Duration::from_secs(4)).await;
                        let _ = send_chat_action(&cl, &b_url, chat_id, "typing").await;
                    }
                })
            };

            let fu_req = crate::StreamChatRequest {
                model: active_model.clone(),
                system_prompt: system_prompt.clone(),
                messages: Some(messages.clone()),
                user_message: None,
                session_id: Some(session_id.clone()),
                session_title: Some(format!("Telegram User {}", sender_id)),
                params: params.clone(),
                mlx_host: app_cfg.mlx_server_host.clone(),
                mlx_port: app_cfg.mlx_server_port,
                ollama_host: app_cfg.ollama_host.clone(),
                ollama_port: app_cfg.ollama_port,
                enable_memory: Some(config.enable_memory),
                enable_facts_memory: Some(config.enable_memory),
                enable_skills_memory: Some(config.enable_tools),
                enable_episodic_memory: Some(config.enable_memory),
            };

            let _ = crate::execute_stream_chat_internal(&state, fu_req, on_fu_chunk).await;
            fu_typing_task.abort();

            let fu_final = fu_accumulated.lock().unwrap().clone();
            if !fu_final.trim().is_empty() {
                *accumulated_text.lock().unwrap() = fu_final;
            }
            current_tool_calls = fu_captured_calls.lock().unwrap().clone();
        }
    }

    let final_text = accumulated_text.lock().unwrap().clone();
    let clean_text = final_text.trim();

    if clean_text.is_empty() {
        let _ = send_telegram_message(client, base_url, chat_id, "(Empty response generated)").await;
    } else {
        // Persist user and assistant exchange into SQLite session
        let now = chrono::Utc::now().to_rfc3339();
        let mut sess = state.db.get_session(&session_id).ok().flatten().unwrap_or_else(|| {
            crate::services::db::DbChatSession {
                id: session_id.clone(),
                title: format!("Telegram Chat {}", chat_id),
                model_id: None,
                model_name: None,
                project_id: None,
                created_at: now.clone(),
                updated_at: Some(now.clone()),
                messages: Vec::new(),
                is_private: Some(false),
                pinned: Some(false),
                archived: Some(false),
                archived_at: None,
                extra: std::collections::HashMap::new(),
            }
        });
        sess.messages.push(crate::services::db::DbChatMessage {
            id: format!("msg_{}", chrono::Utc::now().timestamp_millis()),
            role: "user".to_string(),
            content: text.clone(),
            thinking_content: None,
            tool_calls: None,
            tool_call_id: None,
            images: None,
            attachments: None,
            timestamp: now.clone(),
            tokens_count: None,
            generation_speed_tps: None,
            metrics: None,
            extra: std::collections::HashMap::new(),
        });
        sess.messages.push(crate::services::db::DbChatMessage {
            id: format!("msg_{}", chrono::Utc::now().timestamp_millis() + 1),
            role: "assistant".to_string(),
            content: clean_text.to_string(),
            thinking_content: None,
            tool_calls: None,
            tool_call_id: None,
            images: None,
            attachments: None,
            timestamp: now.clone(),
            tokens_count: None,
            generation_speed_tps: None,
            metrics: None,
            extra: std::collections::HashMap::new(),
        });
        sess.updated_at = Some(now);
        let _ = state.db.save_session(&sess);
        {
            use tauri::Emitter;
            let _ = app.emit("db_sessions_updated", ());
        }

        // Strip internal thinking, tool invoke and memory tags from Telegram user display
        let user_display = crate::services::gateways::strip_internal_tags(clean_text);
        // Send with auto-chunking for Telegram's 4096 char limit
        send_chunked_telegram_message(client, base_url, chat_id, &user_display).await;
    }
}

async fn send_chat_action(client: &reqwest::Client, base_url: &str, chat_id: i64, action: &str) -> Result<(), reqwest::Error> {
    let url = format!("{}/sendChatAction", base_url);
    let _ = client
        .post(&url)
        .json(&json!({
            "chat_id": chat_id,
            "action": action
        }))
        .send()
        .await?;
    Ok(())
}

async fn send_telegram_message(
    client: &reqwest::Client,
    base_url: &str,
    chat_id: i64,
    text: &str,
) -> Result<(), reqwest::Error> {
    let url = format!("{}/sendMessage", base_url);

    // Attempt Markdown format first
    let res = client
        .post(&url)
        .json(&json!({
            "chat_id": chat_id,
            "text": text,
            "parse_mode": "Markdown"
        }))
        .send()
        .await?;

    // If Markdown parsing failed, fallback to plain text
    if !res.status().is_success() {
        let _ = client
            .post(&url)
            .json(&json!({
                "chat_id": chat_id,
                "text": text
            }))
            .send()
            .await?;
    }

    Ok(())
}

async fn send_chunked_telegram_message(
    client: &reqwest::Client,
    base_url: &str,
    chat_id: i64,
    text: &str,
) {
    const CHUNK_LIMIT: usize = 3800;

    if text.len() <= CHUNK_LIMIT {
        let _ = send_telegram_message(client, base_url, chat_id, text).await;
        return;
    }

    let mut remaining = text;
    while !remaining.is_empty() {
        if remaining.len() <= CHUNK_LIMIT {
            let _ = send_telegram_message(client, base_url, chat_id, remaining).await;
            break;
        }

        // Find clean split point on newline or space
        let split_idx = remaining[..CHUNK_LIMIT]
            .rfind('\n')
            .or_else(|| remaining[..CHUNK_LIMIT].rfind(' '))
            .unwrap_or(CHUNK_LIMIT);

        let chunk = &remaining[..split_idx];
        let _ = send_telegram_message(client, base_url, chat_id, chunk).await;
        remaining = remaining[split_idx..].trim_start();
    }
}
