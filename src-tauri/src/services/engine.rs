use crate::core::model::{ChatMessage, InferenceParams, ModelInfo};
use crate::services::backend::BackendManager;

pub struct InferenceEngine;

impl InferenceEngine {
    /// Executes inference against the selected local backend (MLX-LM, Ollama, etc.)
    /// Returns a tuple of (response_text, thinking, elapsed_milliseconds) for real tps calculation.
    #[allow(dead_code)]
    pub async fn execute_chat(
        model: Option<ModelInfo>,
        system_prompt: String,
        messages: Vec<ChatMessage>,
        params: InferenceParams,
        mlx_host: String,
        mlx_port: u16,
        ollama_host: String,
        ollama_port: u16,
    ) -> (String, Option<String>, u64) {
        let start = std::time::Instant::now();

        let (response, thinking) = if let Some(m) = model {
            match BackendManager::execute_stream(
                &m,
                &system_prompt,
                &messages,
                &params,
                &mlx_host,
                mlx_port,
                &ollama_host,
                ollama_port,
            )
            .await
            {
                Ok((response, thinking, _tools)) => (response, thinking),
                Err(err) => (format!("⚠️ Error executing inference: {}", err), None),
            }
        } else {
            ("No model selected. Please select and load a model in the 'Models' tab (MLX or Ollama).".to_string(), None)
        };

        let elapsed_ms = start.elapsed().as_millis() as u64;
        (response, thinking, elapsed_ms)
    }

    /// Executes inference with real-time streaming tokens, thinking, tool_calls extraction, and generation efficiency metrics.
    pub async fn execute_chat_stream<F>(
        model: Option<ModelInfo>,
        system_prompt: String,
        messages: Vec<ChatMessage>,
        params: InferenceParams,
        mlx_host: String,
        mlx_port: u16,
        ollama_host: String,
        ollama_port: u16,
        mut on_chunk: F,
    ) where
        F: FnMut(String, Option<String>, Option<Vec<crate::core::mcp::McpToolCall>>, bool, u64, Option<crate::core::model::GenerationMetrics>) + Send + 'static,
    {
        let start = std::time::Instant::now();
        if let Some(m) = model {
            let res = BackendManager::execute_stream_callback(
                &m,
                &system_prompt,
                &messages,
                &params,
                &mlx_host,
                mlx_port,
                &ollama_host,
                ollama_port,
                |content, thinking, tool_calls, is_last, elapsed, metrics| {
                    on_chunk(content, thinking, tool_calls, is_last, elapsed, metrics);
                },
            )
            .await;

            if let Err(err) = res {
                on_chunk(
                    format!("⚠️ Error executing inference: {}", err),
                    None,
                    None,
                    true,
                    start.elapsed().as_millis() as u64,
                    None,
                );
            }
        } else {
            on_chunk(
                "No model selected. Please select and load a model in the 'Models' tab (MLX or Ollama).".to_string(),
                None,
                None,
                true,
                0,
                None,
            );
        }
    }
}
