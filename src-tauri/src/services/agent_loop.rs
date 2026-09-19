use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::core::model::ChatMessage;
use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStepEvent {
    pub session_id: String,
    pub iteration: u32,
    pub max_iterations: u32,
    /// "thought" | "tool_call" | "tool_result" | "reflection" | "completed" | "budget_exceeded" | "aborted" | "error"
    pub step_type: String,
    pub content: String,
    pub tool_name: Option<String>,
    pub tool_args: Option<serde_json::Value>,
    pub tool_result: Option<serde_json::Value>,
    pub is_done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomousLoopResult {
    pub final_answer: String,
    pub messages: Vec<ChatMessage>,
    pub steps_count: u32,
    pub tools_used: Vec<String>,
}

pub struct AutonomousAgentRunner;

impl AutonomousAgentRunner {
    /// Executes an autonomous multi-step ReAct loop until task completion, cancellation,
    /// or reaching the hard iteration budget limit.
    pub async fn run_loop(
        state: &AppState,
        req: crate::StreamChatRequest,
        max_steps: u32,
        cancellation_token: Arc<AtomicBool>,
        on_step_event: Arc<dyn Fn(AgentStepEvent) + Send + Sync>,
    ) -> Result<AutonomousLoopResult, String> {
        let session_id = req.session_id.clone().unwrap_or_else(|| "autonomous-task".to_string());
        let max_iterations = max_steps.clamp(1, 15);
        let mut current_iteration: u32 = 0;
        let mut messages = req.messages.clone().unwrap_or_default();
        let mut all_tools_used: Vec<String> = Vec::new();

        // If a new user message is provided in the request, append it to the conversation history
        if let Some(user_text) = &req.user_message {
            if !user_text.trim().is_empty() {
                messages.push(ChatMessage {
                    role: "user".to_string(),
                    content: user_text.clone(),
                    images: None,
                    tool_calls: None,
                    tool_call_id: None,
                });
            }
        }

        let mut final_answer = String::new();

        while current_iteration < max_iterations {
            if cancellation_token.load(Ordering::Relaxed) {
                on_step_event(AgentStepEvent {
                    session_id: session_id.clone(),
                    iteration: current_iteration + 1,
                    max_iterations,
                    step_type: "aborted".to_string(),
                    content: "Autonomous task aborted by user or timeout.".to_string(),
                    tool_name: None,
                    tool_args: None,
                    tool_result: None,
                    is_done: true,
                });
                return Err("Autonomous loop aborted".to_string());
            }

            current_iteration += 1;

            // Check and inject ephemeral task scratchpad block if present
            let mut effective_system_prompt = req.system_prompt.clone();
            if let Some(scratchpad_block) = state.scratchpad.format_context_block(&session_id).await {
                effective_system_prompt = format!("{}\n\n{}", effective_system_prompt, scratchpad_block);
            }

            // Prepare iteration streaming accumulators
            let iter_accumulated_text = Arc::new(std::sync::Mutex::new(String::new()));
            let text_clone = iter_accumulated_text.clone();
            let iter_captured_calls = Arc::new(std::sync::Mutex::new(Vec::new()));
            let calls_clone = iter_captured_calls.clone();

            let step_cb = on_step_event.clone();
            let step_sess = session_id.clone();
            let iter_num = current_iteration;

            let on_chunk = Arc::new(move |payload: crate::ChatChunkPayload| {
                if let Ok(mut guard) = text_clone.lock() {
                    *guard = payload.content.clone();
                }
                if let Some(tcs) = payload.tool_calls {
                    if let Ok(mut guard) = calls_clone.lock() {
                        *guard = tcs;
                    }
                }
                // Emit streaming thought update
                step_cb(AgentStepEvent {
                    session_id: step_sess.clone(),
                    iteration: iter_num,
                    max_iterations,
                    step_type: "thought".to_string(),
                    content: payload.content,
                    tool_name: None,
                    tool_args: None,
                    tool_result: None,
                    is_done: payload.is_done,
                });
            });

            let iter_req = crate::StreamChatRequest {
                model: req.model.clone(),
                system_prompt: effective_system_prompt.clone(),
                messages: Some(messages.clone()),
                user_message: None,
                session_id: Some(session_id.clone()),
                session_title: req.session_title.clone(),
                params: req.params.clone(),
                mlx_host: req.mlx_host.clone(),
                mlx_port: req.mlx_port,
                ollama_host: req.ollama_host.clone(),
                ollama_port: req.ollama_port,
                enable_memory: req.enable_memory,
                enable_facts_memory: req.enable_facts_memory,
                enable_skills_memory: req.enable_skills_memory,
                enable_episodic_memory: req.enable_episodic_memory,
            };

            let exec_res = crate::execute_stream_chat_internal(state, iter_req, on_chunk).await;
            if let Err(err) = exec_res {
                on_step_event(AgentStepEvent {
                    session_id: session_id.clone(),
                    iteration: current_iteration,
                    max_iterations,
                    step_type: "error".to_string(),
                    content: format!("Inference error at step {}: {}", current_iteration, err),
                    tool_name: None,
                    tool_args: None,
                    tool_result: None,
                    is_done: true,
                });
                return Err(err);
            }

            let interim_text = iter_accumulated_text.lock().unwrap().clone();
            let current_tool_calls = iter_captured_calls.lock().unwrap().clone();

            // Case A: Model emitted no tool calls — task is concluded!
            if current_tool_calls.is_empty() {
                final_answer = interim_text.clone();
                messages.push(ChatMessage {
                    role: "assistant".to_string(),
                    content: final_answer.clone(),
                    images: None,
                    tool_calls: None,
                    tool_call_id: None,
                });
                on_step_event(AgentStepEvent {
                    session_id: session_id.clone(),
                    iteration: current_iteration,
                    max_iterations,
                    step_type: "completed".to_string(),
                    content: final_answer.clone(),
                    tool_name: None,
                    tool_args: None,
                    tool_result: None,
                    is_done: true,
                });
                break;
            }

            // Case B: Model called tools — execute them and collect observations
            let mut executed_tools = Vec::new();
            for mut tc in current_tool_calls {
                let server_id = tc.server_id.clone().unwrap_or_else(|| {
                    if tc.name.starts_with("atena_") {
                        "atena_native".to_string()
                    } else {
                        "skills".to_string()
                    }
                });

                on_step_event(AgentStepEvent {
                    session_id: session_id.clone(),
                    iteration: current_iteration,
                    max_iterations,
                    step_type: "tool_call".to_string(),
                    content: format!("Invoking tool '{}'", tc.name),
                    tool_name: Some(tc.name.clone()),
                    tool_args: Some(tc.arguments.clone()),
                    tool_result: None,
                    is_done: false,
                });

                log::info!("[Autonomous ReAct] Step {} calling tool '{}' on '{}'", current_iteration, tc.name, server_id);

                let tool_exec_res = Box::pin(crate::execute_tool_call_internal(state, &server_id, &tc.name, tc.arguments.clone())).await;
                match tool_exec_res {
                    Ok(res_val) => {
                        tc.status = Some("completed".to_string());
                        tc.result = Some(res_val.clone());
                        on_step_event(AgentStepEvent {
                            session_id: session_id.clone(),
                            iteration: current_iteration,
                            max_iterations,
                            step_type: "tool_result".to_string(),
                            content: format!("Tool '{}' completed successfully", tc.name),
                            tool_name: Some(tc.name.clone()),
                            tool_args: Some(tc.arguments.clone()),
                            tool_result: Some(res_val),
                            is_done: false,
                        });
                    }
                    Err(err_msg) => {
                        log::warn!("[Autonomous ReAct] Step {} tool '{}' failed: {}", current_iteration, tc.name, err_msg);
                        let err_json = serde_json::json!({
                            "status": "error",
                            "error": err_msg
                        });
                        tc.status = Some("error".to_string());
                        tc.result = Some(err_json.clone());
                        on_step_event(AgentStepEvent {
                            session_id: session_id.clone(),
                            iteration: current_iteration,
                            max_iterations,
                            step_type: "tool_result".to_string(),
                            content: format!("Tool '{}' returned error: {}", tc.name, err_msg),
                            tool_name: Some(tc.name.clone()),
                            tool_args: Some(tc.arguments.clone()),
                            tool_result: Some(err_json),
                            is_done: false,
                        });
                    }
                }
                all_tools_used.push(tc.name.clone());
                executed_tools.push(tc);
            }

            // Append assistant step with executed tool calls
            messages.push(ChatMessage {
                role: "assistant".to_string(),
                content: interim_text,
                images: None,
                tool_calls: Some(executed_tools.clone()),
                tool_call_id: None,
            });

            // Format observation block and feed back into conversation for next thought
            let tool_results_block = crate::services::backend::BackendManager::format_tool_results_block(&executed_tools);
            let observation_content = format!(
                "[Observation / Tool Output - Step {} of {}]:\n{}\n\n\
                Reflect on this result. If errors occurred, determine the fix. Continue with the next necessary action, or present the final answer if the objective is complete.",
                current_iteration, max_iterations, tool_results_block
            );

            messages.push(ChatMessage {
                role: "user".to_string(),
                content: observation_content,
                images: None,
                tool_calls: None,
                tool_call_id: None,
            });

            // If budget is exhausted, run one final synthesis turn
            if current_iteration >= max_iterations {
                on_step_event(AgentStepEvent {
                    session_id: session_id.clone(),
                    iteration: current_iteration,
                    max_iterations,
                    step_type: "budget_exceeded".to_string(),
                    content: format!("Max iteration budget of {} steps reached. Synthesizing final progress report.", max_iterations),
                    tool_name: None,
                    tool_args: None,
                    tool_result: None,
                    is_done: false,
                });

                messages.push(ChatMessage {
                    role: "user".to_string(),
                    content: "The step budget limit has been reached. Please synthesize a final response summarizing what was accomplished, what failed, and what steps remain.".to_string(),
                    images: None,
                    tool_calls: None,
                    tool_call_id: None,
                });

                let syn_accumulated = Arc::new(std::sync::Mutex::new(String::new()));
                let syn_clone = syn_accumulated.clone();
                let syn_cb = on_step_event.clone();
                let syn_sess = session_id.clone();

                let on_syn_chunk = Arc::new(move |payload: crate::ChatChunkPayload| {
                    if let Ok(mut guard) = syn_clone.lock() {
                        *guard = payload.content.clone();
                    }
                    syn_cb(AgentStepEvent {
                        session_id: syn_sess.clone(),
                        iteration: current_iteration,
                        max_iterations,
                        step_type: "thought".to_string(),
                        content: payload.content,
                        tool_name: None,
                        tool_args: None,
                        tool_result: None,
                        is_done: payload.is_done,
                    });
                });

                let syn_req = crate::StreamChatRequest {
                    model: req.model.clone(),
                    system_prompt: effective_system_prompt,
                    messages: Some(messages.clone()),
                    user_message: None,
                    session_id: Some(session_id.clone()),
                    session_title: req.session_title.clone(),
                    params: req.params.clone(),
                    mlx_host: req.mlx_host.clone(),
                    mlx_port: req.mlx_port,
                    ollama_host: req.ollama_host.clone(),
                    ollama_port: req.ollama_port,
                    enable_memory: req.enable_memory,
                    enable_facts_memory: req.enable_facts_memory,
                    enable_skills_memory: req.enable_skills_memory,
                    enable_episodic_memory: req.enable_episodic_memory,
                };

                let _ = crate::execute_stream_chat_internal(state, syn_req, on_syn_chunk).await;
                final_answer = syn_accumulated.lock().unwrap().clone();
                messages.push(ChatMessage {
                    role: "assistant".to_string(),
                    content: final_answer.clone(),
                    images: None,
                    tool_calls: None,
                    tool_call_id: None,
                });

                on_step_event(AgentStepEvent {
                    session_id: session_id.clone(),
                    iteration: current_iteration,
                    max_iterations,
                    step_type: "completed".to_string(),
                    content: final_answer.clone(),
                    tool_name: None,
                    tool_args: None,
                    tool_result: None,
                    is_done: true,
                });
                break;
            }
        }

        Ok(AutonomousLoopResult {
            final_answer,
            messages,
            steps_count: current_iteration,
            tools_used: all_tools_used,
        })
    }
}
