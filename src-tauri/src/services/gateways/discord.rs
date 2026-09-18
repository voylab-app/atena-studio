// services/gateways/discord.rs — Discord Bot Gateway implementation
// Connects Discord channels directly to Atena's internal streaming inference and associative memory.

use std::sync::{Arc, RwLock};
use std::time::Duration;
use serde_json::json;
use crate::core::config::DiscordConfig;
use crate::services::gateways::GatewayStatus;
use tauri::Manager;
use crate::AppState;

pub async fn run_discord_gateway(
    app: tauri::AppHandle,
    config: DiscordConfig,
    status: Arc<RwLock<GatewayStatus>>,
) {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(35))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            log::error!("[Discord Gateway] Failed to build HTTP client: {}", e);
            let mut st = status.write().unwrap();
            *st = GatewayStatus::Error(format!("HTTP client error: {}", e));
            return;
        }
    };

    let token = config.bot_token.trim().to_string();
    let auth_header = if token.starts_with("Bot ") {
        token.clone()
    } else {
        format!("Bot {}", token)
    };

    // 1. Initial health check
    let get_me_url = "https://discord.com/api/v10/users/@me";
    match client
        .get(get_me_url)
        .header("Authorization", &auth_header)
        .header("User-Agent", "AtenaStudio (https://github.com/voylaboratory/atena, 0.1.0)")
        .send()
        .await
    {
        Ok(res) if res.status().is_success() => {
            if let Ok(body) = res.json::<serde_json::Value>().await {
                let username = body["username"].as_str().unwrap_or("AtenaBot");
                log::info!("[Discord Gateway] Connected successfully as {}", username);
                let mut st = status.write().unwrap();
                *st = GatewayStatus::Connected;
            }
        }
        Ok(res) => {
            let err = format!("Discord API returned HTTP status {}", res.status());
            log::error!("[Discord Gateway] Auth failed: {}", err);
            let mut st = status.write().unwrap();
            *st = GatewayStatus::Error(err);
            return;
        }
        Err(e) => {
            let err = format!("Connection failed: {}", e);
            log::error!("[Discord Gateway] Auth failed: {}", err);
            let mut st = status.write().unwrap();
            *st = GatewayStatus::Error(err);
            return;
        }
    }

    // 2. Periodic channel poller / message watcher loop
    log::info!("[Discord Gateway] Running Discord Gateway listener...");

    // Track last seen message IDs per channel to avoid duplicates
    let mut last_seen_ids: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    loop {
        // If specific allowed channels are configured, poll them
        if !config.allowed_channel_ids.is_empty() {
            for channel_id in &config.allowed_channel_ids {
                let channel_id = channel_id.trim();
                if channel_id.is_empty() {
                    continue;
                }

                let mut url = format!("https://discord.com/api/v10/channels/{}/messages?limit=5", channel_id);
                if let Some(after_id) = last_seen_ids.get(channel_id) {
                    url = format!("https://discord.com/api/v10/channels/{}/messages?after={}&limit=5", channel_id, after_id);
                }

                if let Ok(res) = client
                    .get(&url)
                    .header("Authorization", &auth_header)
                    .header("User-Agent", "AtenaStudio (https://github.com/voylaboratory/atena, 0.1.0)")
                    .send()
                    .await
                {
                    if res.status().is_success() {
                        if let Ok(messages) = res.json::<Vec<serde_json::Value>>().await {
                            // Messages arrive latest first; reverse to process chronologically
                            for msg in messages.iter().rev() {
                                let msg_id = msg["id"].as_str().unwrap_or_default().to_string();
                                let author_id = msg["author"]["id"].as_str().unwrap_or_default().to_string();
                                let is_bot = msg["author"]["bot"].as_bool().unwrap_or(false);
                                let content = msg["content"].as_str().unwrap_or_default().trim().to_string();

                                // Update latest seen ID
                                if !msg_id.is_empty() {
                                    last_seen_ids.insert(channel_id.to_string(), msg_id.clone());
                                }

                                if is_bot || content.is_empty() {
                                    continue;
                                }

                                // Security Whitelist Guardrail
                                if !config.allowed_user_ids.is_empty()
                                    && !config.allowed_user_ids.contains(&author_id)
                                {
                                    log::warn!(
                                        "[Discord Gateway] Unauthorized message from user ID {}",
                                        author_id
                                    );
                                    let warning = format!(
                                        "⚠️ **Access Restricted**\nYour Discord User ID is `{}`.\nTo authorize this account, add this ID in **Atena Studio → Settings → Gateways → Discord**.",
                                        author_id
                                    );
                                    let _ = send_discord_message(&client, &auth_header, channel_id, &warning).await;
                                    continue;
                                }

                                // Process prompt / commands
                                handle_discord_message(
                                    app.clone(),
                                    &client,
                                    &auth_header,
                                    channel_id,
                                    &author_id,
                                    content,
                                    &config,
                                )
                                .await;
                            }
                        }
                    }
                }
            }
        }

        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}

async fn handle_discord_message(
    app: tauri::AppHandle,
    client: &reqwest::Client,
    auth_header: &str,
    channel_id: &str,
    author_id: &str,
    content: String,
    config: &DiscordConfig,
) {
    let state = app.state::<AppState>();
    let app_cfg = crate::core::config::AppConfig::load();
    let lang = app_cfg.language.as_str();

    if content.starts_with("!atena") || content.starts_with("/atena") {
        let parts: Vec<&str> = content.split_whitespace().collect();
        let subcmd = parts.get(1).map(|s| s.to_lowercase()).unwrap_or_default();

        match subcmd.as_str() {
            "help" | "" => {
                let msg = crate::services::gateways::i18n::get_help_msg(lang);
                let _ = send_discord_message(client, auth_header, channel_id, msg).await;
                return;
            }
            "id" => {
                let msg = crate::services::gateways::i18n::get_user_id_msg(lang, author_id);
                let _ = send_discord_message(client, auth_header, channel_id, &msg).await;
                return;
            }
            "model" => {
                let active_model = state.active_model.lock().await.clone();
                let model_name = active_model
                    .map(|m| format!("{} ({:?})", m.name, m.backend))
                    .unwrap_or_else(|| crate::services::gateways::i18n::get_no_model_msg(lang).to_string());
                let msg = crate::services::gateways::i18n::get_model_msg(lang, &model_name);
                let _ = send_discord_message(client, auth_header, channel_id, &msg).await;
                return;
            }
            "new" => {
                let session_id = format!("discord_{}", channel_id);
                let now = chrono::Utc::now().to_rfc3339();
                let new_session = crate::services::db::DbChatSession {
                    id: session_id.clone(),
                    title: format!("Discord Channel {}", channel_id),
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
                let _ = send_discord_message(client, auth_header, channel_id, msg).await;
                return;
            }
            _ => {}
        }
    }

    // Chat Inference execution
    let session_id = format!("discord_{}", channel_id);

    // Trigger typing indicator
    let _ = trigger_discord_typing(client, auth_header, channel_id).await;

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
        content: content.clone(),
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
        user_message: Some(content.clone()),
        session_id: Some(session_id.clone()),
        session_title: Some(format!("Discord Channel {}", channel_id)),
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
    let auth_clone = auth_header.to_string();
    let channel_clone = channel_id.to_string();

    let typing_task = tokio::spawn(async move {
        for _ in 0..60 {
            tokio::time::sleep(Duration::from_secs(5)).await;
            let _ = trigger_discord_typing(&client_clone, &auth_clone, &channel_clone).await;
        }
    });

    let result = crate::execute_stream_chat_internal(&state, req, on_chunk).await;
    typing_task.abort();

    if let Err(e) = result {
        let err_msg = format!("⚠️ Inference Error: {}", e);
        let _ = send_discord_message(client, auth_header, channel_id, &err_msg).await;
        return;
    }

    // Check if the assistant requested tool calls to execute
    let tool_calls = captured_tool_calls.lock().unwrap().clone();
    if config.enable_tools && !tool_calls.is_empty() {
        let mut current_tool_calls = tool_calls;
        let mut loop_count = 0;
        while !current_tool_calls.is_empty() && loop_count < 3 {
            loop_count += 1;
            log::info!("[Discord Gateway] Executing {} tool call(s) (loop {})", current_tool_calls.len(), loop_count);
            let _ = trigger_discord_typing(client, auth_header, channel_id).await;

            let mut executed_tools = Vec::new();
            for mut tc in current_tool_calls {
                let server_id = tc.server_id.clone().unwrap_or_else(|| {
                    if tc.name.starts_with("atena_") {
                        "atena_native".to_string()
                    } else {
                        "skills".to_string()
                    }
                });

                log::info!("[Discord Gateway] Invoking tool '{}' on '{}' with args: {:?}", tc.name, server_id, tc.arguments);
                match crate::execute_tool_call_internal(&state, &server_id, &tc.name, tc.arguments.clone()).await {
                    Ok(res) => {
                        log::info!("[Discord Gateway] Tool '{}' succeeded: {:?}", tc.name, res);
                        tc.status = Some("completed".to_string());
                        tc.result = Some(res);
                    }
                    Err(e) => {
                        log::error!("[Discord Gateway] Tool '{}' failed: {}", tc.name, e);
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
                let a_hdr = auth_header.to_string();
                let ch_id = channel_id.to_string();
                tokio::spawn(async move {
                    for _ in 0..60 {
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        let _ = trigger_discord_typing(&cl, &a_hdr, &ch_id).await;
                    }
                })
            };

            let fu_req = crate::StreamChatRequest {
                model: active_model.clone(),
                system_prompt: system_prompt.clone(),
                messages: Some(messages.clone()),
                user_message: None,
                session_id: Some(session_id.clone()),
                session_title: Some(format!("Discord Channel {}", channel_id)),
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
        let _ = send_discord_message(client, auth_header, channel_id, "(Empty response generated)").await;
    } else {
        // Persist user and assistant exchange into SQLite session
        let now = chrono::Utc::now().to_rfc3339();
        let mut sess = state.db.get_session(&session_id).ok().flatten().unwrap_or_else(|| {
            crate::services::db::DbChatSession {
                id: session_id.clone(),
                title: format!("Discord Channel {}", channel_id),
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
            content: content.clone(),
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

        let user_display = crate::services::gateways::strip_internal_tags(clean_text);
        send_chunked_discord_message(client, auth_header, channel_id, &user_display).await;
    }
}

async fn trigger_discord_typing(client: &reqwest::Client, auth_header: &str, channel_id: &str) -> Result<(), reqwest::Error> {
    let url = format!("https://discord.com/api/v10/channels/{}/typing", channel_id);
    let _ = client
        .post(&url)
        .header("Authorization", auth_header)
        .header("User-Agent", "AtenaStudio (https://github.com/voylaboratory/atena, 0.1.0)")
        .send()
        .await?;
    Ok(())
}

async fn send_discord_message(
    client: &reqwest::Client,
    auth_header: &str,
    channel_id: &str,
    text: &str,
) -> Result<(), reqwest::Error> {
    let url = format!("https://discord.com/api/v10/channels/{}/messages", channel_id);
    let _ = client
        .post(&url)
        .header("Authorization", auth_header)
        .header("User-Agent", "AtenaStudio (https://github.com/voylaboratory/atena, 0.1.0)")
        .json(&json!({
            "content": text
        }))
        .send()
        .await?;
    Ok(())
}

async fn send_chunked_discord_message(
    client: &reqwest::Client,
    auth_header: &str,
    channel_id: &str,
    text: &str,
) {
    const CHUNK_LIMIT: usize = 1900;

    if text.len() <= CHUNK_LIMIT {
        let _ = send_discord_message(client, auth_header, channel_id, text).await;
        return;
    }

    let mut remaining = text;
    while !remaining.is_empty() {
        if remaining.len() <= CHUNK_LIMIT {
            let _ = send_discord_message(client, auth_header, channel_id, remaining).await;
            break;
        }

        let split_idx = remaining[..CHUNK_LIMIT]
            .rfind('\n')
            .or_else(|| remaining[..CHUNK_LIMIT].rfind(' '))
            .unwrap_or(CHUNK_LIMIT);

        let chunk = &remaining[..split_idx];
        let _ = send_discord_message(client, auth_header, channel_id, chunk).await;
        remaining = remaining[split_idx..].trim_start();
    }
}
