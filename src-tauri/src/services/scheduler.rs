use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

use crate::services::db::DbScheduledTask;
use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTaskExecutionResult {
    pub task_id: String,
    pub task_name: String,
    pub success: bool,
    pub output: String,
    pub executed_at: String,
}

pub struct BackgroundScheduler {
    is_running: Arc<AtomicBool>,
}

impl BackgroundScheduler {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Starts the background scheduler daemon with zero CPU overhead when idle
    pub fn start(&self, app_handle: AppHandle) {
        if self.is_running.swap(true, Ordering::SeqCst) {
            log::info!("[Scheduler] Background scheduler is already running.");
            return;
        }

        let running_flag = self.is_running.clone();
        log::info!("[Scheduler] Starting proactive background scheduler daemon.");

        tauri::async_runtime::spawn(async move {
            if let Some(state) = app_handle.try_state::<AppState>() {
                Self::refresh_all_task_schedules(&state);
            }
            // Heartbeat loop checks every 30 seconds
            while running_flag.load(Ordering::Relaxed) {
                tokio::time::sleep(Duration::from_secs(30)).await;

                if !running_flag.load(Ordering::Relaxed) {
                    break;
                }

                if let Some(state) = app_handle.try_state::<AppState>() {
                    let now = Utc::now();
                    let tasks = match state.db.get_scheduled_tasks() {
                        Ok(t) => t,
                        Err(err) => {
                            log::warn!("[Scheduler] Failed to fetch scheduled tasks: {}", err);
                            continue;
                        }
                    };

                    for task in tasks {
                        if !task.enabled {
                            continue;
                        }

                        if Self::is_task_due(&task, now) {
                            log::info!("[Scheduler] Executing scheduled task '{}' ({})", task.name, task.id);
                            let h = app_handle.clone();
                            let t = task.clone();
                            tauri::async_runtime::spawn(async move {
                                Self::execute_task(h, t).await;
                            });
                        }
                    }
                }
            }
            log::info!("[Scheduler] Background scheduler daemon stopped.");
        });
    }

    /// Stops the background scheduler daemon
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    /// Determines if a scheduled task is due for execution
    pub fn is_task_due(task: &DbScheduledTask, now: DateTime<Utc>) -> bool {
        // If next_run is explicitly set and now >= next_run, it's due
        if let Some(next_str) = &task.next_run {
            if let Ok(next_dt) = DateTime::parse_from_rfc3339(next_str) {
                return now >= next_dt.with_timezone(&Utc);
            }
        }

        // If last_run is not set, compute based on creation
        let last_time = task.last_run.as_ref().and_then(|s| DateTime::parse_from_rfc3339(s).ok()).map(|dt| dt.with_timezone(&Utc));

        match Self::compute_next_run(&task.cron_expr, last_time.unwrap_or(now)) {
            Some(computed_next) => now >= computed_next,
            None => false,
        }
    }

    /// Resolves the configured timezone from AppConfig (defaulting to America/Sao_Paulo / UTC-3)
    pub fn resolve_configured_timezone() -> chrono_tz::Tz {
        let app_cfg = crate::core::config::AppConfig::load();
        let tz_str = app_cfg.timezone.trim();
        if let Ok(tz) = tz_str.parse::<chrono_tz::Tz>() {
            return tz;
        }

        let upper = tz_str.to_uppercase();
        if upper == "UTC-3" || upper == "GMT-3" || upper == "BRT" || upper == "-03:00" || upper == "-0300" {
            return chrono_tz::America::Sao_Paulo;
        }
        if upper == "UTC-4" || upper == "GMT-4" || upper == "AMT" || upper == "-04:00" {
            return chrono_tz::America::Manaus;
        }
        if upper == "UTC" || upper == "GMT" || upper == "Z" || upper == "+00:00" {
            return chrono_tz::UTC;
        }

        chrono_tz::America::Sao_Paulo
    }

    /// Refreshes next_run for all tasks using the currently configured timezone
    pub fn refresh_all_task_schedules(state: &AppState) {
        let now = Utc::now();
        if let Ok(tasks) = state.db.get_scheduled_tasks() {
            for task in tasks {
                let reference_time = task.last_run
                    .as_ref()
                    .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or(now);

                let computed = match Self::compute_next_run(&task.cron_expr, reference_time) {
                    Some(dt) if dt <= now => Self::compute_next_run(&task.cron_expr, now),
                    other => other,
                };
                let computed_str = computed.map(|dt| dt.to_rfc3339());
                let _ = state.db.update_task_run_timestamps(
                    &task.id,
                    task.last_run.as_deref().unwrap_or(""),
                    computed_str.as_deref(),
                    task.last_result.as_deref(),
                );
            }
        }
    }

    /// Computes next execution timestamp based on cron expression or interval using configured timezone
    pub fn compute_next_run(expr: &str, reference_time: DateTime<Utc>) -> Option<DateTime<Utc>> {
        let tz = Self::resolve_configured_timezone();
        Self::compute_next_run_with_tz(expr, reference_time, tz)
    }

    /// Computes next execution timestamp for a given timezone
    pub fn compute_next_run_with_tz(expr: &str, reference_time: DateTime<Utc>, tz: chrono_tz::Tz) -> Option<DateTime<Utc>> {
        let clean = expr.trim().to_lowercase();

        // 1. Explicit interval in seconds e.g. "interval:3600" or "@every 1h"
        if let Some(stripped) = clean.strip_prefix("interval:") {
            if let Ok(secs) = stripped.parse::<u64>() {
                return Some(reference_time + chrono::Duration::seconds(secs as i64));
            }
        }

        if clean == "@hourly" || clean == "@every 1h" {
            return Some(reference_time + chrono::Duration::hours(1));
        }
        if clean == "@daily" {
            return Self::next_cron_match_with_tz(&["0", "0", "*", "*", "*"], reference_time, tz);
        }
        if clean == "@every 24h" {
            return Some(reference_time + chrono::Duration::days(1));
        }
        if clean == "@weekly" {
            return Self::next_cron_match_with_tz(&["0", "0", "*", "*", "0"], reference_time, tz);
        }
        if let Some(mins_str) = clean.strip_prefix("@every ") {
            if let Some(val) = mins_str.strip_suffix('m').and_then(|v| v.parse::<i64>().ok()) {
                return Some(reference_time + chrono::Duration::minutes(val));
            }
            if let Some(val) = mins_str.strip_suffix('h').and_then(|v| v.parse::<i64>().ok()) {
                return Some(reference_time + chrono::Duration::hours(val));
            }
            if let Some(val) = mins_str.strip_suffix('s').and_then(|v| v.parse::<i64>().ok()) {
                return Some(reference_time + chrono::Duration::seconds(val));
            }
        }

        // 2. Standard 5-field cron: minute hour day month day_of_week
        let parts: Vec<&str> = clean.split_whitespace().collect();
        if parts.len() == 5 {
            return Self::next_cron_match_with_tz(&parts, reference_time, tz);
        }

        // Default fallback: 1 hour
        Some(reference_time + chrono::Duration::hours(1))
    }

    pub fn next_cron_match_with_tz(parts: &[&str], from: DateTime<Utc>, tz: chrono_tz::Tz) -> Option<DateTime<Utc>> {
        let mut candidate = from + chrono::Duration::minutes(1);
        candidate = candidate.with_second(0).unwrap().with_nanosecond(0).unwrap();

        // Scan next 50,400 minutes (up to 35 days ahead to support weekly and monthly schedules)
        for _ in 0..50_400 {
            let local_candidate = candidate.with_timezone(&tz);
            let min = local_candidate.minute();
            let hour = local_candidate.hour();
            let day = local_candidate.day();
            let month = local_candidate.month();
            let dow = local_candidate.weekday().num_days_from_sunday();

            if matches_cron_field(parts[0], min)
                && matches_cron_field(parts[1], hour)
                && matches_cron_field(parts[2], day)
                && matches_cron_field(parts[3], month)
                && matches_cron_field(parts[4], dow)
            {
                return Some(candidate);
            }
            candidate = candidate + chrono::Duration::minutes(1);
        }

        None
    }

    /// Executes a task directly using AppState
    pub async fn execute_task_direct(state: &AppState, task: &DbScheduledTask) -> (bool, String) {
        let now_str = Utc::now().to_rfc3339();
        let next_run_dt = Self::compute_next_run(&task.cron_expr, Utc::now());
        let next_run_str = next_run_dt.map(|dt| dt.to_rfc3339());

        log::info!("[Scheduler] Running action '{}' for task '{}'", task.action_type, task.name);

        let mut success = true;
        let mut tools_used_summary: Vec<String> = Vec::new();
        let mut steps_count: u32 = 1;
        let mut created_session_id: Option<String> = None;
        let mut steps_detail_json: Option<String> = None;
        let output: String;

        match task.action_type.as_str() {
            "skill" => {
                // Determine target skill and execution payload
                let mut tool_name = "run_skill_script".to_string();
                let mut args = serde_json::json!({});

                let payload_json: Option<serde_json::Value> = serde_json::from_str(&task.payload).ok();
                let direct_tool = payload_json.as_ref().and_then(|p| p.get("tool_name")).and_then(|v| v.as_str());
                let direct_args = payload_json.as_ref().and_then(|p| p.get("arguments")).cloned();

                if let (Some(t_name), Some(t_args)) = (direct_tool, direct_args) {
                    tool_name = t_name.to_string();
                    args = t_args;
                } else {
                    // Try to resolve from skill_slug, script_file, or search in prompt/name/description
                    let explicit_slug = payload_json.as_ref().and_then(|p| {
                        p.get("skill_slug")
                            .or_else(|| p.get("slug"))
                            .or_else(|| p.get("skill_id"))
                            .or_else(|| p.get("id"))
                            .and_then(|v| v.as_str())
                    });
                    let explicit_script = payload_json.as_ref().and_then(|p| {
                        p.get("script_file")
                            .or_else(|| p.get("script"))
                            .and_then(|v| v.as_str())
                    });

                    let all_skills = crate::services::memory_engine::MemoryGraphEngine::load_skills();
                    let matched_skill = if let Some(slug) = explicit_slug {
                        all_skills.into_iter().find(|s| {
                            s.id == slug
                                || s.id == format!("skill-{}", slug)
                                || slug == format!("skill-{}", s.id)
                                || s.name.eq_ignore_ascii_case(slug)
                        })
                    } else {
                        // Match from task name, description or prompt text
                        let search_corpus = format!(
                            "{} {} {}",
                            task.name,
                            task.description.as_deref().unwrap_or(""),
                            payload_json.as_ref().and_then(|p| p.get("prompt")).and_then(|v| v.as_str()).unwrap_or("")
                        ).to_lowercase();
                        all_skills.into_iter().find(|s| {
                            let s_name = s.name.to_lowercase();
                            let s_id = s.id.to_lowercase();
                            search_corpus.contains(&s_name)
                                || search_corpus.contains(&s_id)
                                || search_corpus.contains(&s_id.replace("skill-", ""))
                                || s.triggers.iter().any(|tr| search_corpus.contains(&tr.to_lowercase()))
                        })
                    };

                    if let Some(skill) = matched_skill {
                        // If explicit script provided or script exists in skill steps or skill scripts
                        let script_file = explicit_script
                            .map(|s| s.to_string())
                            .or_else(|| skill.steps.iter().find_map(|st| st.script_file.clone()))
                            .or_else(|| skill.scripts.first().cloned());

                        if let Some(script) = script_file {
                            tool_name = "run_skill_script".to_string();
                            args = serde_json::json!({
                                "slug": skill.id,
                                "script_file": script,
                                "args": []
                            });
                        } else if let Some(cmd) = skill.steps.iter().find_map(|st| st.effective_command()) {
                            tool_name = "run_command".to_string();
                            args = serde_json::json!({
                                "command": cmd,
                                "skill_name": skill.name
                            });
                        }
                    }
                }

                tools_used_summary.push(tool_name.clone());
                match Box::pin(crate::execute_tool_call_internal(state, "skills", &tool_name, args.clone())).await {
                    Ok(res) => {
                        let clean_output = if let Ok(cmd_res) = serde_json::from_value::<crate::core::memory::SkillCommandResult>(res.clone()) {
                            if !cmd_res.stdout.trim().is_empty() {
                                cmd_res.stdout
                            } else if !cmd_res.stderr.trim().is_empty() {
                                cmd_res.stderr
                            } else {
                                serde_json::to_string_pretty(&res).unwrap_or_default()
                            }
                        } else {
                            serde_json::to_string_pretty(&res).unwrap_or_default()
                        };
                        output = clean_output;
                        let step = crate::services::db::DbTaskRunStep {
                            step: 1,
                            thought: None,
                            tool_name: tool_name.clone(),
                            arguments: args,
                            status: "completed".to_string(),
                            result: Some(res),
                        };
                        steps_detail_json = serde_json::to_string(&vec![step]).ok();
                    }
                    Err(e) => {
                        success = false;
                        output = format!("Skill error: {}", e);
                        let step = crate::services::db::DbTaskRunStep {
                            step: 1,
                            thought: None,
                            tool_name: tool_name.clone(),
                            arguments: args,
                            status: "error".to_string(),
                            result: Some(serde_json::json!({ "error": e })),
                        };
                        steps_detail_json = serde_json::to_string(&vec![step]).ok();
                    }
                }
            }
            "memory_sleep" => {
                let mut engine = state.memory_engine.lock().await;
                let res = engine.run_sleep_cycle();
                tools_used_summary.push("associative_memory_sleep".to_string());
                output = format!("Memory sleep cycle processed {} events ({} facts consolidated)", res.events_processed, res.facts_consolidated);
            }
            "autonomous_prompt" => {
                // Runs autonomous agent turn
                let prompt_text = if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&task.payload) {
                    json_val.get("prompt").and_then(|v| v.as_str()).unwrap_or("Daily briefing").to_string()
                } else {
                    task.payload.clone()
                };

                let active_model = state.active_model.lock().await.clone();
                let sys_prompt = "You are Atena, an economic and autonomous assistant. Complete the scheduled task concisely.".to_string();
                let app_cfg = crate::core::config::AppConfig::load();

                let mut params = crate::core::model::InferenceParams::default();
                params.mcp_tools = Some(state.mcp_manager.list_all_tools().await);

                let req = crate::StreamChatRequest {
                    model: active_model.clone(),
                    system_prompt: sys_prompt,
                    messages: Some(vec![]),
                    user_message: Some(prompt_text.clone()),
                    session_id: Some(format!("cron-{}", task.id)),
                    session_title: Some(format!("Scheduled: {}", task.name)),
                    params,
                    mlx_host: app_cfg.mlx_server_host,
                    mlx_port: app_cfg.mlx_server_port,
                    ollama_host: app_cfg.ollama_host,
                    ollama_port: app_cfg.ollama_port,
                    enable_memory: Some(app_cfg.enable_cognitive_memory),
                    enable_facts_memory: Some(app_cfg.enable_facts_memory),
                    enable_skills_memory: Some(app_cfg.enable_skills_memory),
                    enable_episodic_memory: Some(app_cfg.enable_episodic_memory),
                };

                let cancel_tok = Arc::new(AtomicBool::new(false));
                let noop_cb = Arc::new(|_evt: crate::services::agent_loop::AgentStepEvent| {});

                match crate::services::agent_loop::AutonomousAgentRunner::run_loop(
                    state,
                    req,
                    6,
                    cancel_tok,
                    noop_cb,
                ).await {
                    Ok(loop_res) => {
                        output = loop_res.final_answer.clone();
                        tools_used_summary = loop_res.tools_used.clone();
                        steps_count = loop_res.steps_count;

                        // Build structured steps detail list with all tool calls, arguments and results
                        let mut steps_detail_list: Vec<crate::services::db::DbTaskRunStep> = Vec::new();
                        let mut step_num = 1;
                        for msg in &loop_res.messages {
                            if let Some(tcs) = &msg.tool_calls {
                                for tc in tcs {
                                    steps_detail_list.push(crate::services::db::DbTaskRunStep {
                                        step: step_num,
                                        thought: if msg.content.trim().is_empty() { None } else { Some(msg.content.clone()) },
                                        tool_name: tc.name.clone(),
                                        arguments: tc.arguments.clone(),
                                        status: tc.status.clone().unwrap_or_else(|| "completed".to_string()),
                                        result: tc.result.clone(),
                                    });
                                    step_num += 1;
                                }
                            }
                        }
                        if !steps_detail_list.is_empty() {
                            steps_detail_json = serde_json::to_string(&steps_detail_list).ok();
                        }

                        // Deliver to Chat if autonomous agent loop produced messages
                        let channel = task.delivery_channel.to_lowercase();
                        if channel == "chat" || channel == "both" || channel.is_empty() {
                            let session_uuid = format!("sched-{}-{}", task.id, Utc::now().timestamp());
                            created_session_id = Some(session_uuid.clone());
                            let title = format!("📅 {} ({})", task.name, Utc::now().format("%d/%m %H:%M"));

                            let db_messages: Vec<crate::services::db::DbChatMessage> = loop_res
                                .messages
                                .into_iter()
                                .enumerate()
                                .map(|(idx, m)| {
                                    let tool_calls_json = m.tool_calls.map(|tcs| serde_json::to_value(tcs).unwrap_or(serde_json::Value::Null));
                                    crate::services::db::DbChatMessage {
                                        id: format!("msg-{}-{}", Utc::now().timestamp_millis(), idx),
                                        role: m.role,
                                        content: m.content,
                                        thinking_content: None,
                                        tool_calls: tool_calls_json,
                                        tool_call_id: m.tool_call_id,
                                        images: m.images,
                                        attachments: None,
                                        timestamp: Utc::now().to_rfc3339(),
                                        tokens_count: None,
                                        generation_speed_tps: None,
                                        metrics: None,
                                        extra: std::collections::HashMap::new(),
                                    }
                                })
                                .collect();

                            let new_session = crate::services::db::DbChatSession {
                                id: session_uuid,
                                title,
                                model_id: active_model.as_ref().map(|m| m.id.clone()),
                                model_name: active_model.as_ref().map(|m| m.name.clone()),
                                project_id: None,
                                created_at: now_str.clone(),
                                updated_at: Some(Utc::now().to_rfc3339()),
                                messages: db_messages,
                                is_private: Some(false),
                                pinned: Some(false),
                                archived: Some(false),
                                archived_at: None,
                                extra: std::collections::HashMap::new(),
                            };
                            let _ = state.db.save_session(&new_session);
                        }
                    }
                    Err(e) => {
                        success = false;
                        output = format!("Autonomous execution error: {}", e);
                    }
                }
            }
            _ => {
                success = false;
                output = format!("Unknown action_type '{}'", task.action_type);
            }
        }

        // Multi-Channel Delivery (Telegram & Chat fallback)
        let channel = task.delivery_channel.to_lowercase();
        if success {
            // Deliver to Chat if not already delivered by autonomous agent loop
            if (channel == "chat" || channel == "both") && created_session_id.is_none() {
                let session_uuid = format!("sched-{}-{}", task.id, Utc::now().timestamp());
                created_session_id = Some(session_uuid.clone());
                let title = format!("📅 {} ({})", task.name, Utc::now().format("%d/%m %H:%M"));

                let active_model = state.active_model.lock().await.clone();
                let db_messages = vec![
                    crate::services::db::DbChatMessage {
                        id: format!("msg-{}-0", Utc::now().timestamp_millis()),
                        role: "user".to_string(),
                        content: format!("Execute routine: {}", task.name),
                        thinking_content: None,
                        tool_calls: None,
                        tool_call_id: None,
                        images: None,
                        attachments: None,
                        timestamp: Utc::now().to_rfc3339(),
                        tokens_count: None,
                        generation_speed_tps: None,
                        metrics: None,
                        extra: std::collections::HashMap::new(),
                    },
                    crate::services::db::DbChatMessage {
                        id: format!("msg-{}-1", Utc::now().timestamp_millis() + 1),
                        role: "assistant".to_string(),
                        content: output.clone(),
                        thinking_content: None,
                        tool_calls: None,
                        tool_call_id: None,
                        images: None,
                        attachments: None,
                        timestamp: Utc::now().to_rfc3339(),
                        tokens_count: None,
                        generation_speed_tps: None,
                        metrics: None,
                        extra: std::collections::HashMap::new(),
                    },
                ];

                let new_session = crate::services::db::DbChatSession {
                    id: session_uuid,
                    title,
                    model_id: active_model.as_ref().map(|m| m.id.clone()),
                    model_name: active_model.as_ref().map(|m| m.name.clone()),
                    project_id: None,
                    created_at: now_str.clone(),
                    updated_at: Some(Utc::now().to_rfc3339()),
                    messages: db_messages,
                    is_private: Some(false),
                    pinned: Some(false),
                    archived: Some(false),
                    archived_at: None,
                    extra: std::collections::HashMap::new(),
                };
                let _ = state.db.save_session(&new_session);
            }

            // Deliver to Telegram
            if channel == "telegram" || channel == "both" {
                let app_cfg = crate::core::config::AppConfig::load();
                let bot_token = app_cfg.gateways.telegram.bot_token.clone();
                if !bot_token.trim().is_empty() {
                    let target_chat_id = task.delivery_target
                        .as_deref()
                        .and_then(|s| s.parse::<i64>().ok())
                        .or_else(|| app_cfg.gateways.telegram.allowed_user_ids.first().copied());

                    if let Some(chat_id) = target_chat_id {
                        let mut telegram_msg = format!("📅 *{}*\n────────────────────\n", task.name);
                        if !tools_used_summary.is_empty() {
                            telegram_msg.push_str("🛠️ _Ferramentas:_ ");
                            telegram_msg.push_str(&tools_used_summary.join(", "));
                            telegram_msg.push_str("\n\n");
                        }
                        telegram_msg.push_str(&output);

                        let _ = crate::services::gateways::telegram::send_proactive_telegram_message(
                            &bot_token,
                            chat_id,
                            &telegram_msg,
                        ).await;
                    }
                }
            }
        }

        // Record execution run in SQLite audit log
        let tools_json = if tools_used_summary.is_empty() {
            None
        } else {
            serde_json::to_string(&tools_used_summary).ok()
        };

        let run_record = crate::services::db::DbScheduledTaskRun {
            id: format!("run-{}", Utc::now().timestamp_millis()),
            task_id: task.id.clone(),
            task_name: task.name.clone(),
            action_type: task.action_type.clone(),
            delivery_channel: task.delivery_channel.clone(),
            status: if success { "success".to_string() } else { "error".to_string() },
            steps_count,
            tools_used: tools_json,
            steps_detail: steps_detail_json,
            output: output.clone(),
            session_id: created_session_id,
            executed_at: now_str.clone(),
        };
        let _ = state.db.record_task_run(&run_record);

        // Update execution timestamps and last_result in SQLite
        let _ = state.db.update_task_run_timestamps(&task.id, &now_str, next_run_str.as_deref(), Some(&output));

        log::info!("[Scheduler] Task '{}' completed (success={}, channel='{}'). Steps: {}, Tools: {:?}", task.name, success, task.delivery_channel, steps_count, tools_used_summary);
        (success, output)
    }

    /// Executes a single due scheduled task
    pub async fn execute_task(app_handle: AppHandle, task: DbScheduledTask) {
        if let Some(state) = app_handle.try_state::<AppState>() {
            let (success, output) = Self::execute_task_direct(&state, &task).await;
            let exec_result = ScheduledTaskExecutionResult {
                task_id: task.id.clone(),
                task_name: task.name.clone(),
                success,
                output,
                executed_at: Utc::now().to_rfc3339(),
            };

            // Emit session refresh and execution event
            let _ = app_handle.emit("db_sessions_updated", ());
            let _ = app_handle.emit("atena://scheduled-task-finished", &exec_result);
        }
    }
}

fn matches_cron_field(field: &str, value: u32) -> bool {
    if field == "*" {
        return true;
    }
    if let Ok(v) = field.parse::<u32>() {
        return v == value;
    }
    if let Some(step_str) = field.strip_prefix("*/") {
        if let Ok(step) = step_str.parse::<u32>() {
            return step > 0 && value % step == 0;
        }
    }
    if field.contains(',') {
        return field.split(',').any(|f| matches_cron_field(f.trim(), value));
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_next_run_intervals() {
        let now = Utc::now();
        let next_hourly = BackgroundScheduler::compute_next_run("@hourly", now).unwrap();
        assert_eq!(next_hourly, now + chrono::Duration::hours(1));

        let next_24h = BackgroundScheduler::compute_next_run("@every 24h", now).unwrap();
        assert_eq!(next_24h, now + chrono::Duration::days(1));

        let next_custom = BackgroundScheduler::compute_next_run("interval:120", now).unwrap();
        assert_eq!(next_custom, now + chrono::Duration::seconds(120));
    }

    #[test]
    fn test_timezone_aware_cron_matching() {
        use chrono::TimeZone;
        let tz = chrono_tz::America::Sao_Paulo;

        // Base time: 2026-09-18 16:30:00 UTC (which is 13:30 in America/Sao_Paulo)
        let base = Utc.with_ymd_and_hms(2026, 9, 18, 16, 30, 0).unwrap();
        let next = BackgroundScheduler::compute_next_run_with_tz("0 17 * * *", base, tz).unwrap();

        // 17:00 in America/Sao_Paulo (UTC-3) is 20:00 UTC
        assert_eq!(next, Utc.with_ymd_and_hms(2026, 9, 18, 20, 0, 0).unwrap());

        let local_dt = next.with_timezone(&tz);
        assert_eq!(local_dt.hour(), 17);
        assert_eq!(local_dt.minute(), 0);
    }

    #[test]
    fn test_cron_pattern_matcher() {
        assert!(matches_cron_field("*", 42));
        assert!(matches_cron_field("15", 15));
        assert!(!matches_cron_field("15", 16));
        assert!(matches_cron_field("*/5", 25));
        assert!(!matches_cron_field("*/5", 26));
        assert!(matches_cron_field("1,2,3", 2));
    }
}
