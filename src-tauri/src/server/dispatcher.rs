use std::collections::HashMap;
use std::sync::Arc;
use serde_json::{json, Value};

use crate::AppState;
use crate::core::mcp::McpServerConfig;
use crate::core::memory::{CompressionType, NodeType, RelationType};
use crate::core::model::ModelInfo;
use crate::services::backend::BackendManager;
use crate::services::mcp_service::McpManager;
use crate::services::memory_engine::MemoryGraphEngine;
use crate::services::plugins::PluginManager;
use crate::services::scanner::ModelScanner;
use crate::services::server_ctl::LocalServerController;
use crate::services::skill_runner::SkillScriptRunner;

/// Dispatches standard unary IPC commands received over HTTP or WebSocket in server mode.
pub async fn dispatch_invoke(
    state: &Arc<AppState>,
    cmd: &str,
    args: Value,
) -> Result<Value, String> {
    match cmd {
        "get_platform_info" => {
            let os = if cfg!(target_os = "macos") {
                "macos"
            } else if cfg!(target_os = "windows") {
                "windows"
            } else {
                "linux"
            };
            Ok(json!({
                "os": os,
                "supports_mlx": cfg!(target_os = "macos"),
                "supports_whisper_local": true,
            }))
        }

        "get_hardware_info" => {
            let mut hw = state.hardware_info.lock().await;
            hw.refresh();
            Ok(serde_json::to_value(&*hw).map_err(|e| e.to_string())?)
        }

        "check_services" => {
            let mlx_host = args.get("mlxHost").or_else(|| args.get("mlx_host")).and_then(|v| v.as_str()).unwrap_or("127.0.0.1");
            let mlx_port = args.get("mlxPort").or_else(|| args.get("mlx_port")).and_then(|v| v.as_u64()).unwrap_or(8080) as u16;
            let ollama_host = args.get("ollamaHost").or_else(|| args.get("ollama_host")).and_then(|v| v.as_str()).unwrap_or("127.0.0.1");
            let ollama_port = args.get("ollamaPort").or_else(|| args.get("ollama_port")).and_then(|v| v.as_u64()).unwrap_or(11434) as u16;

            let mlx_online = BackendManager::check_mlx_health(mlx_host, mlx_port).await;
            let ollama_online = BackendManager::check_ollama_health(ollama_host, ollama_port).await;
            Ok(json!({
                "mlx_online": mlx_online,
                "ollama_online": ollama_online
            }))
        }

        "get_cached_models" => {
            let models = ModelScanner::load_cached_models();
            Ok(serde_json::to_value(models).map_err(|e| e.to_string())?)
        }

        "refresh_model_info" => {
            let model: ModelInfo = serde_json::from_value(args.get("model").cloned().unwrap_or(json!({})))
                .map_err(|e| e.to_string())?;
            let info = ModelScanner::refresh_single_model(&model).await;
            Ok(serde_json::to_value(info).map_err(|e| e.to_string())?)
        }

        "scan_models" => {
            let mut dirs = Vec::new();
            if let Some(ds) = args.get("modelsDirs").and_then(|v| v.as_array()) {
                for d in ds {
                    if let Some(s) = d.as_str() {
                        let trimmed = s.trim().to_string();
                        if !trimmed.is_empty() && !dirs.contains(&trimmed) {
                            dirs.push(trimmed);
                        }
                    }
                }
            }
            if let Some(d) = args.get("modelsDir").and_then(|v| v.as_str()) {
                let trimmed = d.trim().to_string();
                if !trimmed.is_empty() && !dirs.contains(&trimmed) {
                    dirs.push(trimmed);
                }
            }
            if dirs.is_empty() {
                dirs.push(crate::core::config::default_models_directory());
            }
            let ollama_host = args.get("ollamaHost").and_then(|v| v.as_str()).unwrap_or("127.0.0.1");
            let ollama_port = args.get("ollamaPort").and_then(|v| v.as_u64()).unwrap_or(11434) as u16;

            let models = ModelScanner::scan_all_directories(&dirs, ollama_host, ollama_port).await;
            Ok(serde_json::to_value(models).map_err(|e| e.to_string())?)
        }

        "detect_model_directories" => {
            let dirs = ModelScanner::detect_common_model_dirs();
            Ok(serde_json::to_value(dirs).map_err(|e| e.to_string())?)
        }

        "get_default_models_directory" => {
            Ok(json!(crate::core::config::default_models_directory()))
        }

        "set_active_model" => {
            let model = if let Some(m) = args.get("model") {
                if m.is_null() {
                    None
                } else {
                    serde_json::from_value::<ModelInfo>(m.clone()).ok()
                }
            } else {
                None
            };
            *state.active_model.lock().await = model;
            Ok(json!(null))
        }

        "get_active_model" => {
            let active = state.active_model.lock().await.clone();
            Ok(serde_json::to_value(active).map_err(|e| e.to_string())?)
        }

        "start_mlx_server" => {
            let model_path = args.get("modelPath").and_then(|v| v.as_str()).unwrap_or_default();
            let host = args.get("host").and_then(|v| v.as_str()).unwrap_or("127.0.0.1");
            let port = args.get("port").and_then(|v| v.as_u64()).unwrap_or(8080) as u16;
            state.backend_manager.start_mlx_server(model_path, host, port, None).await?;
            Ok(json!({ "status": "ok" }))
        }

        "start_llama_server" => {
            let model_path = args.get("modelPath").and_then(|v| v.as_str()).unwrap_or_default();
            let host = args.get("host").and_then(|v| v.as_str()).unwrap_or("127.0.0.1");
            let port = args.get("port").and_then(|v| v.as_u64()).unwrap_or(8080) as u16;
            let context_length = args.get("contextLength").and_then(|v| v.as_u64()).map(|v| v as usize).unwrap_or(8192);
            let kv_cache_quant = args.get("kvCacheQuant").and_then(|v| v.as_str()).map(String::from);
            let flash_attn = args.get("flashAttn").and_then(|v| v.as_bool());
            let prompt_cache = args.get("promptCache").and_then(|v| v.as_bool());

            state.backend_manager.start_llama_server(
                model_path,
                host,
                port,
                context_length,
                kv_cache_quant,
                flash_attn,
                prompt_cache,
                None,
            ).await?;
            Ok(json!({ "status": "ok" }))
        }

        "stop_all_servers" => {
            state.backend_manager.stop_all().await;
            Ok(json!(null))
        }

        "get_server_logs" => {
            let logs = LocalServerController::get_logs();
            if let Ok(mut state_logs) = state.server_logs.try_lock() {
                *state_logs = logs.clone();
            }
            Ok(serde_json::to_value(logs).map_err(|e| e.to_string())?)
        }

        "clear_server_logs" => {
            LocalServerController::clear_logs();
            if let Ok(mut state_logs) = state.server_logs.try_lock() {
                state_logs.clear();
            }
            Ok(json!(null))
        }

        "get_mcp_servers" => {
            let servers = state.mcp_manager.get_servers().await;
            Ok(serde_json::to_value(servers).map_err(|e| e.to_string())?)
        }

        "save_mcp_servers" => {
            let servers: Vec<McpServerConfig> = serde_json::from_value(args.get("servers").cloned().unwrap_or(json!([])))
                .map_err(|e| e.to_string())?;
            state.mcp_manager.save_servers(servers).await?;
            Ok(json!(null))
        }

        "inspect_server_tools" => {
            let server: McpServerConfig = serde_json::from_value(args.get("server").cloned().unwrap_or(json!({})))
                .map_err(|e| e.to_string())?;
            let tools = McpManager::inspect_server_tools(&server).await?;
            Ok(serde_json::to_value(tools).map_err(|e| e.to_string())?)
        }

        "list_all_mcp_tools" => {
            let tools = state.mcp_manager.list_all_tools().await;
            Ok(serde_json::to_value(tools).map_err(|e| e.to_string())?)
        }

        "call_mcp_tool" => {
            let server_id = args.get("serverId").and_then(|v| v.as_str()).unwrap_or_default();
            let tool_name = args.get("toolName").and_then(|v| v.as_str()).unwrap_or_default();
            let arguments = args.get("arguments").cloned().unwrap_or(json!({}));
            let res = state.mcp_manager.call_tool(server_id, tool_name, arguments).await?;
            Ok(res)
        }

        "update_mcp_tool_label" => {
            let server_id = args.get("serverId").and_then(|v| v.as_str()).unwrap_or_default();
            let tool_name = args.get("toolName").and_then(|v| v.as_str()).unwrap_or_default();
            let label = args.get("label").and_then(|v| v.as_str()).unwrap_or_default();

            let mut servers = state.mcp_manager.get_servers().await;
            if let Some(server) = servers.iter_mut().find(|s| s.id == server_id) {
                if label.trim().is_empty() {
                    server.tool_labels.remove(tool_name);
                } else {
                    server.tool_labels.insert(tool_name.to_string(), label.trim().to_string());
                }
                state.mcp_manager.save_servers(servers).await?;
                Ok(json!(null))
            } else {
                Err(format!("Server {} not found", server_id))
            }
        }

        "update_mcp_tool_details" => {
            let server_id = args.get("serverId").and_then(|v| v.as_str()).unwrap_or_default();
            let tool_name = args.get("toolName").and_then(|v| v.as_str()).unwrap_or_default();
            let label = args.get("label").and_then(|v| v.as_str());
            let field_labels: Option<HashMap<String, String>> = args.get("fieldLabels")
                .and_then(|v| serde_json::from_value(v.clone()).ok());
            let enabled = args.get("enabled").and_then(|v| v.as_bool());

            let mut servers = state.mcp_manager.get_servers().await;
            if let Some(server) = servers.iter_mut().find(|s| s.id == server_id) {
                if let Some(lbl) = label {
                    if lbl.trim().is_empty() {
                        server.tool_labels.remove(tool_name);
                    } else {
                        server.tool_labels.insert(tool_name.to_string(), lbl.trim().to_string());
                    }
                }
                if let Some(fields) = field_labels {
                    if fields.is_empty() {
                        server.tool_field_labels.remove(tool_name);
                    } else {
                        server.tool_field_labels.insert(tool_name.to_string(), fields);
                    }
                }
                if let Some(en) = enabled {
                    if en {
                        server.disabled_tools.retain(|t| t != tool_name);
                    } else if !server.disabled_tools.contains(&tool_name.to_string()) {
                        server.disabled_tools.push(tool_name.to_string());
                    }
                }
                state.mcp_manager.save_servers(servers).await?;
                Ok(json!(null))
            } else {
                Err(format!("Server {} not found", server_id))
            }
        }

        "read_file_attachment" => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or_default();
            let content = tokio::fs::read_to_string(path).await.map_err(|e| e.to_string())?;
            Ok(json!(content))
        }

        "hf_search_models" => {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or_default();
            let filter_type = args.get("filterType").and_then(|v| v.as_str());
            let sort = args.get("sort").and_then(|v| v.as_str());
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(30) as usize;
            let results = state.downloader.search_hf_models(query, filter_type, sort, limit).await?;
            Ok(serde_json::to_value(results).map_err(|e| e.to_string())?)
        }

        "hf_get_model_details" => {
            let repo_id = args.get("repoId").and_then(|v| v.as_str()).unwrap_or_default();
            let detail = state.downloader.get_model_details(repo_id).await?;
            Ok(serde_json::to_value(detail).map_err(|e| e.to_string())?)
        }

        "hf_get_author_avatar" => {
            let author = args.get("author").and_then(|v| v.as_str()).unwrap_or_default();
            let avatar = state.downloader.get_author_avatar(author).await;
            Ok(json!(avatar))
        }

        "hf_get_author_avatars" => {
            let authors: Vec<String> = serde_json::from_value(args.get("authors").cloned().unwrap_or(json!([])))
                .unwrap_or_default();
            let avatars = state.downloader.get_author_avatars(authors).await;
            Ok(serde_json::to_value(avatars).map_err(|e| e.to_string())?)
        }

        "start_hf_download" => {
            let repo_id = args.get("repoId").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let selected_files: Vec<String> = serde_json::from_value(args.get("selectedFiles").cloned().unwrap_or(json!([])))
                .unwrap_or_default();
            let target_dir = args.get("targetDir").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let is_mlx_bundle = args.get("isMlxBundle").and_then(|v| v.as_bool()).unwrap_or(false);
            let model_title = args.get("modelTitle").and_then(|v| v.as_str()).map(String::from);
            let author_avatar_url = args.get("authorAvatarUrl").and_then(|v| v.as_str()).map(String::from);

            let task_id = state.downloader.start_download(
                repo_id,
                selected_files,
                target_dir,
                is_mlx_bundle,
                model_title,
                author_avatar_url,
            ).await?;
            Ok(json!(task_id))
        }

        "check_hf_model_downloaded" => {
            let repo_id = args.get("repoId").and_then(|v| v.as_str()).unwrap_or_default();
            let files: Vec<String> = serde_json::from_value(args.get("files").cloned().unwrap_or(json!([])))
                .unwrap_or_default();
            let models_dirs: Vec<String> = serde_json::from_value(args.get("modelsDirs").cloned().unwrap_or(json!([])))
                .unwrap_or_default();
            let is_mlx_bundle = args.get("isMlxBundle").and_then(|v| v.as_bool()).unwrap_or(false);

            let res = ModelScanner::check_model_exists_on_disk(&models_dirs, repo_id, &files, is_mlx_bundle).await;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        }

        "cancel_hf_download" => {
            let task_id = args.get("taskId").and_then(|v| v.as_str()).unwrap_or_default();
            state.downloader.cancel_download(task_id).await?;
            Ok(json!(null))
        }

        "retry_hf_download" => {
            let task_id = args.get("taskId").and_then(|v| v.as_str()).unwrap_or_default();
            state.downloader.retry_download(task_id).await?;
            Ok(json!(null))
        }

        "dismiss_download_task" => {
            let task_id = args.get("taskId").and_then(|v| v.as_str()).unwrap_or_default();
            state.downloader.dismiss_task(task_id).await?;
            Ok(json!(null))
        }

        "clear_finished_downloads" => {
            state.downloader.clear_finished_tasks().await?;
            Ok(json!(null))
        }

        "get_active_downloads" => {
            let tasks = state.downloader.get_all_tasks().await;
            Ok(serde_json::to_value(tasks).map_err(|e| e.to_string())?)
        }

        "delete_model" => {
            let model_id = args.get("modelId").or_else(|| args.get("model_id")).and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let local_path = args.get("localPath").or_else(|| args.get("local_path")).and_then(|v| v.as_str()).map(String::from);
            let format = args.get("format").and_then(|v| v.as_str()).map(String::from);
            let backend = args.get("backend").and_then(|v| v.as_str()).map(String::from);

            // 1. If currently loaded active model, unload it
            {
                let mut active = state.active_model.lock().await;
                if let Some(ref m) = *active {
                    if m.id == model_id {
                        *active = None;
                    }
                }
            }

            // Protection: Antigravity models are cloud-backed CLI models; do not delete the agy binary!
            if backend.as_deref() == Some("Antigravity") || format.as_deref() == Some("Agy") || model_id.starts_with("agy/") {
                return Ok(json!(null));
            }

            // 2. If it has a local path on disk, remove directory or file
            if let Some(ref path_str) = local_path {
                let resolved = crate::core::config::resolve_path(path_str);
                if resolved.exists() {
                    if resolved.is_dir() {
                        tokio::fs::remove_dir_all(&resolved)
                            .await
                            .map_err(|e| format!("Falha ao excluir diretório do modelo: {}", e))?;
                    } else if resolved.is_file() {
                        let parent = resolved.parent().map(|p| p.to_path_buf());
                        tokio::fs::remove_file(&resolved)
                            .await
                            .map_err(|e| format!("Falha ao excluir arquivo do modelo: {}", e))?;

                        if let Some(p) = parent {
                            if let Ok(mut rd) = tokio::fs::read_dir(&p).await {
                                if rd.next_entry().await.ok().flatten().is_none() {
                                    let _ = tokio::fs::remove_dir(&p).await;
                                }
                            }
                        }
                    }
                }
            }

            // 3. If Ollama model
            let backend_str = backend.unwrap_or_default().to_lowercase();
            let format_str = format.unwrap_or_default().to_lowercase();
            if backend_str == "ollama" || format_str == "ollama" {
                let client = reqwest::Client::new();
                let t_start = std::time::Instant::now();
                let res = client
                    .delete("http://127.0.0.1:11434/api/delete")
                    .json(&serde_json::json!({ "name": model_id }))
                    .send()
                    .await;
                let latency = t_start.elapsed().as_millis() as u64;
                let status = res.map(|r| r.status().as_u16()).unwrap_or(502);
                LocalServerController::record_log(
                    "DELETE",
                    "http://127.0.0.1:11434/api/delete",
                    status,
                    latency,
                    0,
                    0,
                );
            }

            Ok(json!(null))
        }

        "check_cli_installed" => {
            let status = crate::check_cli_installed_internal().await?;
            Ok(serde_json::to_value(status).map_err(|e| e.to_string())?)
        }

        "install_cli_command" => {
            let res = crate::install_cli_command_internal().await?;
            Ok(json!(res))
        }

        "open_url" => {
            // Client browser opens URLs directly
            Ok(json!(null))
        }

        "open_folder" => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or_default();
            #[cfg(target_os = "macos")]
            let _ = std::process::Command::new("open").arg(path).spawn();
            #[cfg(target_os = "windows")]
            let _ = std::process::Command::new("explorer").arg(path).spawn();
            #[cfg(target_os = "linux")]
            let _ = std::process::Command::new("xdg-open").arg(path).spawn();
            Ok(json!(null))
        }

        "select_folder" => {
            let default_path = args.get("defaultPath").and_then(|v| v.as_str()).map(String::from);
            Ok(json!(default_path.unwrap_or_else(crate::core::config::default_models_directory)))
        }

        "save_file_content" => {
            let content = args.get("content").and_then(|v| v.as_str()).unwrap_or_default();
            let default_name = args.get("defaultName").and_then(|v| v.as_str()).unwrap_or("export.txt");
            let default_path = args.get("defaultPath").and_then(|v| v.as_str());
            let target_path = if let Some(p) = default_path {
                std::path::Path::new(p).join(default_name)
            } else {
                std::path::PathBuf::from(default_name)
            };
            tokio::fs::write(&target_path, content.as_bytes()).await.map_err(|e| e.to_string())?;
            Ok(json!(target_path.to_string_lossy()))
        }

        "check_runtimes_status" | "get_runtime_status" => {
            let status = state.runtime_manager.get_status().await;
            Ok(serde_json::to_value(status).map_err(|e| e.to_string())?)
        }

        // Memory Graph Commands
        "memory_add_node" => {
            let type_flag = args.get("typeFlag").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let label = args.get("label").and_then(|v| v.as_str()).unwrap_or_default();
            let node_type = NodeType::from_byte(type_flag)
                .ok_or_else(|| format!("Invalid node type flag: 0x{:02X}", type_flag))?;
            let mut engine = state.memory_engine.lock().await;
            let id = engine.add_node(node_type, label);
            Ok(json!(id))
        }

        "memory_add_edge" => {
            let source_id = args.get("sourceId").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let target_id = args.get("targetId").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let relation_type = args.get("relationType").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
            let rel = RelationType::from_byte(relation_type)
                .ok_or_else(|| format!("Invalid relation type flag: 0x{:02X}", relation_type))?;
            let mut engine = state.memory_engine.lock().await;
            engine.add_edge(source_id, target_id, rel)?;
            Ok(json!(null))
        }

        "memory_query" => {
            let start_label = args.get("startLabel").and_then(|v| v.as_str()).unwrap_or_default();
            let max_depth = args.get("maxDepth").and_then(|v| v.as_u64()).unwrap_or(3) as usize;
            let mut engine = state.memory_engine.lock().await;
            let paths = engine.traverse_associations(start_label, max_depth)?;
            let llm_context = MemoryGraphEngine::build_llm_context(&paths);
            let stats = engine.stats();
            Ok(json!({
                "llm_context": llm_context,
                "paths": paths,
                "stats": stats,
            }))
        }

        "memory_reinforce" => {
            let source_id = args.get("sourceId").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let target_id = args.get("targetId").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let mut engine = state.memory_engine.lock().await;
            engine.reinforce_edge(source_id, target_id)?;
            Ok(json!(null))
        }

        "memory_apply_decay" => {
            let decay_factor = args.get("decayFactor").and_then(|v| v.as_f64()).map(|v| v as f32);
            let mut engine = state.memory_engine.lock().await;
            engine.apply_decay(decay_factor.unwrap_or(0.05));
            Ok(json!(null))
        }

        "memory_save" => {
            let filepath = args.get("filepath").and_then(|v| v.as_str()).unwrap_or_default();
            let comp = args.get("compression")
                .and_then(|v| v.as_u64())
                .map(|v| v as u8)
                .and_then(CompressionType::from_byte)
                .unwrap_or(CompressionType::Zstd);
            let engine = state.memory_engine.lock().await;
            let bytes = engine.save_to_compressed_binary(std::path::Path::new(filepath), comp)?;
            Ok(json!(bytes))
        }

        "memory_load" => {
            let filepath = args.get("filepath").and_then(|v| v.as_str()).unwrap_or_default();
            let new_engine = MemoryGraphEngine::load_from_compressed_binary(std::path::Path::new(filepath), 128)?;
            let stats = new_engine.stats();
            let mut engine = state.memory_engine.lock().await;
            *engine = new_engine;
            Ok(serde_json::to_value(stats).map_err(|e| e.to_string())?)
        }

        "memory_stats" => {
            let engine = state.memory_engine.lock().await;
            let stats = engine.stats();
            Ok(serde_json::to_value(stats).map_err(|e| e.to_string())?)
        }

        "memory_get_full_graph" => {
            let engine = state.memory_engine.lock().await;
            let graph = engine.get_full_graph();
            Ok(serde_json::to_value(graph).map_err(|e| e.to_string())?)
        }

        "memory_learn_text" => {
            let text = args.get("text").and_then(|v| v.as_str()).unwrap_or_default();
            let session_id = args.get("sessionId").and_then(|v| v.as_str());
            let mut engine = state.memory_engine.lock().await;
            let learned = engine.learn_from_text_with_session(text, session_id);
            Ok(serde_json::to_value(learned).map_err(|e| e.to_string())?)
        }

        "memory_update_node_scope" => {
            let id = args.get("id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let session_id = args.get("sessionId").and_then(|v| v.as_str()).map(String::from);
            let mut engine = state.memory_engine.lock().await;
            engine.update_node_scope(id, session_id)?;
            Ok(json!(null))
        }

        "memory_delete_node" => {
            let id = args.get("id").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
            let mut engine = state.memory_engine.lock().await;
            engine.delete_node(id)?;
            Ok(json!(null))
        }

        "memory_clear_all" => {
            let mut engine = state.memory_engine.lock().await;
            engine.clear_all();
            Ok(json!(null))
        }

        "memory_record_vigilia_event" => {
            let source = args.get("source").and_then(|v| v.as_str()).unwrap_or_default();
            let text = args.get("text").and_then(|v| v.as_str()).unwrap_or_default();
            let category = args.get("category").and_then(|v| v.as_str());
            MemoryGraphEngine::record_vigilia_event(source, text, None, category)?;
            Ok(json!(null))
        }

        "memory_get_vigilia_buffer" => {
            let events = MemoryGraphEngine::load_vigilia_buffer();
            Ok(serde_json::to_value(events).map_err(|e| e.to_string())?)
        }

        "memory_trigger_sleep_cycle" => {
            let mut engine = state.memory_engine.lock().await;
            let report = engine.run_sleep_cycle();
            Ok(serde_json::to_value(report).map_err(|e| e.to_string())?)
        }

        "memory_trigger_deep_sleep_cycle" => {
            let active_model = {
                let lock = state.active_model.lock().await;
                lock.clone()
            };
            let events = MemoryGraphEngine::load_vigilia_buffer();
            if events.is_empty() || active_model.is_none() {
                let mut engine = state.memory_engine.lock().await;
                let report = engine.run_sleep_cycle();
                return Ok(serde_json::to_value(report).map_err(|e| e.to_string())?);
            }
            let mut engine = state.memory_engine.lock().await;
            let report = engine.run_sleep_cycle();
            Ok(serde_json::to_value(report).map_err(|e| e.to_string())?)
        }

        "memory_optimize_and_prune" => {
            let mut engine = state.memory_engine.lock().await;
            let report = engine.optimize_and_prune_graph();
            Ok(serde_json::to_value(report).map_err(|e| e.to_string())?)
        }

        "memory_clear_vigilia_buffer" => {
            MemoryGraphEngine::clear_vigilia_buffer()?;
            Ok(json!(null))
        }

        // Skills & Automations
        "skills_get_all" => {
            let skills = MemoryGraphEngine::load_skills();
            Ok(serde_json::to_value(skills).map_err(|e| e.to_string())?)
        }

        "skills_get_folder" => {
            let dir = MemoryGraphEngine::skills_root_dir();
            Ok(json!(dir.to_string_lossy().to_string()))
        }

        "skills_open_folder" => {
            let skill_id = args.get("skillId").and_then(|v| v.as_str()).map(String::from);
            let root = MemoryGraphEngine::skills_root_dir();
            let target_dir = if let Some(id) = skill_id {
                let skills = MemoryGraphEngine::load_skills();
                if let Some(s) = skills.iter().find(|item| item.id == id) {
                    if let Some(ref folder) = s.folder_path {
                        std::path::PathBuf::from(folder)
                    } else {
                        root.join(MemoryGraphEngine::skill_slug(&s.name))
                    }
                } else {
                    root
                }
            } else {
                root
            };

            let _ = std::fs::create_dir_all(&target_dir);
            #[cfg(target_os = "macos")]
            let _ = std::process::Command::new("open").arg(&target_dir).spawn();
            #[cfg(target_os = "windows")]
            let _ = std::process::Command::new("explorer").arg(&target_dir).spawn();
            #[cfg(target_os = "linux")]
            let _ = std::process::Command::new("xdg-open").arg(&target_dir).spawn();
            Ok(json!(null))
        }

        "skills_list_scripts" => {
            let skill_id = args.get("skillId").and_then(|v| v.as_str()).unwrap_or_default();
            let skills = MemoryGraphEngine::load_skills();
            if let Some(s) = skills.iter().find(|item| item.id == skill_id) {
                Ok(serde_json::to_value(&s.scripts).map_err(|e| e.to_string())?)
            } else {
                Ok(json!([]))
            }
        }

        "skills_run_command" => {
            let skill_id = args.get("skillId").and_then(|v| v.as_str()).map(String::from);
            let command = args.get("command").and_then(|v| v.as_str()).unwrap_or_default();
            let cwd = args.get("cwd").and_then(|v| v.as_str()).map(std::path::PathBuf::from);
            let timeout_ms = args.get("timeoutMs").and_then(|v| v.as_u64());

            let skill_dir = if let Some(id) = &skill_id {
                let skills = MemoryGraphEngine::load_skills();
                skills
                    .into_iter()
                    .find(|s| s.id == *id)
                    .and_then(|s| s.folder_path.map(std::path::PathBuf::from))
            } else {
                None
            };

            let res = SkillScriptRunner::run_command(
                command,
                skill_dir.as_deref(),
                cwd.as_deref(),
                None,
                timeout_ms,
            ).await?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        }

        "skills_run_script" => {
            let skill_id = args.get("skillId")
                .or_else(|| args.get("skill_id"))
                .or_else(|| args.get("slug"))
                .or_else(|| args.get("id"))
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let script_name = args.get("scriptName")
                .or_else(|| args.get("script_name"))
                .or_else(|| args.get("scriptFile"))
                .or_else(|| args.get("script_file"))
                .or_else(|| args.get("script"))
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let script_args: Vec<String> = serde_json::from_value(args.get("args").cloned().unwrap_or(json!([])))
                .unwrap_or_default();
            let cwd = args.get("cwd").and_then(|v| v.as_str()).map(std::path::PathBuf::from);
            let timeout_ms = args.get("timeoutMs").or_else(|| args.get("timeout_ms")).and_then(|v| v.as_u64());

            let skills = MemoryGraphEngine::load_skills();
            let skill = skills
                .into_iter()
                .find(|s| {
                    s.id == skill_id
                        || s.id == format!("skill-{}", skill_id)
                        || skill_id == format!("skill-{}", s.id)
                        || s.name.eq_ignore_ascii_case(skill_id)
                        || s.id.trim_start_matches("skill-") == skill_id.trim_start_matches("skill-")
                })
                .ok_or_else(|| format!("Skill '{}' not found", skill_id))?;

            let folder = skill
                .folder_path
                .ok_or_else(|| format!("Skill '{}' has no directory configured", skill_id))?;

            let skill_dir = std::path::PathBuf::from(&folder);
            let clean_script = script_name.trim_start_matches('/');
            let script_rel_path = if clean_script.starts_with("scripts/") {
                clean_script.to_string()
            } else {
                format!("scripts/{}", clean_script)
            };

            let res = SkillScriptRunner::run_script(
                &skill_dir,
                &script_rel_path,
                &script_args,
                cwd.as_deref(),
                skill.env_vars.as_ref(),
                timeout_ms,
            ).await?;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        }

        "skills_save_manual" => {
            let id = args.get("id").and_then(|v| v.as_str()).map(String::from);
            let name = args.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let description = args.get("description").and_then(|v| v.as_str()).unwrap_or_default().to_string();
            let triggers: Vec<String> = serde_json::from_value(args.get("triggers").cloned().unwrap_or(json!([])))
                .unwrap_or_default();
            let steps: Vec<String> = serde_json::from_value(args.get("steps").cloned().unwrap_or(json!([])))
                .unwrap_or_default();
            let steps_detailed: Option<Vec<crate::core::memory::SkillStep>> = serde_json::from_value(args.get("stepsDetailed").cloned().unwrap_or(json!(null))).ok();
            let refinement_note = args.get("refinementNote").and_then(|v| v.as_str()).map(String::from);
            let env_vars: Option<HashMap<String, String>> = serde_json::from_value(args.get("envVars").cloned().unwrap_or(json!(null))).ok();
            let permission_mode = args.get("permissionMode").and_then(|v| v.as_str()).map(String::from);

            let step_structs = if let Some(detailed) = steps_detailed {
                detailed
            } else {
                let mut s_list = Vec::new();
                for (i, s) in steps.into_iter().enumerate() {
                    let clean = s.trim().to_string();
                    if !clean.is_empty() {
                        s_list.push(crate::core::memory::SkillStep {
                            order: (i + 1) as u32,
                            instruction: clean,
                            tool_name: None,
                            command: None,
                            script_file: None,
                            cwd: None,
                            timeout_ms: None,
                        });
                    }
                }
                s_list
            };

            let skill = MemoryGraphEngine::learn_or_refine_skill_advanced(
                id,
                &name,
                &description,
                triggers,
                step_structs,
                refinement_note,
                env_vars,
                permission_mode,
            );
            Ok(serde_json::to_value(skill).map_err(|e| e.to_string())?)
        }

        "skills_create_with_scripts" => {
            let name = args.get("name").and_then(|v| v.as_str()).unwrap_or_default();
            let description = args.get("description").and_then(|v| v.as_str()).unwrap_or_default();
            let triggers: Vec<String> = serde_json::from_value(args.get("triggers").cloned().unwrap_or(json!([])))
                .unwrap_or_default();
            let steps = serde_json::from_value(args.get("steps").cloned().unwrap_or(json!([])))
                .unwrap_or_default();
            let scripts = serde_json::from_value(args.get("scripts").cloned().unwrap_or(json!([])))
                .unwrap_or_default();
            let env_vars: Option<HashMap<String, String>> = serde_json::from_value(args.get("envVars").or_else(|| args.get("env_vars")).cloned().unwrap_or(json!(null))).ok();
            let skill = MemoryGraphEngine::create_skill_with_scripts(name, description, triggers, steps, scripts, env_vars)?;
            Ok(serde_json::to_value(skill).map_err(|e| e.to_string())?)
        }

        "skills_update_with_scripts" => {
            let id = args.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
            let name = args.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
            let description = args.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
            let triggers: Option<Vec<String>> = serde_json::from_value(args.get("triggers").cloned().unwrap_or(json!(null))).ok();
            let steps = serde_json::from_value(args.get("steps").cloned().unwrap_or(json!(null))).ok();
            let scripts = serde_json::from_value(args.get("scripts").cloned().unwrap_or(json!(null))).ok();
            let refinement_note = args.get("refinementNote").or_else(|| args.get("refinement_note")).and_then(|v| v.as_str()).map(|s| s.to_string());
            let env_vars: Option<HashMap<String, String>> = serde_json::from_value(args.get("envVars").or_else(|| args.get("env_vars")).cloned().unwrap_or(json!(null))).ok();
            let permission_mode = args.get("permissionMode").or_else(|| args.get("permission_mode")).and_then(|v| v.as_str()).map(|s| s.to_string());
            let skill = MemoryGraphEngine::update_skill_with_scripts(id, name, description, triggers, steps, scripts, refinement_note, env_vars, permission_mode)?;
            Ok(serde_json::to_value(skill).map_err(|e| e.to_string())?)
        }

        "skills_delete" => {
            let skill_id = args.get("skillId").and_then(|v| v.as_str()).unwrap_or_default();
            MemoryGraphEngine::delete_skill(skill_id)?;
            Ok(json!(null))
        }

        "skills_set_permission_mode" => {
            let skill_id = args.get("skillId").and_then(|v| v.as_str()).unwrap_or_default();
            let mode = args.get("mode").and_then(|v| v.as_str()).unwrap_or_default();
            let mut skills = MemoryGraphEngine::load_skills();
            if let Some(skill) = skills.iter_mut().find(|s| s.id == skill_id) {
                skill.permission_mode = mode.to_string();
                MemoryGraphEngine::save_skills(&skills)?;
                Ok(json!(null))
            } else {
                Err(format!("Skill '{}' not found", skill_id))
            }
        }

        // Episodes
        "episodes_get_all" => {
            let episodes = MemoryGraphEngine::get_all_episodes();
            Ok(serde_json::to_value(episodes).map_err(|e| e.to_string())?)
        }

        "episodes_search" => {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or_default();
            let session_id = args.get("sessionId").and_then(|v| v.as_str());
            let limit = args.get("limit").and_then(|v| v.as_u64()).map(|v| v as usize);
            let episodes = MemoryGraphEngine::search_episodes(query, session_id, limit.unwrap_or(10));
            Ok(serde_json::to_value(episodes).map_err(|e| e.to_string())?)
        }

        "episodes_get_detail" => {
            let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
            let detail = MemoryGraphEngine::get_episode_detail(id);
            Ok(serde_json::to_value(detail).map_err(|e| e.to_string())?)
        }

        "session_memory_stats" => {
            let session_id = args.get("sessionId").and_then(|v| v.as_str()).unwrap_or_default();
            let (ep_count, mem_count) = MemoryGraphEngine::count_session_episodes_and_memories(session_id);
            Ok(json!({
                "session_id": session_id,
                "episodes_count": ep_count,
                "memories_count": mem_count,
            }))
        }

        "session_delete_memories" => {
            let session_id = args.get("sessionId").and_then(|v| v.as_str()).unwrap_or_default();
            let delete_episodes = args.get("deleteEpisodes").and_then(|v| v.as_bool()).unwrap_or(false);
            let delete_memories = args.get("deleteMemories").and_then(|v| v.as_bool()).unwrap_or(false);
            let mut engine = state.memory_engine.lock().await;
            let (deleted_eps, deleted_mems) = engine.delete_session_episodes_and_memories(
                session_id,
                delete_episodes,
                delete_memories,
            )?;
            Ok(json!({
                "session_id": session_id,
                "deleted_episodes": deleted_eps,
                "deleted_memories": deleted_mems,
            }))
        }

        "episodes_open_folder" => {
            let dir = MemoryGraphEngine::episodios_dir();
            let _ = std::fs::create_dir_all(&dir);
            #[cfg(target_os = "macos")]
            let _ = std::process::Command::new("open").arg(&dir).spawn();
            #[cfg(target_os = "windows")]
            let _ = std::process::Command::new("explorer").arg(&dir).spawn();
            #[cfg(target_os = "linux")]
            let _ = std::process::Command::new("xdg-open").arg(&dir).spawn();
            Ok(json!(null))
        }

        // Configuration & Cloud Providers
        "check_agy_session" => {
            let res = BackendManager::check_agy_session().await;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        }

        "start_agy_login" => {
            BackendManager::launch_agy_login_terminal()?;
            Ok(json!(null))
        }

        "launch_terminal_command" => {
            let command = args.get("command").and_then(|v| v.as_str()).unwrap_or_default();
            BackendManager::launch_terminal_command(command)?;
            Ok(json!(null))
        }

        "get_agy_usage" => {
            let res = BackendManager::get_agy_usage().await;
            Ok(serde_json::to_value(res).map_err(|e| e.to_string())?)
        }

        "test_cloud_provider_connection" => {
            let provider = args.get("provider").and_then(|v| v.as_str()).unwrap_or_default();
            let api_key = args.get("apiKey").and_then(|v| v.as_str()).unwrap_or_default();
            let base_url = args.get("baseUrl").and_then(|v| v.as_str());
            let res = BackendManager::test_cloud_provider_connection(provider, api_key, base_url).await?;
            Ok(json!(res))
        }

        "fetch_provider_models" => {
            let provider = args.get("provider").and_then(|v| v.as_str()).unwrap_or_default();
            let api_key = args.get("apiKey").and_then(|v| v.as_str());
            let base_url = args.get("baseUrl").and_then(|v| v.as_str());
            let models = ModelScanner::fetch_provider_models_direct(provider, api_key, base_url).await?;
            Ok(serde_json::to_value(models).map_err(|e| e.to_string())?)
        }

        "get_app_config" => {
            let cfg = crate::core::config::AppConfig::load();
            Ok(serde_json::to_value(cfg).map_err(|e| e.to_string())?)
        }

        "save_app_config" => {
            let mut current = crate::core::config::AppConfig::load();
            if let Some(config_val) = args.get("config") {
                if let Ok(parsed) = serde_json::from_value::<crate::core::config::AppConfig>(config_val.clone()) {
                    parsed.save()?;
                    return Ok(json!(null));
                }
                if let Some(cp) = config_val.get("cloud_providers") {
                    if let Ok(parsed_cp) = serde_json::from_value::<crate::core::config::CloudProvidersConfig>(cp.clone()) {
                        current.cloud_providers = parsed_cp;
                    }
                }
                if let Some(mem) = config_val.get("enable_cognitive_memory").and_then(|v| v.as_bool()) {
                    current.enable_cognitive_memory = mem;
                }
                if let Some(mem) = config_val.get("enable_facts_memory").and_then(|v| v.as_bool()) {
                    current.enable_facts_memory = mem;
                }
                if let Some(mem) = config_val.get("enable_skills_memory").and_then(|v| v.as_bool()) {
                    current.enable_skills_memory = mem;
                }
                if let Some(mem) = config_val.get("enable_episodic_memory").and_then(|v| v.as_bool()) {
                    current.enable_episodic_memory = mem;
                }
            }
            current.save()?;
            Ok(json!(null))
        }

        "get_cloud_providers_config" => {
            let cfg = crate::core::config::AppConfig::load().cloud_providers;
            Ok(serde_json::to_value(cfg).map_err(|e| e.to_string())?)
        }

        "save_cloud_providers_config" => {
            let cp_val = args.get("cloudProviders").cloned().unwrap_or(json!({}));
            let mut current = crate::core::config::AppConfig::load();
            if let Ok(parsed) = serde_json::from_value::<crate::core::config::CloudProvidersConfig>(cp_val) {
                current.cloud_providers = parsed;
                current.save()?;
            }
            Ok(json!(null))
        }

        // Plugins
        "list_plugins" => {
            let plugins = PluginManager::list_all_plugins();
            Ok(serde_json::to_value(plugins).map_err(|e| e.to_string())?)
        }

        "toggle_plugin" => {
            let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
            let enabled = args.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
            PluginManager::toggle_plugin(id, enabled)?;
            Ok(json!(null))
        }

        "get_plugins_folder" => {
            let folder = PluginManager::get_plugins_directory().to_string_lossy().to_string();
            Ok(json!(folder))
        }

        "open_plugins_folder" => {
            let folder = PluginManager::get_plugins_directory();
            #[cfg(target_os = "macos")]
            let _ = std::process::Command::new("open").arg(&folder).spawn();
            #[cfg(target_os = "windows")]
            let _ = std::process::Command::new("explorer").arg(&folder.to_string_lossy().to_string()).spawn();
            #[cfg(target_os = "linux")]
            let _ = std::process::Command::new("xdg-open").arg(&folder).spawn();
            Ok(json!(null))
        }

        "get_plugin_settings" => {
            let plugin_id = args.get("pluginId").and_then(|v| v.as_str()).unwrap_or_default();
            let settings = PluginManager::get_plugin_settings(plugin_id);
            Ok(serde_json::to_value(settings).map_err(|e| e.to_string())?)
        }

        "save_plugin_settings" => {
            let plugin_id = args.get("pluginId").and_then(|v| v.as_str()).unwrap_or_default();
            let settings: std::collections::HashMap<String, Value> = serde_json::from_value(args.get("settings").cloned().unwrap_or(json!({})))
                .unwrap_or_default();
            PluginManager::save_plugin_settings(plugin_id, settings)?;
            Ok(json!(null))
        }

        "run_plugin_native" => {
            let plugin_id = args.get("pluginId").and_then(|v| v.as_str()).unwrap_or_default();
            let command = args.get("command").and_then(|v| v.as_str()).unwrap_or_default();
            let payload = args.get("payload").cloned().unwrap_or(json!({}));
            let res = PluginManager::run_plugin_native(plugin_id, command, payload)?;
            Ok(res)
        }

        "read_plugin_script" => {
            let plugin_id = args.get("pluginId").and_then(|v| v.as_str()).unwrap_or_default();
            let script = PluginManager::read_plugin_script(plugin_id)?;
            Ok(json!(script))
        }

        "db_get_sessions" => {
            let sessions = state.db.get_sessions()?;
            Ok(serde_json::to_value(sessions).map_err(|e| e.to_string())?)
        }

        "db_get_session" => {
            let session_id = args.get("sessionId").or_else(|| args.get("session_id")).and_then(|v| v.as_str()).unwrap_or_default();
            let session = state.db.get_session(session_id)?;
            Ok(serde_json::to_value(session).map_err(|e| e.to_string())?)
        }

        "db_save_session" => {
            let session_val = args.get("session").cloned().unwrap_or(args.clone());
            let session: crate::services::db::DbChatSession = serde_json::from_value(session_val)
                .map_err(|e| format!("Invalid session payload: {}", e))?;
            state.db.save_session(&session)?;
            Ok(json!(null))
        }

        "db_save_sessions_batch" => {
            let sessions_val = args.get("sessions").cloned().unwrap_or(args.clone());
            let sessions: Vec<crate::services::db::DbChatSession> = serde_json::from_value(sessions_val)
                .map_err(|e| format!("Invalid sessions payload: {}", e))?;
            state.db.save_sessions_batch(&sessions)?;
            Ok(json!(null))
        }

        "db_delete_session" => {
            let session_id = args.get("sessionId").or_else(|| args.get("session_id")).and_then(|v| v.as_str()).unwrap_or_default();
            state.db.delete_session(session_id)?;
            Ok(json!(null))
        }

        "db_delete_sessions_batch" => {
            let session_ids: Vec<String> = if let Some(arr) = args.get("sessionIds").or_else(|| args.get("session_ids")).and_then(|v| v.as_array()) {
                arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
            } else if let Some(arr) = args.as_array() {
                arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
            } else {
                Vec::new()
            };
            state.db.delete_sessions_batch(&session_ids)?;
            Ok(json!(null))
        }

        "db_get_personas_data" => {
            let data = state.db.get_personas_data()?;
            Ok(serde_json::to_value(data).map_err(|e| e.to_string())?)
        }

        "db_save_persona" => {
            let persona_val = args.get("persona").cloned().unwrap_or(args.clone());
            let persona: crate::services::db::DbPersona = serde_json::from_value(persona_val)
                .map_err(|e| format!("Invalid persona payload: {}", e))?;
            state.db.save_persona(&persona)?;
            Ok(json!(null))
        }

        "db_delete_persona" => {
            let persona_id = args.get("personaId").or_else(|| args.get("persona_id")).and_then(|v| v.as_str()).unwrap_or_default();
            state.db.delete_persona(persona_id)?;
            Ok(json!(null))
        }

        "db_save_persona_override" => {
            let persona_id = args.get("personaId").or_else(|| args.get("persona_id")).and_then(|v| v.as_str()).unwrap_or_default();
            let prompt = args.get("systemPrompt").or_else(|| args.get("system_prompt")).and_then(|v| v.as_str()).unwrap_or_default();
            state.db.save_persona_override(persona_id, prompt)?;
            Ok(json!(null))
        }

        "db_reset_persona_override" => {
            let persona_id = args.get("personaId").or_else(|| args.get("persona_id")).and_then(|v| v.as_str()).unwrap_or_default();
            state.db.reset_persona_override(persona_id)?;
            Ok(json!(null))
        }

        "db_get_setting" => {
            let key = args.get("key").and_then(|v| v.as_str()).unwrap_or_default();
            let val = state.db.get_setting(key)?;
            Ok(json!(val))
        }

        "db_set_setting" => {
            let key = args.get("key").and_then(|v| v.as_str()).unwrap_or_default();
            let val = args.get("value").and_then(|v| v.as_str()).unwrap_or_default();
            state.db.set_setting(key, val)?;
            Ok(json!(null))
        }

        "db_get_all_settings" => {
            let map = state.db.get_all_settings()?;
            Ok(serde_json::to_value(map).map_err(|e| e.to_string())?)
        }

        _ => Err(format!("Unknown command in Atena server dispatcher: {}", cmd)),
    }
}
