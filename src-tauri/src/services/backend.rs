use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

static REQ_STREAM_LOG_COUNTER: AtomicU64 = AtomicU64::new(100);
static ACTIVE_PROMPT_TICKER: std::sync::Mutex<Option<(Arc<AtomicBool>, tokio::task::AbortHandle)>> =
    std::sync::Mutex::new(None);
static ACTIVE_INFERENCE_ABORT: std::sync::Mutex<Option<Arc<AtomicBool>>> =
    std::sync::Mutex::new(None);
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

use crate::core::model::{BackendType, InferenceParams, ModelInfo};
#[allow(unused_imports)]
use crate::core::process::{silent_command, silent_tokio_command, SilentCommand};

pub struct StreamingThinkingState {
    pub in_think_tag: bool,
    pub starts_in_thinking: bool,
    pub closed_think_tag: bool,
    pub thinking_acc: String,
    pub content_acc: String,
    pub raw_buffer: String,
}

impl StreamingThinkingState {
    pub const OPEN_TAGS: &'static [&'static str] = &[
        "<|channel>thought",
        "<|channel|>thought",
        "<|thought|>",
        "<|start_thought|>",
        "<thought>",
        "<think>",
        "<thinking>",
    ];

    pub const CLOSE_TAGS: &'static [&'static str] = &[
        "<channel|>",
        "<|channel|>",
        "<|/thought|>",
        "<|end_of_thought|>",
        "<|end_thought|>",
        "</thought>",
        "</think>",
        "</thinking>",
    ];

    /// `starts_in_thinking` is used by templates that put an opening thought tag in
    /// the prompt or models that stream reasoning first. In that format the server
    /// streams reasoning text directly and eventually emits its closing tag.
    pub fn new(starts_in_thinking: bool) -> Self {
        Self {
            in_think_tag: starts_in_thinking,
            starts_in_thinking,
            closed_think_tag: false,
            thinking_acc: String::new(),
            content_acc: String::new(),
            raw_buffer: String::new(),
        }
    }

    pub fn strip_special_channel_tokens(text: &str) -> String {
        let mut cleaned = text.to_string();
        let tokens_to_remove = [
            "<|channel>thought",
            "<|channel|>thought",
            "<channel|>",
            "<|channel|>",
            "<|channel>",
            "<start_of_turn>",
            "<end_of_turn>",
            "<turn|>",
            "<|turn>",
            "<|thought|>",
            "<|/thought|>",
            "<|start_thought|>",
            "<|end_thought|>",
            "<|end_of_thought|>",
        ];
        for tok in &tokens_to_remove {
            cleaned = cleaned.replace(tok, "");
        }
        cleaned
    }

    pub fn push_delta(&mut self, reasoning_delta: Option<&str>, content_delta: Option<&str>) {
        if let Some(r) = reasoning_delta {
            if !r.is_empty() {
                self.thinking_acc.push_str(r);
                // Backends with an explicit reasoning field (e.g. MLX 0.31+) signal reasoning explicitly.
                self.closed_think_tag = true;
                if self.in_think_tag && self.raw_buffer.is_empty() {
                    self.in_think_tag = false;
                }
            }
        }

        if let Some(c) = content_delta {
            if !c.is_empty() {
                self.raw_buffer.push_str(c);
                self.process_raw_buffer();
            }
        }
    }

    fn process_raw_buffer(&mut self) {
        loop {
            // Special case: the server/model resends an opening tag at the start of the stream when starts_in_thinking=true
            if self.in_think_tag && self.thinking_acc.trim().is_empty() {
                for &tag in Self::OPEN_TAGS {
                    if let Some(pos) = self.raw_buffer.find(tag) {
                        if self.raw_buffer[..pos].trim().is_empty() {
                            self.raw_buffer.drain(..pos + tag.len());
                            break;
                        }
                    }
                }
            }

            if !self.in_think_tag {
                let earliest_open = Self::OPEN_TAGS.iter()
                    .filter_map(|&tag| self.raw_buffer.find(tag).map(|pos| (pos, tag.len())))
                    .min_by_key(|&(pos, _)| pos);

                let earliest_close = Self::CLOSE_TAGS.iter()
                    .filter_map(|&tag| self.raw_buffer.find(tag).map(|pos| (pos, tag.len())))
                    .min_by_key(|&(pos, _)| pos);

                // Special case: we found a closing tag (e.g. </think>) BEFORE any opening tag.
                // This happens when the template injected the initial <think> in the generation prompt.
                if let Some((close_pos, close_len)) = earliest_close {
                    let is_before_open = match earliest_open {
                        Some((open_pos, _)) => close_pos < open_pos,
                        None => true,
                    };

                    if is_before_open {
                        let think_part = &self.raw_buffer[..close_pos];
                        self.thinking_acc.push_str(&self.content_acc);
                        self.content_acc.clear();
                        self.thinking_acc.push_str(think_part);
                        self.raw_buffer.drain(..close_pos + close_len);
                        self.in_think_tag = false;
                        self.closed_think_tag = true;
                        continue;
                    }
                }

                if let Some((pos, len)) = earliest_open {
                    let before = &self.raw_buffer[..pos];
                    self.content_acc.push_str(before);
                    self.raw_buffer.drain(..pos + len);
                    self.in_think_tag = true;
                } else {
                    let partial_match = Self::OPEN_TAGS.iter().any(|tag| {
                        (1..tag.len()).any(|i| self.raw_buffer.ends_with(&tag[..i]))
                    });
                    if !partial_match {
                        self.content_acc.push_str(&self.raw_buffer);
                        self.raw_buffer.clear();
                    }
                    break;
                }
            } else {
                let earliest_close = Self::CLOSE_TAGS.iter()
                    .filter_map(|&tag| self.raw_buffer.find(tag).map(|pos| (pos, tag.len())))
                    .min_by_key(|&(pos, _)| pos);

                if let Some((pos, len)) = earliest_close {
                    let think_part = &self.raw_buffer[..pos];
                    self.thinking_acc.push_str(think_part);
                    self.raw_buffer.drain(..pos + len);
                    self.in_think_tag = false;
                    self.closed_think_tag = true;
                } else {
                    let partial_match = Self::CLOSE_TAGS.iter().any(|tag| {
                        (1..tag.len()).any(|i| self.raw_buffer.ends_with(&tag[..i]))
                    });
                    if !partial_match {
                        self.thinking_acc.push_str(&self.raw_buffer);
                        self.raw_buffer.clear();
                    }
                    break;
                }
            }
        }
    }

    pub fn finalize(&mut self, enable_thinking: bool) -> (String, Option<String>) {
        if !self.raw_buffer.is_empty() {
            if self.in_think_tag {
                self.thinking_acc.push_str(&self.raw_buffer);
            } else {
                self.content_acc.push_str(&self.raw_buffer);
            }
            self.raw_buffer.clear();
        }

        let mut cleaned_content = Self::strip_special_channel_tokens(&self.content_acc);
        let mut cleaned_thinking_raw = Self::strip_special_channel_tokens(&self.thinking_acc);

        // Remove any residual tags that might have remained in the buffers
        for tag in Self::OPEN_TAGS.iter().chain(Self::CLOSE_TAGS.iter()) {
            cleaned_content = cleaned_content.replace(tag, "");
            cleaned_thinking_raw = cleaned_thinking_raw.replace(tag, "");
        }
        let mut cleaned_content = cleaned_content.trim().to_string();
        let cleaned_thinking_raw = cleaned_thinking_raw.trim().to_string();

        if !enable_thinking {
            if cleaned_content.is_empty() && !cleaned_thinking_raw.is_empty() {
                cleaned_content = cleaned_thinking_raw;
            }
            (cleaned_content, None)
        } else {
            let clean_thinking = if cleaned_thinking_raw.is_empty() {
                None
            } else {
                Some(cleaned_thinking_raw)
            };
            (cleaned_content, clean_thinking)
        }
    }

    pub fn current(&self, enable_thinking: bool) -> (String, Option<String>) {
        let mut clone = Self {
            in_think_tag: self.in_think_tag,
            starts_in_thinking: self.starts_in_thinking,
            closed_think_tag: self.closed_think_tag,
            thinking_acc: self.thinking_acc.clone(),
            content_acc: self.content_acc.clone(),
            raw_buffer: self.raw_buffer.clone(),
        };
        clone.finalize(enable_thinking)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelLoadProgress {
    pub percent: u8,
    pub message: String,
}

pub type ProgressCallback = Arc<dyn Fn(ModelLoadProgress) + Send + Sync>;

/// Extracts loading percentage (1 to 99) from llama-server or MLX/tqdm log messages
pub fn parse_model_progress(line: &str) -> Option<u8> {
    // 1. llama.cpp layer offload pattern: "offloaded X/Y layers"
    if let Some(idx) = line.find("offloaded ") {
        let rest = &line[idx + "offloaded ".len()..];
        if let Some(slash_idx) = rest.find('/') {
            let num1_str = rest[..slash_idx].trim();
            let after_slash = &rest[slash_idx + 1..];
            let end_num2 = after_slash.find(|c: char| !c.is_ascii_digit()).unwrap_or(after_slash.len());
            let num2_str = &after_slash[..end_num2];
            if let (Ok(n1), Ok(n2)) = (num1_str.parse::<f32>(), num2_str.parse::<f32>()) {
                if n2 > 0.0 {
                    let p = ((n1 / n2) * 100.0).round() as u8;
                    return Some(p.clamp(1, 99));
                }
            }
        }
    }

    // 2. Percentage pattern "( XX.X%)" or " XX%"
    if let Some(percent_idx) = line.find('%') {
        let before = line[..percent_idx].trim_end();
        let mut start_idx = before.len();
        for (i, c) in before.char_indices().rev() {
            if c.is_ascii_digit() || c == '.' {
                start_idx = i;
            } else {
                break;
            }
        }
        if start_idx < before.len() {
            let num_str = &before[start_idx..];
            if let Ok(val) = num_str.parse::<f32>() {
                if val >= 0.0 && val <= 100.0 {
                    let p = val.round() as u8;
                    return Some(p.clamp(1, 99));
                }
            }
        }
    }

    None
}

/// Tracks loading progress combining real-time log reading with a smooth asymptotic ticker
struct LoadProgressTracker {
    current_percent: Arc<AtomicU8>,
    is_done: Arc<AtomicBool>,
    callback: Option<ProgressCallback>,
}

impl LoadProgressTracker {
    pub fn new(callback: Option<ProgressCallback>, initial_msg: &str) -> Self {
        let current_percent = Arc::new(AtomicU8::new(5));
        let is_done = Arc::new(AtomicBool::new(false));

        if let Some(ref cb) = callback {
            cb(ModelLoadProgress {
                percent: 5,
                message: initial_msg.to_string(),
            });
        }

        let tracker = Self {
            current_percent: current_percent.clone(),
            is_done: is_done.clone(),
            callback: callback.clone(),
        };

        let cb_ticker = callback.clone();
        let pct_ticker = current_percent.clone();
        let done_ticker = is_done.clone();
        let start_time = std::time::Instant::now();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(350));
            while !done_ticker.load(Ordering::Relaxed) {
                interval.tick().await;
                if done_ticker.load(Ordering::Relaxed) {
                    break;
                }
                let elapsed = start_time.elapsed().as_secs_f32();
                let target = (100.0 * (1.0 - (-elapsed / 5.5).exp()) * 0.94) as u8;
                let cur = pct_ticker.load(Ordering::Relaxed);
                let next = cur.max(target.min(94)).max(5);
                if next > cur {
                    pct_ticker.store(next, Ordering::Relaxed);
                    if let Some(ref cb) = cb_ticker {
                        cb(ModelLoadProgress {
                            percent: next,
                            message: format!("Carregando pesos na memória... {}%", next),
                        });
                    }
                }
            }
        });

        tracker
    }

    pub fn attach_stream<R: tokio::io::AsyncRead + Unpin + Send + 'static>(&self, reader: R) {
        let pct = self.current_percent.clone();
        let cb = self.callback.clone();
        let done = self.is_done.clone();

        tokio::spawn(async move {
            let mut lines = BufReader::new(reader).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if !done.load(Ordering::Relaxed) {
                    if let Some(parsed) = parse_model_progress(&line) {
                        let cur = pct.load(Ordering::Relaxed);
                        if parsed > cur {
                            pct.store(parsed, Ordering::Relaxed);
                            if let Some(ref callback) = cb {
                                callback(ModelLoadProgress {
                                    percent: parsed,
                                    message: format!("Carregando tensores... {}%", parsed),
                                });
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn finish(&self, success_msg: &str) {
        self.is_done.store(true, Ordering::Relaxed);
        self.current_percent.store(100, Ordering::Relaxed);
        if let Some(ref cb) = self.callback {
            cb(ModelLoadProgress {
                percent: 100,
                message: success_msg.to_string(),
            });
        }
    }

    pub fn cancel(&self) {
        self.is_done.store(true, Ordering::Relaxed);
    }
}

pub struct BackendManager {
    mlx_process: Arc<Mutex<Option<Child>>>,
    llama_process: Arc<Mutex<Option<Child>>>,
    ollama_process: Arc<Mutex<Option<Child>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ChatCompletionChunk {
    pub delta: String,
    pub tokens_per_sec: f32,
    pub is_done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgySessionStatus {
    pub installed: bool,
    pub binary_path: Option<String>,
    pub authenticated: bool,
    pub active_account: Option<String>,
    pub available_models: Vec<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgyUsageBucket {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub window: Option<String>,
    pub remaining_fraction: f64,
    pub reset_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgyUsageGroup {
    pub name: String,
    pub description: Option<String>,
    pub buckets: Vec<AgyUsageBucket>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgyUsageData {
    pub description: Option<String>,
    pub groups: Vec<AgyUsageGroup>,
}

impl BackendManager {
    pub fn model_starts_in_thinking(model: &ModelInfo, enable_thinking: bool) -> bool {
        if !enable_thinking {
            return false;
        }

        if model.supports_thinking {
            return true;
        }

        let path_str = model.local_path.as_deref().unwrap_or("");
        let identity = format!("{} {} {} {}", model.id, model.name, model.architecture, path_str).to_lowercase();
        identity.contains("qwen3")
            || identity.contains("ornith")
            || identity.contains("qwq")
            || identity.contains("deepseek-r1")
            || identity.contains("r1-distill")
            || identity.contains("gemma-4")
            || identity.contains("gemma4")
            || identity.contains("gemma 4")
            || identity.contains("gemma-3")
            || identity.contains("gemma3")
            || identity.contains("gemma 3")
            || identity.contains("phi-4")
            || identity.contains("phi4")
            || identity.contains("reasoning")
            || identity.contains("thinking")
            || identity.contains("cot")
    }

    pub fn new() -> Self {
        Self {
            mlx_process: Arc::new(Mutex::new(None)),
            llama_process: Arc::new(Mutex::new(None)),
            ollama_process: Arc::new(Mutex::new(None)),
        }
    }

    /// Generates augmented PATH with standard binary locations for the current platform
    pub fn augmented_path() -> String {
        let current_path = std::env::var("PATH").unwrap_or_default();

        #[cfg(target_os = "windows")]
        {
            let userprofile = std::env::var("USERPROFILE").unwrap_or_default();
            let localappdata = std::env::var("LOCALAPPDATA").unwrap_or_default();
            let programfiles = std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
            let rdir = crate::services::runtime::RuntimeManager::runtime_dir();
            let extra_paths = [
                rdir.join("bin").to_string_lossy().to_string(),
                rdir.join("llama").to_string_lossy().to_string(),
                rdir.join("venv\\Scripts").to_string_lossy().to_string(),
                format!("{}\\Ollama", localappdata),
                format!("{}\\Programs\\Ollama", localappdata),
                format!("{}\\.cargo\\bin", userprofile),
                format!("{}\\llama.cpp", programfiles),
            ];
            let mut combined = extra_paths.join(";");
            if !current_path.is_empty() {
                combined.push(';');
                combined.push_str(&current_path);
            }
            combined
        }

        #[cfg(not(target_os = "windows"))]
        {
            let home = std::env::var("HOME").unwrap_or_default();
            let rdir = crate::services::runtime::RuntimeManager::runtime_dir();
            let mut extra_paths = vec![
                rdir.join("bin").to_string_lossy().to_string(),
                rdir.join("llama").to_string_lossy().to_string(),
                rdir.join("venv/bin").to_string_lossy().to_string(),
                "/opt/homebrew/bin".to_string(),
                "/opt/homebrew/sbin".to_string(),
                "/usr/local/bin".to_string(),
                "/usr/bin".to_string(),
                "/bin".to_string(),
                "/usr/sbin".to_string(),
                "/sbin".to_string(),
                format!("{}/.gemini/antigravity-cli/bin", home),
                format!("{}/.gemini/antigravity-cli", home),
                format!("{}/.gemini/bin", home),
                format!("{}/.gemini/antigravity-ide/bin", home),
                format!("{}/.antigravity/bin", home),
                format!("{}/.local/bin", home),
                format!("{}/.cargo/bin", home),
                format!("{}/.bun/bin", home),
                format!("{}/.volta/bin", home),
                format!("{}/.fnm/current/bin", home),
                format!("{}/Library/pnpm", home),
                format!("{}/.local/share/pnpm", home),
            ];

            // Add nvm node versions if present
            let nvm_versions_dir = std::path::PathBuf::from(&home).join(".nvm/versions/node");
            if let Ok(entries) = std::fs::read_dir(nvm_versions_dir) {
                for entry in entries.flatten() {
                    let bin_path = entry.path().join("bin");
                    if bin_path.exists() {
                        if let Some(p) = bin_path.to_str() {
                            extra_paths.push(p.to_string());
                        }
                    }
                }
            }

            let mut combined = extra_paths.join(":");
            if !current_path.is_empty() {
                combined.push(':');
                combined.push_str(&current_path);
            }
            combined
        }
    }

    /// Checks if Antigravity CLI is installed and has an active cloud session
    pub async fn check_agy_session() -> AgySessionStatus {
        let bin_opt = Self::find_agy_binary();
        if bin_opt.is_none() {
            return AgySessionStatus {
                installed: false,
                binary_path: None,
                authenticated: false,
                active_account: None,
                available_models: Vec::new(),
                error_message: Some("Antigravity CLI (agy) não encontrado no PATH do sistema.".to_string()),
            };
        }

        let bin_path = bin_opt.unwrap();
        let bin_str = bin_path.to_string_lossy().to_string();

        // Run `agy models` with adequate timeout to confirm live auth and fetch available models
        let mut cmd = silent_tokio_command(&bin_path);
        cmd.env("PATH", Self::augmented_path());
        cmd.arg("models");
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let result = tokio::time::timeout(Duration::from_secs(12), async {
            match cmd.output().await {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    (output.status.success(), stdout, stderr)
                }
                Err(e) => (false, String::new(), e.to_string()),
            }
        }).await;

        // Resolve active account dynamically from the latest Antigravity CLI session log or credentials
        let active_account = Self::find_active_agy_account().await;

        let fallback_models = vec![
            "gemini-3.8-flash-high".to_string(),
            "gemini-3.8-flash-medium".to_string(),
            "gemini-3.8-flash-low".to_string(),
            "gemini-3.7-flash-high".to_string(),
            "gemini-3.7-flash-medium".to_string(),
            "claude-sonnet-4-6".to_string(),
            "claude-opus-4-6-thinking".to_string(),
            "gpt-oss-120b-medium".to_string(),
        ];

        match result {
            Ok((true, stdout, _)) => {
                let mut models = Vec::new();
                for line in stdout.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with("Fetching") {
                        continue;
                    }
                    if let Some(first_col) = trimmed.split_whitespace().next() {
                        if first_col.contains("gemini") || first_col.contains("claude") || first_col.contains("gpt") {
                            models.push(first_col.to_string());
                        }
                    }
                }

                let is_auth = !models.is_empty() || stdout.contains("gemini") || stdout.contains("claude") || active_account.is_some();

                AgySessionStatus {
                    installed: true,
                    binary_path: Some(bin_str),
                    authenticated: is_auth,
                    active_account,
                    available_models: if models.is_empty() && is_auth { fallback_models } else { models },
                    error_message: if is_auth { None } else { Some("Sessão requer autenticação no Antigravity CLI.".to_string()) },
                }
            }
            Ok((false, stdout, stderr)) => {
                let combined_err = if !stderr.is_empty() { stderr } else { stdout };
                let is_auth = active_account.is_some();
                AgySessionStatus {
                    installed: true,
                    binary_path: Some(bin_str),
                    authenticated: is_auth,
                    active_account,
                    available_models: if is_auth { fallback_models } else { Vec::new() },
                    error_message: if is_auth { None } else { Some(combined_err.trim().to_string()) },
                }
            }
            Err(_) => {
                let is_auth = active_account.is_some();
                AgySessionStatus {
                    installed: true,
                    binary_path: Some(bin_str),
                    authenticated: is_auth,
                    active_account,
                    available_models: if is_auth { fallback_models } else { Vec::new() },
                    error_message: if is_auth { None } else { Some("Tempo limite excedido ao testar a sessão do Antigravity.".to_string()) },
                }
            }
        }
    }

    /// Queries quota usage and remaining limits from Antigravity CLI (`agy --output-format json -p="/usage"`)
    pub async fn get_agy_usage() -> Result<AgyUsageData, String> {
        let bin_path = Self::find_agy_binary()
            .ok_or_else(|| "Executável do Antigravity (`agy`) não encontrado no sistema.".to_string())?;

        let mut cmd = silent_tokio_command(&bin_path);
        cmd.env("PATH", Self::augmented_path());
        cmd.args(["--output-format", "json", "-p=/usage"]);
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let result = tokio::time::timeout(Duration::from_secs(12), async {
            match cmd.output().await {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    (output.status.success(), stdout, stderr)
                }
                Err(e) => (false, String::new(), e.to_string()),
            }
        }).await;

        match result {
            Ok((true, stdout, _)) => {
                // Parse either direct JSON or JSON Lines format
                let mut target_val: Option<serde_json::Value> = None;
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&stdout) {
                    target_val = Some(v);
                } else {
                    for line in stdout.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with('{') {
                            if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
                                target_val = Some(v);
                                break;
                            }
                        }
                    }
                }

                if let Some(json) = target_val {
                    let usage_obj = json
                        .get("command")
                        .and_then(|c| c.get("data"))
                        .or_else(|| json.get("data"))
                        .unwrap_or(&json);

                    if let Ok(usage_data) = serde_json::from_value::<AgyUsageData>(usage_obj.clone()) {
                        return Ok(usage_data);
                    }
                }

                Err("Formato de resposta inesperado do comando /usage do Antigravity.".to_string())
            }
            Ok((false, stdout, stderr)) => {
                let err_msg = if !stderr.is_empty() { stderr } else { stdout };
                Err(format!("Falha ao consultar limites do Antigravity: {}", err_msg.trim()))
            }
            Err(_) => Err("Tempo limite excedido ao consultar o uso do Antigravity.".to_string()),
        }
    }

    /// Launches a native terminal window running a specific shell command
    pub fn launch_terminal_command(command: &str) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            let escaped = command.replace('\\', "\\\\").replace('"', "\\\"");
            let script = format!("tell application \"Terminal\" to do script \"{}\"", escaped);
            let activate = "tell application \"Terminal\" to activate";
            let res = std::process::Command::new("osascript")
                .arg("-e")
                .arg(&script)
                .arg("-e")
                .arg(activate)
                .spawn();
            return match res {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Falha ao abrir o Terminal: {}", e)),
            };
        }

        #[cfg(target_os = "windows")]
        {
            let res = std::process::Command::new("cmd")
                .args(["/c", "start", "wt", "cmd", "/k", command, "||", "start", "cmd", "/k", command])
                .spawn();
            return match res {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Falha ao abrir terminal no Windows: {}", e)),
            };
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            for term in &["x-terminal-emulator", "gnome-terminal", "konsole", "xterm"] {
                if let Ok(_) = std::process::Command::new(term)
                    .args(["-e", &format!("bash -c '{} ; exec bash'", command)])
                    .spawn()
                {
                    return Ok(());
                }
            }
            Err("Nenhum emulador de terminal compatível encontrado.".to_string())
        }
    }

    /// Launches a native terminal window running `agy` for interactive login
    pub fn launch_agy_login_terminal() -> Result<(), String> {
        Self::launch_terminal_command("agy")
    }

    /// Dynamically locates the Antigravity (`agy`) CLI binary on the system
    pub fn find_agy_binary() -> Option<std::path::PathBuf> {
        let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap_or_default();
        let bin_name = if cfg!(target_os = "windows") { "agy.exe" } else { "agy" };

        // 1. Direct candidate paths based on standard Antigravity CLI installation directories
        if !home.is_empty() {
            let home_path = std::path::PathBuf::from(&home);
            let candidates = [
                home_path.join(".gemini/antigravity-cli/bin").join(bin_name),
                home_path.join(".gemini/antigravity-cli").join(bin_name),
                home_path.join(".gemini/bin").join(bin_name),
                home_path.join(".gemini/antigravity-ide/bin").join(bin_name),
                home_path.join(".antigravity/bin").join(bin_name),
                home_path.join(".local/bin").join(bin_name),
                home_path.join(".cargo/bin").join(bin_name),
                home_path.join("bin").join(bin_name),
            ];
            for c in &candidates {
                if c.is_file() {
                    return Some(c.clone());
                }
            }
        }

        // 2. System and Application bundles (macOS)
        #[cfg(target_os = "macos")]
        {
            let mac_candidates = [
                std::path::PathBuf::from("/Applications/Antigravity.app/Contents/Resources/app/bin").join(bin_name),
                std::path::PathBuf::from("/Applications/Antigravity.app/Contents/Resources/bin").join(bin_name),
                std::path::PathBuf::from("/Applications/Antigravity.app/Contents/Resources/antigravity-cli/bin").join(bin_name),
                std::path::PathBuf::from("/Applications/Antigravity.app/Contents/MacOS").join(bin_name),
                std::path::PathBuf::from("/opt/homebrew/bin").join(bin_name),
                std::path::PathBuf::from("/usr/local/bin").join(bin_name),
                std::path::PathBuf::from("/usr/bin").join(bin_name),
            ];
            for c in &mac_candidates {
                if c.is_file() {
                    return Some(c.clone());
                }
            }
        }

        // 3. Search in all directories in augmented_path()
        let aug_path = Self::augmented_path();
        for dir in std::env::split_paths(&aug_path) {
            let candidate = dir.join(bin_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }

        // 4. Fallback: which agy (Unix) or where agy.exe (Windows)
        #[cfg(not(target_os = "windows"))]
        {
            if let Ok(out) = std::process::Command::new("which").arg("agy").output() {
                if out.status.success() {
                    let path_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if !path_str.is_empty() {
                        let p = std::path::PathBuf::from(path_str);
                        if p.is_file() {
                            return Some(p);
                        }
                    }
                }
            }
            if let Ok(out) = std::process::Command::new("/bin/zsh").args(&["-l", "-c", "which agy"]).output() {
                if out.status.success() {
                    let path_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if !path_str.is_empty() {
                        let p = std::path::PathBuf::from(path_str);
                        if p.is_file() {
                            return Some(p);
                        }
                    }
                }
            }
        }

        None
    }

    /// Locates the real active account email used by Antigravity CLI
    pub async fn find_active_agy_account() -> Option<String> {
        let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap_or_default();
        if home.is_empty() {
            return None;
        }

        let base_gemini = std::path::PathBuf::from(&home).join(".gemini");
        let agy_cli_dir = base_gemini.join("antigravity-cli");

        // 1. Try reading the active CLI log (cli.log symlink or latest cli-*.log)
        let mut log_candidates = Vec::new();
        let symlink_log = agy_cli_dir.join("cli.log");
        if symlink_log.exists() {
            log_candidates.push(symlink_log);
        }

        let logs_dir = agy_cli_dir.join("log");
        if let Ok(mut entries) = tokio::fs::read_dir(&logs_dir).await {
            let mut file_entries = Vec::new();
            while let Ok(Some(entry)) = entries.next_entry().await {
                let p = entry.path();
                if p.extension().map(|e| e == "log").unwrap_or(false) {
                    if let Ok(meta) = entry.metadata().await {
                        if let Ok(modified) = meta.modified() {
                            file_entries.push((modified, p));
                        }
                    }
                }
            }
            file_entries.sort_by(|a, b| b.0.cmp(&a.0));
            for (_, p) in file_entries.into_iter().take(6) {
                log_candidates.push(p);
            }
        }

        for log_path in log_candidates {
            if let Ok(bytes) = tokio::fs::read(&log_path).await {
                let slice = if bytes.len() > 65536 { &bytes[..65536] } else { &bytes[..] };
                let content = String::from_utf8_lossy(slice);

                // Pattern 1: "authenticated successfully as <email>"
                if let Some(pos) = content.find("authenticated successfully as ") {
                    let rem = &content[pos + 30..];
                    if let Some(end) = rem.find(|c: char| c.is_whitespace() || c == ',' || c == '"' || c == '\n') {
                        let email = rem[..end].trim();
                        if email.contains('@') {
                            return Some(email.to_string());
                        }
                    }
                }

                // Pattern 2: "applyAuthResult: email=<email>"
                if let Some(pos) = content.find("email=") {
                    let rem = &content[pos + 6..];
                    if let Some(end) = rem.find(|c: char| c.is_whitespace() || c == ',' || c == '"' || c == '\n') {
                        let email = rem[..end].trim();
                        if email.contains('@') {
                            return Some(email.to_string());
                        }
                    }
                }
            }
        }

        // 2. Fallback: ~/.gemini/google_accounts.json
        let acct_file = base_gemini.join("google_accounts.json");
        if let Ok(content) = tokio::fs::read_to_string(&acct_file).await {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(act) = val.get("active").and_then(|v| v.as_str()) {
                    if !act.is_empty() && act.contains('@') {
                        return Some(act.to_string());
                    }
                }
            }
        }

        None
    }

    /// Checks if MLX-LM server is healthy and responding
    pub async fn check_mlx_health(host: &str, port: u16) -> bool {
        let url = format!("http://{}:{}/v1/models", host, port);
        let client = match reqwest::Client::builder()
            .timeout(Duration::from_millis(600))
            .build()
        {
            Ok(c) => c,
            Err(_) => return false,
        };

        client.get(&url).send().await.map(|r| r.status().is_success()).unwrap_or(false)
    }

    /// Checks if Ollama service is healthy and responding
    pub async fn check_ollama_health(host: &str, port: u16) -> bool {
        let url = format!("http://{}:{}/api/version", host, port);
        let client = match reqwest::Client::builder()
            .timeout(Duration::from_millis(600))
            .build()
        {
            Ok(c) => c,
            Err(_) => return false,
        };

        client.get(&url).send().await.map(|r| r.status().is_success()).unwrap_or(false)
    }

    /// Checks if a model path contains a VLM or Gemma 4 unified architecture
    pub fn is_vlm_or_unified_model(model_path: &str) -> bool {
        let path = std::path::Path::new(model_path);
        let config_path = path.join("config.json");
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(m_type) = val.get("model_type").and_then(|v| v.as_str()) {
                    let m_lower = m_type.to_lowercase();
                    if m_lower.contains("gemma4") || m_lower.contains("vl") || m_lower.contains("vision") || m_lower.contains("paligemma") {
                        return true;
                    }
                }
                if let Some(archs) = val.get("architectures").and_then(|v| v.as_array()) {
                    for arch in archs {
                        if let Some(a) = arch.as_str() {
                            let a_lower = a.to_lowercase();
                            if a_lower.contains("gemma4") || a_lower.contains("vl") || a_lower.contains("vision") || a_lower.contains("paligemma") {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        if path.join("processor_config.json").exists() {
            return true;
        }
        false
    }

    /// Launches MLX-LM / MLX-VLM server for a given model path and waits for it to become ready.
    /// Only available on macOS Apple Silicon.
    pub async fn start_mlx_server(
        &self,
        model_path: &str,
        host: &str,
        port: u16,
        progress_cb: Option<ProgressCallback>,
    ) -> Result<(), String> {
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (model_path, host, port, progress_cb);
            return Err("MLX requer macOS com Apple Silicon. Use Ollama ou llama-server nesta plataforma.".to_string());
        }

        #[cfg(target_os = "macos")]
        {
            let mut proc_guard = self.mlx_process.lock().await;
            if let Some(mut existing) = proc_guard.take() {
                let _ = existing.kill().await;
            }

            let aug_path = Self::augmented_path();

            // Garantir que ~/.cache/huggingface/hub existe para evitar CacheNotFound no mlx_lm.server
            let home = std::env::var("HOME").unwrap_or_default();
            let hf_cache_dir = if !home.is_empty() {
                let p = std::path::PathBuf::from(&home).join(".cache/huggingface/hub");
                let _ = tokio::fs::create_dir_all(&p).await;
                p.to_string_lossy().to_string()
            } else {
                String::new()
            };

            // Kill any orphaned mlx_lm.server or mlx_vlm.server process holding the port
            let _ = Command::new("pkill").env("PATH", &aug_path).arg("-9").arg("-f").arg("mlx_lm.server").output().await;
            let _ = Command::new("pkill").env("PATH", &aug_path).arg("-9").arg("-f").arg("mlx_vlm.server").output().await;
            tokio::time::sleep(Duration::from_millis(400)).await;

            let is_vlm = Self::is_vlm_or_unified_model(model_path);
            let module_name = if is_vlm { "mlx_vlm.server" } else { "mlx_lm.server" };

            let tracker = LoadProgressTracker::new(progress_cb, "Carregando pesos do modelo MLX...");

            let child = if let Some(py_path) = crate::services::runtime::RuntimeManager::isolated_python() {
                let mut cmd = Command::new(&py_path);
                cmd.env("PATH", &aug_path);
                if !hf_cache_dir.is_empty() {
                    cmd.env("HF_HUB_CACHE", &hf_cache_dir);
                }
                cmd.arg("-m")
                    .arg(module_name)
                    .arg("--model")
                    .arg(model_path)
                    .arg("--host")
                    .arg(host)
                    .arg("--port")
                    .arg(port.to_string());
                if is_vlm {
                    cmd.arg("--enable-thinking");
                } else {
                    cmd.arg("--prompt-cache-size").arg("1");
                    cmd.arg("--chat-template-args").arg("{\"enable_thinking\":true}");
                }
                cmd.stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
            } else if let Some(uv_path) = crate::services::runtime::RuntimeManager::resolve_binary("uv") {
                let mut cmd = Command::new(&uv_path);
                cmd.env("PATH", &aug_path)
                    .arg("run")
                    .arg("--with")
                    .arg(if is_vlm { "mlx-vlm" } else { "mlx-lm" })
                    .arg("--with")
                    .arg("jinja2");
                if is_vlm {
                    cmd.arg("--with")
                        .arg("mlx-lm")
                        .arg("python3")
                        .arg("-m")
                        .arg("mlx_vlm.server");
                } else {
                    cmd.arg("mlx_lm.server");
                }
                cmd.arg("--model")
                    .arg(model_path)
                    .arg("--host")
                    .arg(host)
                    .arg("--port")
                    .arg(port.to_string());
                if is_vlm {
                    cmd.arg("--enable-thinking");
                } else {
                    cmd.arg("--prompt-cache-size").arg("1");
                    cmd.arg("--chat-template-args").arg("{\"enable_thinking\":true}");
                }
                cmd.stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
            } else {
                let mut cmd = Command::new("python3");
                cmd.env("PATH", &aug_path)
                    .arg("-m")
                    .arg(module_name)
                    .arg("--model")
                    .arg(model_path)
                    .arg("--host")
                    .arg(host)
                    .arg("--port")
                    .arg(port.to_string());
                if is_vlm {
                    cmd.arg("--enable-thinking");
                } else {
                    cmd.arg("--prompt-cache-size").arg("1");
                    cmd.arg("--chat-template-args").arg("{\"enable_thinking\":true}");
                }
                cmd.stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
                    .or_else(|_| {
                        let mut alt = Command::new(module_name);
                        alt.env("PATH", &aug_path)
                            .arg("--model")
                            .arg(model_path)
                            .arg("--host")
                            .arg(host)
                            .arg("--port")
                            .arg(port.to_string());
                        if is_vlm {
                            alt.arg("--enable-thinking");
                        } else {
                            alt.arg("--prompt-cache-size").arg("1");
                            alt.arg("--chat-template-args").arg("{\"enable_thinking\":true}");
                        }
                        alt.stdout(std::process::Stdio::piped())
                            .stderr(std::process::Stdio::piped());
                        alt.spawn()
                    })
            };

            match child {
                Ok(mut c) => {
                    if let Some(stdout) = c.stdout.take() {
                        tracker.attach_stream(stdout);
                    }
                    if let Some(stderr) = c.stderr.take() {
                        tracker.attach_stream(stderr);
                    }

                    *proc_guard = Some(c);
                    drop(proc_guard);

                    // Wait for the MLX server to load model weights and respond to health checks
                    let start_wait = std::time::Instant::now();
                    let max_wait = Duration::from_secs(90);
                    let poll_interval = Duration::from_millis(400);

                    loop {
                        if Self::check_mlx_health(host, port).await {
                            tracker.finish("Modelo pronto!");
                            return Ok(());
                        }

                        // Check if child exited prematurely with error
                        let mut guard = self.mlx_process.lock().await;
                        if let Some(ref mut proc) = *guard {
                            if let Ok(Some(status)) = proc.try_wait() {
                                tracker.cancel();
                                return Err(format!("O processo MLX encerrou prematuramente com status: {:?}", status));
                            }
                        }
                        drop(guard);

                        if start_wait.elapsed() > max_wait {
                            tracker.cancel();
                            return Err("Tempo limite excedido (90s) ao carregar modelo no servidor MLX.".to_string());
                        }

                        tokio::time::sleep(poll_interval).await;
                    }
                }
                Err(e) => {
                    tracker.cancel();
                    Err(format!("Falha ao iniciar MLX: {}", e))
                }
            }
        }
    }

    /// Launches llama-server for GGUF models with GPU offloading, KV cache quantization, flash attention, and waits for it to become ready.
    /// On macOS uses Metal/DYLD; on Windows uses CUDA paths.
    pub async fn start_llama_server(
        &self,
        model_path: &str,
        host: &str,
        port: u16,
        context_length: usize,
        kv_cache_quant: Option<String>,
        flash_attn: Option<bool>,
        prompt_cache: Option<bool>,
        progress_cb: Option<ProgressCallback>,
    ) -> Result<(), String> {
        let mut proc_guard = self.llama_process.lock().await;
        if let Some(mut existing) = proc_guard.take() {
            let _ = existing.kill().await;
        }

        let aug_path = Self::augmented_path();

        // Kill any orphaned llama-server process holding the port
        #[cfg(target_os = "windows")]
        {
            let _ = silent_tokio_command("taskkill").args(["/F", "/IM", "llama-server.exe"]).output().await;
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = Command::new("pkill").env("PATH", &aug_path).arg("-9").arg("-f").arg("llama-server").output().await;
        }
        tokio::time::sleep(Duration::from_millis(400)).await;

        let ctx_str = if context_length > 0 { context_length.to_string() } else { "8192".to_string() };

        #[cfg(target_os = "windows")]
        let llama_bin = crate::services::runtime::RuntimeManager::resolve_binary("llama-server")
            .unwrap_or_else(|| {
                crate::services::runtime::RuntimeManager::runtime_dir()
                    .join("llama")
                    .join("llama-server.exe")
            });

        #[cfg(not(target_os = "windows"))]
        let llama_bin = crate::services::runtime::RuntimeManager::resolve_binary("llama-server")
            .unwrap_or_else(|| {
                crate::services::runtime::RuntimeManager::runtime_dir()
                    .join("llama")
                    .join("llama-server")
            });

        let mut base_cmd = silent_tokio_command(&llama_bin);
        base_cmd.env("PATH", &aug_path);

        // macOS: set DYLD library paths for Metal acceleration, ensuring llama directory is prioritized
        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME").unwrap_or_default();
            let parent_dir = llama_bin.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
            let rdir_llama = crate::services::runtime::RuntimeManager::runtime_dir().join("llama").to_string_lossy().to_string();
            let dyld_paths = if !parent_dir.is_empty() {
                format!("{}:{}:{}/lib:/opt/homebrew/lib:/usr/local/lib:/opt/homebrew/opt/ggml/lib", parent_dir, rdir_llama, home)
            } else {
                format!("{}:{}/lib:/opt/homebrew/lib:/usr/local/lib:/opt/homebrew/opt/ggml/lib", rdir_llama, home)
            };
            base_cmd.env("DYLD_FALLBACK_LIBRARY_PATH", &dyld_paths)
                    .env("DYLD_LIBRARY_PATH", &dyld_paths);
        }

        let config = crate::core::config::AppConfig::load();
        let threads = config.thread_count.max(1);
        let threads_str = threads.to_string();

        #[cfg(target_os = "macos")]
        let ngl = "99";

        #[cfg(not(target_os = "macos"))]
        let ngl = if crate::core::hardware::SystemHardwareInfo::has_gpu_acceleration() {
            "99"
        } else {
            "0"
        };

        base_cmd
            .arg("-m")
            .arg(model_path)
            .arg("--host")
            .arg(host)
            .arg("--port")
            .arg(port.to_string())
            .arg("-ngl")
            .arg(ngl)
            .arg("-c")
            .arg(&ctx_str)
            .arg("-np")
            .arg("1")
            .arg("-t")
            .arg(&threads_str)
            .arg("--threads-batch")
            .arg(&threads_str)
            .arg("-b")
            .arg("512")
            .arg("-ub")
            .arg("256");

        let q_to_use = match kv_cache_quant {
            Some(ref q) if !q.is_empty() => q.to_lowercase(),
            _ => "q8_0".to_string(),
        };
        if q_to_use != "f16" && q_to_use != "fp16" {
            base_cmd.arg("-ctk").arg(&q_to_use).arg("-ctv").arg(&q_to_use);
        }

        if flash_attn.unwrap_or(true) {
            base_cmd.arg("-fa").arg("on");
        } else {
            base_cmd.arg("-fa").arg("off");
        }

        if prompt_cache.unwrap_or(true) {
            base_cmd.arg("--cache-reuse").arg("256");
        }

        base_cmd.stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());

        let tracker = LoadProgressTracker::new(progress_cb, "Carregando modelo GGUF...");

        let child = base_cmd.spawn().or_else(|_| {
            // Fallback: try plain "llama-server" on PATH
            let mut alt = silent_tokio_command(if cfg!(target_os = "windows") { "llama-server.exe" } else { "llama-server" });
            alt.env("PATH", &aug_path);

            #[cfg(target_os = "macos")]
            {
                let home = std::env::var("HOME").unwrap_or_default();
                let parent_dir = llama_bin.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
                let dyld_paths = if !parent_dir.is_empty() {
                    format!("{}:{}/lib:/opt/homebrew/lib:/usr/local/lib:/opt/homebrew/opt/ggml/lib", parent_dir, home)
                } else {
                    format!("{}/lib:/opt/homebrew/lib:/usr/local/lib:/opt/homebrew/opt/ggml/lib", home)
                };
                alt.env("DYLD_FALLBACK_LIBRARY_PATH", &dyld_paths)
                   .env("DYLD_LIBRARY_PATH", &dyld_paths);
            }

            alt.arg("-m")
                .arg(model_path)
                .arg("--host")
                .arg(host)
                .arg("--port")
                .arg(port.to_string())
                .arg("-ngl")
                .arg(ngl)
                .arg("-c")
                .arg(&ctx_str)
                .arg("-np")
                .arg("1")
                .arg("-t")
                .arg(&threads_str)
                .arg("--threads-batch")
                .arg(&threads_str)
                .arg("-b")
                .arg("512")
                .arg("-ub")
                .arg("256");

            if q_to_use != "f16" && q_to_use != "fp16" {
                alt.arg("-ctk").arg(&q_to_use).arg("-ctv").arg(&q_to_use);
            }

            if flash_attn.unwrap_or(true) {
                alt.arg("-fa").arg("on");
            } else {
                alt.arg("-fa").arg("off");
            }

            if prompt_cache.unwrap_or(true) {
                alt.arg("--cache-reuse").arg("256");
            }

            alt.stdout(std::process::Stdio::piped())
               .stderr(std::process::Stdio::piped());

            alt.spawn()
        });

        match child {
            Ok(mut c) => {
                if let Some(stdout) = c.stdout.take() {
                    tracker.attach_stream(stdout);
                }
                if let Some(stderr) = c.stderr.take() {
                    tracker.attach_stream(stderr);
                }

                *proc_guard = Some(c);
                drop(proc_guard);

                let start_wait = std::time::Instant::now();
                let max_wait = Duration::from_secs(90);
                let poll_interval = Duration::from_millis(400);

                loop {
                    if Self::check_mlx_health(host, port).await {
                        tracker.finish("Modelo GGUF pronto!");
                        return Ok(());
                    }

                    let mut guard = self.llama_process.lock().await;
                    if let Some(ref mut proc) = *guard {
                        if let Ok(Some(status)) = proc.try_wait() {
                            tracker.cancel();
                            return Err(format!("O processo llama-server encerrou prematuramente com status: {:?}", status));
                        }
                    }
                    drop(guard);

                    if start_wait.elapsed() > max_wait {
                        tracker.cancel();
                        return Err("Tempo limite excedido (90s) ao carregar modelo GGUF no llama-server.".to_string());
                    }

                    tokio::time::sleep(poll_interval).await;
                }
            }
            Err(e) => {
                tracker.cancel();
                Err(format!("Falha ao iniciar llama-server: {}. Verifique se o llama.cpp está instalado.", e))
            }
        }
    }

    /// Launches Ollama service (`ollama serve`)
    pub async fn start_ollama_serve(&self, progress_cb: Option<ProgressCallback>) -> Result<(), String> {
        let mut proc_guard = self.ollama_process.lock().await;
        if proc_guard.is_some() {
            if let Some(ref cb) = progress_cb {
                cb(ModelLoadProgress {
                    percent: 100,
                    message: "Ollama pronto!".to_string(),
                });
            }
            return Ok(());
        }

        let tracker = LoadProgressTracker::new(progress_cb, "Iniciando serviço Ollama...");

        let aug_path = Self::augmented_path();
        let ollama_bin = if cfg!(target_os = "windows") { "ollama.exe" } else { "ollama" };
        let mut cmd = Command::new(ollama_bin);
        cmd.env("PATH", &aug_path);

        #[cfg(not(target_os = "macos"))]
        {
            if !crate::core::hardware::SystemHardwareInfo::has_gpu_acceleration() {
                cmd.env("OLLAMA_NUM_PARALLEL", "1");
            }
        }

        match cmd
            .arg("serve")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(mut c) => {
                if let Some(stdout) = c.stdout.take() {
                    tracker.attach_stream(stdout);
                }
                if let Some(stderr) = c.stderr.take() {
                    tracker.attach_stream(stderr);
                }
                *proc_guard = Some(c);
                tracker.finish("Ollama iniciado com sucesso!");
                Ok(())
            }
            Err(e) => {
                tracker.cancel();
                Err(format!("Falha ao executar 'ollama serve': {}. Verifique se o Ollama está no PATH.", e))
            }
        }
    }

    /// Instructs Ollama to unload all models currently occupying RAM/VRAM asynchronously
    pub async fn unload_ollama_models_async() {
        if let Ok(client) = reqwest::Client::builder().timeout(Duration::from_millis(600)).build() {
            if let Ok(resp) = client.get("http://127.0.0.1:11434/api/ps").send().await {
                if let Ok(val) = resp.json::<serde_json::Value>().await {
                    if let Some(models) = val.get("models").and_then(|m| m.as_array()) {
                        for m in models {
                            if let Some(name) = m.get("name").and_then(|n| n.as_str()) {
                                let _ = client
                                    .post("http://127.0.0.1:11434/api/generate")
                                    .json(&serde_json::json!({
                                        "model": name,
                                        "keep_alive": 0
                                    }))
                                    .send()
                                    .await;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Sync stub to satisfy sync callers without spawning blocking runtimes inside Tokio
    pub fn unload_ollama_models_sync() {
        // No-op to prevent Tokio runtime panics
    }

    /// Synchronously and forcefully terminates active MLX, llama-server, and Ollama processes spawned by Atena,
    /// and unloads any in-memory LLM models from Ollama to release unified memory / RAM.
    pub fn stop_all_sync(&self) {
        if let Ok(mut mlx) = self.mlx_process.try_lock() {
            if let Some(mut proc) = mlx.take() {
                let _ = proc.start_kill();
            }
        }
        if let Ok(mut llama) = self.llama_process.try_lock() {
            if let Some(mut proc) = llama.take() {
                let _ = proc.start_kill();
            }
        }

        let ollama_was_spawned = if let Ok(mut ollama) = self.ollama_process.try_lock() {
            if let Some(mut proc) = ollama.take() {
                let _ = proc.start_kill();
                true
            } else {
                false
            }
        } else {
            false
        };

        // Release loaded models from Ollama memory
        Self::unload_ollama_models_sync();

        #[cfg(target_os = "windows")]
        {
            let _ = silent_command("taskkill")
                .args(["/F", "/IM", "llama-server.exe"])
                .output();
            if ollama_was_spawned {
                let _ = silent_command("taskkill")
                    .args(["/F", "/IM", "ollama.exe"])
                    .output();
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            let aug_path = Self::augmented_path();
            let _ = std::process::Command::new("pkill")
                .env("PATH", &aug_path)
                .arg("-9")
                .arg("-f")
                .arg("mlx_lm.server")
                .output();
            let _ = std::process::Command::new("pkill")
                .env("PATH", &aug_path)
                .arg("-9")
                .arg("-f")
                .arg("mlx_vlm.server")
                .output();
            let _ = std::process::Command::new("pkill")
                .env("PATH", &aug_path)
                .arg("-9")
                .arg("-f")
                .arg("llama-server")
                .output();
            if ollama_was_spawned {
                let _ = std::process::Command::new("pkill")
                    .env("PATH", &aug_path)
                    .arg("-9")
                    .arg("-f")
                    .arg("ollama serve")
                    .output();
                let _ = std::process::Command::new("pkill")
                    .env("PATH", &aug_path)
                    .arg("-9")
                    .arg("-f")
                    .arg("ollama_llama_server")
                    .output();
            }
        }
    }

    /// Terminates active MLX, llama-server and Ollama processes spawned by Atena
    pub async fn stop_all(&self) {
        Self::unload_ollama_models_async().await;
        self.stop_all_sync();
    }

    /// Formats an instruction block with active MCP tools schema to inject into the model's system prompt
    /// Formats an instruction block with active MCP tools schema to inject into the model's system prompt
    pub fn format_mcp_tools_system_prompt(tools: &[crate::core::mcp::McpToolWithServer]) -> String {
        let active_tools: Vec<_> = tools.iter().filter(|t| t.enabled).collect();
        if active_tools.is_empty() {
            return String::new();
        }

        let mut prompt = String::new();
        prompt.push_str("\n\n# Available External Tools (Model Context Protocol - MCP)\n");
        prompt.push_str("You have direct access to the external tools listed below to query and interact with external systems (such as financial records, bank statements, accounts, files, and internet services). When the user asks a question that requires external data, YOU MUST CALL THE APPROPRIATE TOOL instead of claiming you lack access.\n\n");
        prompt.push_str("## Critical Tool Calling Rules:\n");
        prompt.push_str("1. **Strict Parameters**: Only use parameter names and types explicitly defined in each tool's JSON Schema. Never invent or guess non-existent parameter names (e.g., do not invent date parameters if only months are supported).\n");
        prompt.push_str("2. **Avoid Redundant Calls**: If the required data was already retrieved in previous tool calls in this conversation history, DO NOT call the tool again. Use the existing data in the conversation history to answer.\n");
        prompt.push_str("3. **Tool Results Integration**: When you receive a `[MCP Tools Results]` or `[Retorno das Ferramentas MCP]` block, provide your final response directly to the user in natural language, integrating and analyzing the returned data clearly.\n");
        prompt.push_str("4. **Batch & Parallel Calling (Multi-Tool Grouping)**: When fulfilling a request requires multiple queries, actions, or data points (e.g., checking multiple accounts, reading different files, comparing periods, or querying independent data sources), **EMIT ALL `<tool_call>` BLOCKS SEQUENTIALLY IN THE SAME RESPONSE**. Atena Studio automatically intercepts, groups, and executes all consecutive tool calls together in a single batch. DO NOT call tools one by one across multiple back-and-forth turns when multiple calls can be issued up-front.\n\n");

        for t in &active_tools {
            prompt.push_str(&format!("### Tool: `{}` (Server: {})\n", t.tool.name, t.server_name));
            if let Some(desc) = &t.tool.description {
                prompt.push_str(&format!("Description: {}\n", desc));
            }
            let schema_str = serde_json::to_string(&t.tool.input_schema).unwrap_or_else(|_| "{}".to_string());
            prompt.push_str(&format!("Parameters (JSON Schema): `{}`\n\n", schema_str));
        }

        prompt.push_str("# Tool Call Format:\n");
        prompt.push_str("To invoke a tool, respond with a `<tool_call>` block containing JSON with the tool name and arguments:\n");
        prompt.push_str("<tool_call>\n{\n  \"name\": \"tool_name\",\n  \"arguments\": {\n    \"param1\": \"value\"\n  }\n}\n</tool_call>\n\n");
        prompt.push_str("### Calling Multiple Tools in Batch:\n");
        prompt.push_str("To invoke multiple tools in a single turn, emit multiple `<tool_call>` blocks sequentially:\n");
        prompt.push_str("<tool_call>\n{\n  \"name\": \"first_tool\",\n  \"arguments\": { ... }\n}\n</tool_call>\n");
        prompt.push_str("<tool_call>\n{\n  \"name\": \"second_tool\",\n  \"arguments\": { ... }\n}\n</tool_call>\n");
        prompt.push_str("Atena Studio will group them together in the interface, allowing the user to review and approve all tools in batch, and provide all results back to you in a single consolidated block.\n");

        prompt
    }

    /// Formats a block of tool call results for LLM consumption
    pub fn format_tool_results_block(tool_calls: &[crate::core::mcp::McpToolCall]) -> String {
        let mut blocks = Vec::new();
        for tc in tool_calls {
            let status = tc.status.as_deref().unwrap_or("completed");
            match status {
                "completed" => {
                    let res_str = match &tc.result {
                        Some(val) => {
                            if let Some(s) = val.as_str() {
                                s.to_string()
                            } else {
                                serde_json::to_string_pretty(val).unwrap_or_default()
                            }
                        }
                        None => "{}".to_string(),
                    };
                    blocks.push(format!("### Tool '{}':\n```json\n{}\n```", tc.name, res_str));
                }
                "rejected" => {
                    if let Some(ref reason) = tc.rejection_reason {
                        if !reason.trim().is_empty() {
                            blocks.push(format!("### Tool '{}':\n[Execution rejected by user. Reason / Feedback: {}]", tc.name, reason.trim()));
                        } else {
                            blocks.push(format!("### Tool '{}':\n[Execution rejected by user]", tc.name));
                        }
                    } else {
                        blocks.push(format!("### Tool '{}':\n[Execution rejected by user]", tc.name));
                    }
                }
                _ => {
                    let err_str = match &tc.result {
                        Some(val) => {
                            if let Some(s) = val.as_str() {
                                s.to_string()
                            } else {
                                serde_json::to_string(val).unwrap_or_else(|_| "Unknown error".to_string())
                            }
                        }
                        None => "Unknown error".to_string(),
                    };
                    blocks.push(format!("### Tool '{}':\n[Execution error: {}]", tc.name, err_str));
                }
            }
        }
        if blocks.is_empty() {
            return String::new();
        }
        format!(
            "[MCP Tools Results]:\n\n{}\n\nPlease answer the user's request by clearly integrating and correlating the data returned above with all previous context from this conversation.",
            blocks.join("\n\n")
        )
    }

    /// Resolves and builds a standardized McpToolCall, matching available MCP servers and procedural skills
    pub fn resolve_tool_call(
        name: &str,
        arguments: serde_json::Value,
        status: &str,
        available_tools: Option<&[crate::core::mcp::McpToolWithServer]>,
        call_count: usize,
    ) -> crate::core::mcp::McpToolCall {
        let canonical_name = match name {
            "run_skill_command" => "run_command",
            "edit_procedural_skill" => "update_procedural_skill",
            other => other,
        };

        let call_id = format!("call_{}", call_count + 1);
        let mut tc = crate::core::mcp::McpToolCall {
            id: call_id,
            name: canonical_name.to_string(),
            arguments,
            server_id: None,
            server_name: None,
            permission_mode: Some("ask".to_string()),
            status: Some(status.to_string()),
            rejection_reason: None,
            result: None,
            label: None,
            field_labels: std::collections::HashMap::new(),
        };
        if let Some(tools) = available_tools {
            if let Some(matched) = tools.iter().find(|t| t.tool.name == canonical_name || t.tool.name == name) {
                tc.server_id = Some(matched.server_id.clone());
                tc.server_name = Some(matched.server_name.clone());
                tc.permission_mode = Some(matched.permission_mode.clone());
                tc.label = matched.tool.label.clone();
                tc.field_labels = matched.tool.field_labels.clone();
            }
        }
        if tc.server_id.is_none()
            && (tc.name == "run_command"
                || tc.name == "run_skill_command"
                || tc.name == "run_skill_script"
                || tc.name == "create_procedural_skill"
                || tc.name == "update_procedural_skill"
                || tc.name == "edit_procedural_skill")
        {
            tc.server_id = Some("skills".to_string());
            tc.server_name = Some("Procedural Skills".to_string());
        }

        if tc.server_id.is_none()
            && (tc.name == "atena_search_memory"
                || tc.name == "atena_search_episodes"
                || tc.name == "atena_read_episode")
        {
            tc.server_id = Some("atena_native".to_string());
            tc.server_name = Some("Atena Core (Memory & Episodes)".to_string());
            tc.permission_mode = Some("auto".to_string());
        }

        // Check if matching procedural skill defines auto permission
        if tc.server_id.as_deref() == Some("skills")
            || tc.name == "run_command"
            || tc.name == "run_skill_command"
            || tc.name == "run_skill_script"
        {
            let skills = crate::services::memory_engine::MemoryGraphEngine::load_skills();
            let cmd_opt = tc.arguments.get("command").or_else(|| tc.arguments.get("cmd")).and_then(|v| v.as_str());
            let slug_opt = tc.arguments.get("slug")
                .or_else(|| tc.arguments.get("skillId"))
                .or_else(|| tc.arguments.get("skill_id"))
                .or_else(|| tc.arguments.get("id"))
                .and_then(|v| v.as_str());
            let script_opt = tc.arguments.get("script_file")
                .or_else(|| tc.arguments.get("script"))
                .or_else(|| tc.arguments.get("script_name"))
                .or_else(|| tc.arguments.get("file_name"))
                .and_then(|v| v.as_str());

            if let Some(matched) = skills.iter().find(|s| {
                if !s.enabled {
                    return false;
                }
                if let Some(slug) = slug_opt {
                    let clean_s_id = s.id.trim_start_matches("skill-");
                    let clean_slug = slug.trim_start_matches("skill-");
                    if s.id.eq_ignore_ascii_case(slug)
                        || s.name.eq_ignore_ascii_case(slug)
                        || clean_s_id.eq_ignore_ascii_case(clean_slug)
                        || s.id == format!("skill-{}", slug)
                        || slug == format!("skill-{}", s.id)
                    {
                        return true;
                    }
                }
                if let Some(cmd) = cmd_opt {
                    if s.steps.iter().any(|st| st.effective_command().as_deref() == Some(cmd)) {
                        return true;
                    }
                }
                if let Some(script) = script_opt {
                    let clean = script.trim_start_matches('/').trim_start_matches("scripts/");
                    if s.scripts.iter().any(|sc| sc.eq_ignore_ascii_case(clean) || sc.eq_ignore_ascii_case(script))
                        || s.steps.iter().any(|st| st.script_file.as_deref().map(|sf| sf.trim_start_matches('/').trim_start_matches("scripts/")).eq(&Some(clean)))
                    {
                        return true;
                    }
                }
                false
            }) {
                tc.permission_mode = Some(matched.permission_mode.clone());
            }
        }

        // Fallback: If tool name matches an existing procedural skill by name/slug, automatically resolve to run_command or run_skill_script
        if tc.server_id.is_none() {
            let skills = crate::services::memory_engine::MemoryGraphEngine::load_skills();
            let norm_name = name.to_lowercase().replace(['_', '-'], " ");
            if let Some(matched_skill) = skills.iter().find(|s| {
                s.enabled
                    && (s.name.eq_ignore_ascii_case(name)
                        || s.id.eq_ignore_ascii_case(name)
                        || s.name.to_lowercase().replace(['_', '-'], " ") == norm_name)
            }) {
                if let Some(cmd) = matched_skill.steps.iter().find_map(|st| st.effective_command()) {
                    tc.name = "run_command".to_string();
                    tc.arguments = serde_json::json!({
                        "command": cmd,
                        "skill_name": matched_skill.name
                    });
                    tc.server_id = Some("skills".to_string());
                    tc.server_name = Some("Procedural Skills".to_string());
                    tc.label = Some(format!("{} (`{}`)", matched_skill.name, cmd));
                    tc.permission_mode = Some(matched_skill.permission_mode.clone());
                } else if let Some(script_step) = matched_skill.steps.iter().find(|st| st.script_file.is_some()) {
                    let script_file = script_step.script_file.clone().unwrap_or_default();
                    tc.name = "run_skill_script".to_string();
                    tc.arguments = serde_json::json!({
                        "slug": matched_skill.id,
                        "script_file": script_file,
                        "args": []
                    });
                    tc.server_id = Some("skills".to_string());
                    tc.server_name = Some("Procedural Skills".to_string());
                    tc.label = Some(format!("{} (`{}`)", matched_skill.name, script_file));
                    tc.permission_mode = Some(matched_skill.permission_mode.clone());
                } else {
                    tc.server_id = Some("skills".to_string());
                    tc.server_name = Some("Procedural Skills".to_string());
                    tc.permission_mode = Some(matched_skill.permission_mode.clone());
                }
            }
        }

        tc
    }

    /// Extracts tool calls from streaming delta or tagged content text (<tool_call>...</tool_call>, <execute_command>...</execute_command>, or call:name{...})
    pub fn extract_tool_calls(
        raw_content: &str,
        delta_tool_calls: Option<&Vec<serde_json::Value>>,
        available_tools: Option<&[crate::core::mcp::McpToolWithServer]>,
    ) -> (String, Option<Vec<crate::core::mcp::McpToolCall>>) {
        let mut calls = Vec::new();

        // 1. Check native tool_calls array (from OpenAI delta or Ollama message)
        if let Some(dt_list) = delta_tool_calls {
            // Group and merge streamed delta chunks by tool call index/id
            let mut merged_calls: std::collections::BTreeMap<usize, (String, String, String, serde_json::Value)> = std::collections::BTreeMap::new();

            for (idx, dt) in dt_list.iter().enumerate() {
                let call_idx = dt.get("index").and_then(|i| i.as_u64()).map(|i| i as usize).unwrap_or(idx);
                let id = dt.get("id").and_then(|i| i.as_str()).unwrap_or("").to_string();
                let name = dt.get("function")
                    .and_then(|f| f.get("name"))
                    .or_else(|| dt.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("")
                    .to_string();

                let raw_args = dt.get("function")
                    .and_then(|f| f.get("arguments"))
                    .or_else(|| dt.get("arguments"));

                let entry = merged_calls.entry(call_idx).or_insert_with(|| (
                    format!("call_{}", call_idx + 1),
                    String::new(),
                    String::new(),
                    serde_json::Value::Null,
                ));

                if !id.is_empty() {
                    entry.0 = id;
                }
                if !name.is_empty() {
                    entry.1 = name;
                }
                if let Some(args_val) = raw_args {
                    match args_val {
                        serde_json::Value::String(s) => {
                            entry.2.push_str(s);
                        }
                        other => {
                            entry.3 = other.clone();
                        }
                    }
                }
            }

            for (_, (id, name, args_str, args_val)) in merged_calls {
                if !name.is_empty() {
                    let arguments = if !args_val.is_null() {
                        args_val
                    } else if !args_str.trim().is_empty() {
                        serde_json::from_str::<serde_json::Value>(&args_str).unwrap_or_else(|_| json!({}))
                    } else {
                        json!({})
                    };

                    let mut tc = Self::resolve_tool_call(&name, arguments, "pending_approval", available_tools, calls.len());
                    tc.id = id;
                    calls.push(tc);
                }
            }
        }

        // 2. Check for <tool_call>...</tool_call> tags in text content
        let mut cleaned_content = raw_content.to_string();
        let search_idx = 0;
        while let Some(start_tag) = cleaned_content[search_idx..].find("<tool_call>") {
            let actual_start = search_idx + start_tag;
            if let Some(end_tag) = cleaned_content[actual_start..].find("</tool_call>") {
                let actual_end = actual_start + end_tag + "</tool_call>".len();
                let inner = &cleaned_content[actual_start + "<tool_call>".len()..actual_start + end_tag].trim();

                if let Ok(parsed_json) = serde_json::from_str::<serde_json::Value>(inner) {
                    let mut name = parsed_json.get("name")
                        .or_else(|| parsed_json.get("tool"))
                        .and_then(|n| n.as_str())
                        .unwrap_or("")
                        .to_string();

                    let mut arguments = parsed_json.get("arguments")
                        .or_else(|| parsed_json.get("parameters"))
                        .cloned()
                        .unwrap_or_else(|| json!({}));

                    // Defensive normalization: if the model omitted the tool wrapper and emitted skill definition directly at root level
                    if name != "create_procedural_skill"
                        && name != "update_procedural_skill"
                        && name != "edit_procedural_skill"
                        && (parsed_json.get("steps").is_some() || parsed_json.get("triggers").is_some())
                        && arguments.as_object().map_or(true, |o| o.is_empty())
                    {
                        arguments = parsed_json.clone();
                        name = if parsed_json.get("id").is_some() || parsed_json.get("refinement_note").is_some() {
                            "update_procedural_skill".to_string()
                        } else {
                            "create_procedural_skill".to_string()
                        };
                    } else if name == "edit_procedural_skill" {
                        name = "update_procedural_skill".to_string();
                    }

                    if !name.is_empty() {
                        let tc = Self::resolve_tool_call(&name, arguments, "pending_approval", available_tools, calls.len());
                        calls.push(tc);
                    }
                }

                cleaned_content.replace_range(actual_start..actual_end, "");
            } else {
                // If there is an unclosed/partial <tool_call> tag (actively streaming tokens),
                // extract any partial tool name and emit a streaming placeholder tool call so the UI
                // provides immediate visual feedback to the user instead of appearing frozen.
                let partial_raw = &cleaned_content[actual_start + "<tool_call>".len()..];
                let mut partial_name = String::new();
                if let Some(name_pos) = partial_raw.find("\"name\"") {
                    let after_name = &partial_raw[name_pos + 6..];
                    if let Some(first_quote) = after_name.find('"') {
                        let val_part = &after_name[first_quote + 1..];
                        if let Some(second_quote) = val_part.find('"') {
                            partial_name = val_part[..second_quote].trim().to_string();
                        }
                    }
                } else if let Some(tool_pos) = partial_raw.find("\"tool\"") {
                    let after_tool = &partial_raw[tool_pos + 6..];
                    if let Some(first_quote) = after_tool.find('"') {
                        let val_part = &after_tool[first_quote + 1..];
                        if let Some(second_quote) = val_part.find('"') {
                            partial_name = val_part[..second_quote].trim().to_string();
                        }
                    }
                }

                let tc = Self::resolve_tool_call(
                    if partial_name.is_empty() { "loading_tool" } else { &partial_name },
                    serde_json::json!({}),
                    "streaming",
                    available_tools,
                    calls.len(),
                );
                calls.push(tc);
                cleaned_content.replace_range(actual_start.., "");
                break;
            }
        }

        // 3. Check for <execute_command>...</execute_command> tags (used by models like Ornith or custom models)
        while let Some(start_tag) = cleaned_content.find("<execute_command>") {
            if let Some(end_tag) = cleaned_content[start_tag..].find("</execute_command>") {
                let actual_end = start_tag + end_tag + "</execute_command>".len();
                let inner = cleaned_content[start_tag + "<execute_command>".len()..start_tag + end_tag].trim();

                let mut cmd_str = String::new();
                if let Ok(parsed_json) = serde_json::from_str::<serde_json::Value>(inner) {
                    if let Some(c) = parsed_json.get("command").or_else(|| parsed_json.get("cmd")).and_then(|v| v.as_str()) {
                        cmd_str = c.trim().to_string();
                    }
                }
                if cmd_str.is_empty() {
                    let mut collected = Vec::new();
                    for line in inner.lines() {
                        let trimmed = line.trim();
                        if trimmed.is_empty() || trimmed.starts_with("```") {
                            continue;
                        }
                        let cleaned_line = if let Some(stripped) = trimmed.strip_prefix("command:") {
                            stripped.trim()
                        } else if let Some(stripped) = trimmed.strip_prefix("cmd:") {
                            stripped.trim()
                        } else {
                            trimmed
                        };
                        if !cleaned_line.is_empty() {
                            collected.push(cleaned_line);
                        }
                    }
                    cmd_str = collected.join(" ");
                }

                if !cmd_str.is_empty() {
                    let tc = Self::resolve_tool_call(
                        "run_command",
                        json!({ "command": cmd_str }),
                        "pending_approval",
                        available_tools,
                        calls.len(),
                    );
                    calls.push(tc);
                }

                cleaned_content.replace_range(start_tag..actual_end, "");
            } else {
                let tc = Self::resolve_tool_call(
                    "run_command",
                    serde_json::json!({}),
                    "streaming",
                    available_tools,
                    calls.len(),
                );
                calls.push(tc);
                cleaned_content.replace_range(start_tag.., "");
                break;
            }
        }

        // 4. Check for call:<tool_name>{...} (commonly emitted by Gemini, Antigravity or custom agents)
        let mut call_search_idx = 0;
        while let Some(rel_start) = cleaned_content[call_search_idx..].find("call:") {
            let call_start = call_search_idx + rel_start;
            let after_call = &cleaned_content[call_start + 5..];

            // Extract tool name (alphanumeric, underscores or hyphens)
            let name_len = after_call
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                .map(|c| c.len_utf8())
                .sum::<usize>();

            if name_len == 0 {
                call_search_idx = call_start + 5;
                continue;
            }

            let tool_name = after_call[..name_len].trim().to_string();
            let after_name = &after_call[name_len..];
            let trimmed_after_name = after_name.trim_start();
            let whitespace_len = after_name.len() - trimmed_after_name.len();

            if !trimmed_after_name.starts_with('{') {
                call_search_idx = call_start + 5 + name_len;
                continue;
            }

            let obj_offset_in_after = name_len + whitespace_len;
            let raw_obj = &after_call[obj_offset_in_after..];

            // Scan for balanced braces {...}
            let mut depth = 0;
            let mut in_quote = false;
            let mut quote_char = '"';
            let mut escaped = false;
            let mut end_offset = None;

            for (byte_offset, ch) in raw_obj.char_indices() {
                if in_quote {
                    if escaped {
                        escaped = false;
                    } else if ch == '\\' {
                        escaped = true;
                    } else if ch == '"' || ch == '”' || ch == '“' || ch == quote_char {
                        in_quote = false;
                    }
                } else if ch == '"' || ch == '“' || ch == '”' {
                    in_quote = true;
                    quote_char = ch;
                } else if ch == '{' {
                    depth += 1;
                } else if ch == '}' {
                    depth -= 1;
                    if depth == 0 {
                        end_offset = Some(byte_offset + ch.len_utf8());
                        break;
                    }
                }
            }

            // Also check if surrounded by backticks (e.g. `call:...`) or preceded by "Action: "
            let mut actual_start = call_start;
            if actual_start > 0 && cleaned_content.as_bytes()[actual_start - 1] == b'`' {
                actual_start -= 1;
            }
            if actual_start >= 8 && cleaned_content[actual_start - 8..actual_start].eq_ignore_ascii_case("action: ") {
                actual_start -= 8;
            }

            if let Some(end_len) = end_offset {
                let mut actual_end = call_start + 5 + obj_offset_in_after + end_len;
                if actual_end < cleaned_content.len() && cleaned_content.as_bytes()[actual_end] == b'`' {
                    actual_end += 1;
                }

                let raw_json_slice = &raw_obj[..end_len];

                // Attempt JSON parsing with normalization for curly / smart quotes
                let normalized_json = raw_json_slice
                    .replace(['“', '”'], "\"")
                    .replace(['‘', '’'], "'");

                let args = match serde_json::from_str::<serde_json::Value>(&normalized_json) {
                    Ok(val) => {
                        if val.is_object() {
                            val
                        } else {
                            json!({})
                        }
                    }
                    Err(_) => {
                        // Fallback: extract command/args if inner quotes were unescaped
                        let mut extracted_args = json!({});
                        let search_str = raw_json_slice.trim();
                        if let Some(pos) = search_str.find("command").or_else(|| search_str.find("cmd")) {
                            if let Some(colon_pos) = search_str[pos..].find(':') {
                                let val_part = search_str[pos + colon_pos + 1..].trim();
                                let mut val_trimmed = val_part.trim_end_matches('}').trim();
                                // Strip only a single outer quote from the beginning and end
                                if let Some(first_ch) = val_trimmed.chars().next() {
                                    if first_ch == '"' || first_ch == '“' || first_ch == '\'' {
                                        val_trimmed = &val_trimmed[first_ch.len_utf8()..];
                                    }
                                }
                                if let Some(last_ch) = val_trimmed.chars().next_back() {
                                    if last_ch == '"' || last_ch == '”' || last_ch == '\'' {
                                        val_trimmed = &val_trimmed[..val_trimmed.len() - last_ch.len_utf8()];
                                    }
                                }
                                let clean_cmd = val_trimmed.trim();
                                if !clean_cmd.is_empty() {
                                    extracted_args = json!({ "command": clean_cmd });
                                }
                            }
                        }
                        extracted_args
                    }
                };

                // Defensive normalization: if model emitted skill definition flat payload
                let eff_name = if tool_name != "create_procedural_skill"
                    && tool_name != "update_procedural_skill"
                    && tool_name != "edit_procedural_skill"
                    && (args.get("steps").is_some() || args.get("triggers").is_some())
                    && args.as_object().map_or(true, |o| o.is_empty())
                {
                    if args.get("id").is_some() || args.get("refinement_note").is_some() {
                        "update_procedural_skill".to_string()
                    } else {
                        "create_procedural_skill".to_string()
                    }
                } else if tool_name == "edit_procedural_skill" {
                    "update_procedural_skill".to_string()
                } else {
                    tool_name
                };

                let tc = Self::resolve_tool_call(&eff_name, args, "pending_approval", available_tools, calls.len());
                calls.push(tc);

                cleaned_content.replace_range(actual_start..actual_end, "");
                call_search_idx = actual_start;
            } else {
                // Streaming / unclosed call
                let tc = Self::resolve_tool_call(&tool_name, json!({}), "streaming", available_tools, calls.len());
                calls.push(tc);
                cleaned_content.replace_range(actual_start.., "");
                break;
            }
        }

        // 5. Check for <invoke name="...">...</invoke> tags (emitted by Anthropic, Qwen, or custom tool models)
        let mut invoke_search_idx = 0;
        while let Some(rel_start) = cleaned_content[invoke_search_idx..].find("<invoke") {
            let start_tag = invoke_search_idx + rel_start;
            if let Some(rel_end) = cleaned_content[start_tag..].find("</invoke>") {
                let actual_end = start_tag + rel_end + "</invoke>".len();
                let full_block = &cleaned_content[start_tag..actual_end];

                let mut tool_name = String::new();
                if let Some(open_end) = full_block.find('>') {
                    let open_tag = &full_block[..open_end];
                    if let Some(name_idx) = open_tag.find("name=") {
                        let after_name = &open_tag[name_idx + 5..];
                        let quote = after_name.chars().next().unwrap_or('"');
                        if quote == '"' || quote == '\'' {
                            let after_quote = &after_name[1..];
                            if let Some(close_quote) = after_quote.find(quote) {
                                tool_name = after_quote[..close_quote].trim().to_string();
                            }
                        } else {
                            let unquoted = after_name.split_whitespace().next().unwrap_or("").trim_end_matches('>');
                            tool_name = unquoted.to_string();
                        }
                    }
                }

                let mut arguments = serde_json::Map::new();
                let mut param_search = &full_block[..];
                while let Some(p_start) = param_search.find("<parameter") {
                    if let Some(p_close) = param_search[p_start..].find("</parameter>") {
                        let p_end = p_start + p_close + "</parameter>".len();
                        let p_block = &param_search[p_start..p_end];
                        if let Some(tag_close) = p_block.find('>') {
                            let p_tag = &p_block[..tag_close];
                            let p_inner = &p_block[tag_close + 1..p_close];
                            if let Some(name_idx) = p_tag.find("name=") {
                                let after_name = &p_tag[name_idx + 5..];
                                let quote = after_name.chars().next().unwrap_or('"');
                                let param_name = if quote == '"' || quote == '\'' {
                                    let after_quote = &after_name[1..];
                                    if let Some(close_quote) = after_quote.find(quote) {
                                        after_quote[..close_quote].trim().to_string()
                                    } else {
                                        String::new()
                                    }
                                } else {
                                    after_name.split_whitespace().next().unwrap_or("").trim_end_matches('>').to_string()
                                };

                                if !param_name.is_empty() {
                                    let val_trimmed = p_inner.trim();
                                    let val = if let Ok(num) = val_trimmed.parse::<i64>() {
                                        serde_json::json!(num)
                                    } else if let Ok(f) = val_trimmed.parse::<f64>() {
                                        serde_json::json!(f)
                                    } else if val_trimmed.eq_ignore_ascii_case("true") {
                                        serde_json::json!(true)
                                    } else if val_trimmed.eq_ignore_ascii_case("false") {
                                        serde_json::json!(false)
                                    } else {
                                        serde_json::json!(val_trimmed)
                                    };
                                    arguments.insert(param_name, val);
                                }
                            }
                        }
                        param_search = &param_search[p_end..];
                    } else {
                        break;
                    }
                }

                if !tool_name.is_empty() {
                    let tc = Self::resolve_tool_call(&tool_name, serde_json::Value::Object(arguments), "pending_approval", available_tools, calls.len());
                    calls.push(tc);
                }

                cleaned_content.replace_range(start_tag..actual_end, "");
                invoke_search_idx = start_tag;
            } else {
                cleaned_content.replace_range(start_tag.., "");
                break;
            }
        }

        let final_clean = StreamingThinkingState::strip_special_channel_tokens(&cleaned_content).trim().to_string();
        let final_calls = if calls.is_empty() { None } else { Some(calls) };
        (final_clean, final_calls)
    }

    /// Builds structured JSON messages array combining system prompt, conversation history and multimodal images
    fn build_messages_payload(
        system_prompt: &str,
        messages: &[crate::core::model::ChatMessage],
        mcp_tools: Option<&[crate::core::mcp::McpToolWithServer]>,
        enable_thinking: Option<bool>,
    ) -> Vec<serde_json::Value> {
        let mut result = Vec::new();
        let mut combined_system = system_prompt.to_string();
        if let Some(false) = enable_thinking {
            if !combined_system.is_empty() {
                combined_system.push_str("\n\n[Directive: Respond directly without internal thinking or reasoning blocks.]");
            }
        }
        if let Some(tools) = mcp_tools {
            combined_system.push_str(&Self::format_mcp_tools_system_prompt(tools));
        }

        let has_system = messages.iter().any(|m| m.role == "system");
        if !combined_system.trim().is_empty() && !has_system {
            result.push(json!({
                "role": "system",
                "content": combined_system
            }));
        }

        for (i, msg) in messages.iter().enumerate() {
            let mut content = msg.content.clone();

            // Reconstitute assistant tool calls into the message content if they were stripped
            if msg.role == "assistant" {
                if let Some(tcs) = &msg.tool_calls {
                    if !tcs.is_empty() {
                        // Strip any dangling or pre-existing <tool_call> or <execute_command> tags from content first
                        while let Some(start) = content.find("<tool_call>") {
                            if let Some(end) = content[start..].find("</tool_call>") {
                                content.replace_range(start..start + end + "</tool_call>".len(), "");
                            } else {
                                content.replace_range(start.., "");
                                break;
                            }
                        }
                        while let Some(start) = content.find("<execute_command>") {
                            if let Some(end) = content[start..].find("</execute_command>") {
                                content.replace_range(start..start + end + "</execute_command>".len(), "");
                            } else {
                                content.replace_range(start.., "");
                                break;
                            }
                        }
                        let mut tc_blocks = Vec::new();
                        for tc in tcs {
                            let tc_json = json!({
                                "name": tc.name,
                                "arguments": tc.arguments
                            });
                            tc_blocks.push(format!("<tool_call>\n{}\n</tool_call>", serde_json::to_string_pretty(&tc_json).unwrap_or_default()));
                        }
                        let calls_str = tc_blocks.join("\n");
                        if content.trim().is_empty() {
                            content = calls_str;
                        } else {
                            content = format!("{}\n\n{}", content.trim(), calls_str);
                        }
                    }
                }
            }

            if let Some(imgs) = &msg.images {
                if !imgs.is_empty() {
                    let mut content_parts = vec![json!({
                        "type": "text",
                        "text": content
                    })];
                    for img in imgs {
                        let url_str = if img.starts_with("data:") || img.starts_with("http") {
                            img.clone()
                        } else if let Ok(bytes) = std::fs::read(img) {
                            let b64 = Self::encode_base64(&bytes);
                            format!("data:image/jpeg;base64,{}", b64)
                        } else {
                            img.clone()
                        };
                        content_parts.push(json!({
                            "type": "image_url",
                            "image_url": { "url": url_str }
                        }));
                    }
                    result.push(json!({
                        "role": msg.role,
                        "content": content_parts
                    }));
                } else {
                    result.push(json!({
                        "role": msg.role,
                        "content": content
                    }));
                }
            } else {
                result.push(json!({
                    "role": msg.role,
                    "content": content
                }));
            }

            // If assistant message had executed tool calls, ensure the tool results follow in context
            if msg.role == "assistant" {
                if let Some(tcs) = &msg.tool_calls {
                    let has_results = tcs.iter().any(|t| t.result.is_some() || t.status.as_deref() == Some("completed") || t.status.as_deref() == Some("rejected") || t.status.as_deref() == Some("error"));
                    if has_results {
                        let next_is_tool_return = messages.get(i + 1).map(|next_m| next_m.content.starts_with("[MCP Tools Results]") || next_m.content.starts_with("[Retorno das Ferramentas MCP]")).unwrap_or(false);
                        if !next_is_tool_return {
                            let tool_results_str = Self::format_tool_results_block(tcs);
                            if !tool_results_str.is_empty() {
                                result.push(json!({
                                    "role": "user",
                                    "content": tool_results_str
                                }));
                            }
                        }
                    }
                }
            }
        }
        result
    }

    fn build_ollama_messages_payload(
        system_prompt: &str,
        messages: &[crate::core::model::ChatMessage],
        mcp_tools: Option<&[crate::core::mcp::McpToolWithServer]>,
        enable_thinking: Option<bool>,
    ) -> Vec<serde_json::Value> {
        let mut result = Vec::new();
        let mut combined_system = system_prompt.to_string();
        if let Some(false) = enable_thinking {
            if !combined_system.is_empty() {
                combined_system.push_str("\n\n[Directive: Respond directly without internal thinking or reasoning blocks.]");
            }
        }
        if let Some(tools) = mcp_tools {
            combined_system.push_str(&Self::format_mcp_tools_system_prompt(tools));
        }

        let has_system = messages.iter().any(|m| m.role == "system");
        if !combined_system.trim().is_empty() && !has_system {
            result.push(json!({
                "role": "system",
                "content": combined_system
            }));
        }

        for (i, msg) in messages.iter().enumerate() {
            let mut content = msg.content.clone();

            // Reconstitute assistant tool calls into the message content if they were stripped
            if msg.role == "assistant" {
                if let Some(tcs) = &msg.tool_calls {
                    if !tcs.is_empty() {
                        // Strip any dangling or pre-existing <tool_call> tags from content first
                        while let Some(start) = content.find("<tool_call>") {
                            if let Some(end) = content[start..].find("</tool_call>") {
                                content.replace_range(start..start + end + "</tool_call>".len(), "");
                            } else {
                                content.replace_range(start.., "");
                                break;
                            }
                        }
                        let mut tc_blocks = Vec::new();
                        for tc in tcs {
                            let tc_json = json!({
                                "name": tc.name,
                                "arguments": tc.arguments
                            });
                            tc_blocks.push(format!("<tool_call>\n{}\n</tool_call>", serde_json::to_string_pretty(&tc_json).unwrap_or_default()));
                        }
                        let calls_str = tc_blocks.join("\n");
                        if content.trim().is_empty() {
                            content = calls_str;
                        } else {
                            content = format!("{}\n\n{}", content.trim(), calls_str);
                        }
                    }
                }
            }

            let mut obj = json!({
                "role": msg.role,
                "content": content
            });
            if let Some(imgs) = &msg.images {
                if !imgs.is_empty() {
                    let mut b64_list = Vec::new();
                    for img in imgs {
                        if img.starts_with("data:") {
                            if let Some(pos) = img.find(";base64,") {
                                b64_list.push(img[pos + 8..].to_string());
                            }
                        } else if let Ok(bytes) = std::fs::read(img) {
                            b64_list.push(Self::encode_base64(&bytes));
                        }
                    }
                    if !b64_list.is_empty() {
                        obj["images"] = json!(b64_list);
                    }
                }
            }
            result.push(obj);

            // If assistant message had executed tool calls, ensure the tool results follow in context
            if msg.role == "assistant" {
                if let Some(tcs) = &msg.tool_calls {
                    let has_results = tcs.iter().any(|t| t.result.is_some() || t.status.as_deref() == Some("completed") || t.status.as_deref() == Some("rejected") || t.status.as_deref() == Some("error"));
                    if has_results {
                        let next_is_tool_return = messages.get(i + 1).map(|next_m| next_m.content.starts_with("[MCP Tools Results]") || next_m.content.starts_with("[Retorno das Ferramentas MCP]")).unwrap_or(false);
                        if !next_is_tool_return {
                            let tool_results_str = Self::format_tool_results_block(tcs);
                            if !tool_results_str.is_empty() {
                                result.push(json!({
                                    "role": "user",
                                    "content": tool_results_str
                                }));
                            }
                        }
                    }
                }
            }
        }
        result
    }

    pub fn encode_base64(bytes: &[u8]) -> String {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
        for chunk in bytes.chunks(3) {
            let b0 = chunk[0];
            let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
            let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };
            out.push(CHARS[(b0 >> 2) as usize] as char);
            out.push(CHARS[(((b0 & 3) << 4) | (b1 >> 4)) as usize] as char);
            if chunk.len() > 1 {
                out.push(CHARS[(((b1 & 15) << 2) | (b2 >> 6)) as usize] as char);
            } else {
                out.push('=');
            }
            if chunk.len() > 2 {
                out.push(CHARS[(b2 & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
        out
    }

    /// Decodes a base64 string into raw bytes
    pub fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
        let mut table = [255u8; 256];
        for (i, &c) in b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".iter().enumerate() {
            table[c as usize] = i as u8;
        }

        let clean: Vec<u8> = input.bytes().filter(|&b| b != b'=' && !b.is_ascii_whitespace()).collect();
        let mut out = Vec::with_capacity(clean.len() * 3 / 4);

        for chunk in clean.chunks(4) {
            let b0 = table[chunk[0] as usize];
            if b0 == 255 { return Err("Caractere base64 inválido".into()); }
            let b1 = if chunk.len() > 1 { table[chunk[1] as usize] } else { 0 };
            if b1 == 255 { return Err("Caractere base64 inválido".into()); }

            out.push((b0 << 2) | (b1 >> 4));

            if chunk.len() > 2 {
                let b2 = table[chunk[2] as usize];
                if b2 == 255 { return Err("Caractere base64 inválido".into()); }
                out.push(((b1 & 15) << 4) | (b2 >> 2));

                if chunk.len() > 3 {
                    let b3 = table[chunk[3] as usize];
                    if b3 == 255 { return Err("Caractere base64 inválido".into()); }
                    out.push(((b2 & 3) << 6) | b3);
                }
            }
        }

        Ok(out)
    }

    /// Persists an image/media attachment to a temporary folder and returns its absolute path
    pub fn persist_temp_image(img_str: &str, index: usize) -> Option<std::path::PathBuf> {
        let temp_dir = std::env::temp_dir().join("atena_media");
        let _ = std::fs::create_dir_all(&temp_dir);

        // If it's already a valid existing file on the filesystem, use it directly
        let p = std::path::Path::new(img_str);
        if p.exists() && p.is_file() {
            return Some(p.to_path_buf());
        }

        let (ext, raw_b64) = if img_str.starts_with("data:") {
            if let Some(semi) = img_str.find(';') {
                let mime = &img_str[5..semi];
                let ext = match mime {
                    "image/png" => "png",
                    "image/jpeg" | "image/jpg" => "jpg",
                    "image/webp" => "webp",
                    "image/gif" => "gif",
                    "application/pdf" => "pdf",
                    _ => "png",
                };
                let b64 = if let Some(comma) = img_str.find(',') {
                    &img_str[comma + 1..]
                } else {
                    img_str
                };
                (ext, b64)
            } else {
                ("png", img_str)
            }
        } else {
            ("png", img_str)
        };

        if let Ok(bytes) = Self::decode_base64(raw_b64.trim()) {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            let file_name = format!("atena_attachment_{}_{}.{}", timestamp, index, ext);
            let file_path = temp_dir.join(file_name);
            if std::fs::write(&file_path, &bytes).is_ok() {
                return Some(file_path);
            }
        }
        None
    }

    /// Executes inference with streaming tokens and real-time thinking and tool_calls extraction.
    /// Computes full inference metrics (prefill tokens, cache tokens, cache efficiency %, speeds and memory savings)
    pub fn compute_generation_metrics(
        params: &InferenceParams,
        est_prompt_tokens: usize,
        est_cached_tokens: usize,
        reported_prompt_tokens: Option<usize>,
        reported_cached_tokens: Option<usize>,
        reported_comp_tokens: Option<usize>,
        c_acc: &str,
        t_acc: Option<&str>,
        start: std::time::Instant,
        first_token_instant: Option<std::time::Instant>,
        reported_prefill_speed: Option<f32>,
        reported_gen_speed: Option<f32>,
        reported_finish_reason: Option<String>,
    ) -> crate::core::model::GenerationMetrics {
        let quant_type = params.kv_cache_quant.clone().unwrap_or_else(|| "f16".into());
        let vram_saving = match quant_type.to_lowercase().as_str() {
            "q4_0" | "q4_1" | "iq4_nl" => 75.0,
            "q5_0" | "q5_1" => 68.75,
            "q8_0" => 50.0,
            _ => 0.0,
        };

        let p_toks = reported_prompt_tokens.unwrap_or(est_prompt_tokens).max(1);
        let c_toks = match reported_cached_tokens {
            Some(c) if c > 0 => c,
            _ => {
                if params.enable_prompt_cache.unwrap_or(true) && est_cached_tokens > 0 {
                    let scaled = if let Some(rep_p) = reported_prompt_tokens {
                        if est_prompt_tokens > 0 {
                            ((est_cached_tokens as f64 / est_prompt_tokens as f64) * rep_p as f64).round() as usize
                        } else {
                            est_cached_tokens
                        }
                    } else {
                        est_cached_tokens
                    };
                    scaled.min(p_toks.saturating_sub(1))
                } else {
                    0
                }
            }
        };

        let prefill_toks = p_toks.saturating_sub(c_toks);
        let comp_toks = reported_comp_tokens.unwrap_or_else(|| {
            let total_chars = c_acc.len() + t_acc.map(|s| s.len()).unwrap_or(0);
            (total_chars as f32 / 3.7).ceil() as usize
        });
        let tot_toks = p_toks + comp_toks;
        let eff_pct = if p_toks > 0 {
            ((c_toks as f32 / p_toks as f32) * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };

        let gen_speed = reported_gen_speed.or_else(|| {
            let elapsed_secs = start.elapsed().as_secs_f32();
            if elapsed_secs > 0.05 && comp_toks > 0 {
                Some(comp_toks as f32 / elapsed_secs)
            } else {
                None
            }
        });

        let ttft_ms = first_token_instant.map(|t| t.duration_since(start).as_millis() as u64);

        let eff_finish_reason = match reported_finish_reason {
            Some(fr) if !fr.is_empty() && fr != "null" => Some(fr),
            _ => {
                // Safety check: if generated tokens reached or exceeded configured max_tokens
                if params.max_tokens > 0 && comp_toks >= params.max_tokens.saturating_sub(4) {
                    Some("length".to_string())
                } else {
                    None
                }
            }
        };

        crate::core::model::GenerationMetrics {
            prompt_tokens: p_toks,
            prefill_tokens: prefill_toks,
            cached_tokens: c_toks,
            completion_tokens: comp_toks,
            total_tokens: tot_toks,
            cache_efficiency_pct: (eff_pct * 10.0).round() / 10.0,
            prefill_speed_tps: reported_prefill_speed.map(|s| (s * 10.0).round() / 10.0),
            generation_speed_tps: gen_speed.map(|s| (s * 10.0).round() / 10.0),
            time_to_first_token_ms: ttft_ms,
            total_duration_ms: start.elapsed().as_millis() as u64,
            kv_cache_quant: Some(quant_type),
            vram_saving_pct: Some(vram_saving),
            finish_reason: eff_finish_reason,
        }
    }

    /// Formats prompt for Antigravity CLI ensuring Atena's active MCP tools, media attachments and strict LLM directives are enforced
    fn build_antigravity_prompt(
        system_prompt: &str,
        messages: &[crate::core::model::ChatMessage],
        mcp_tools: Option<&[crate::core::mcp::McpToolWithServer]>,
    ) -> String {
        let mut full_prompt = String::new();

        // 1. Mandatory runtime directive for the model inside Antigravity
        full_prompt.push_str("[ATENA STUDIO OPERATION DIRECTIVE]:\n");
        full_prompt.push_str("You are operating as the AI inference engine for Atena Studio.\n");
        full_prompt.push_str("DO NOT execute terminal commands, do not modify system files, and DO NOT run external CLI MCPs on your own.\n");
        full_prompt.push_str("ATTACHED FILES & DOCUMENTS: All textual content of files, documents, JSON, CSV, spreadsheets, code, or PDF documents attached by the user has already been read and is included DIRECTLY in the user's message text. Analyze and respond regarding this data immediately, without asking the user to resend or paste it.\n");
        full_prompt.push_str("Visual media exception: if there are explicitly indicated attached images in the messages below, use the view_file tool to inspect them.\n");

        let has_tools = mcp_tools.map(|t| t.iter().any(|item| item.enabled)).unwrap_or(false)
            || system_prompt.contains("[LEARNED PROCEDURAL SKILLS")
            || system_prompt.contains("<tool_call>");
        if has_tools {
            full_prompt.push_str("If you need to query external data or execute actions/skills to answer the user, EXCLUSIVELY use the Atena Studio tools or procedural skills available, strictly emitting the tool call tag:\n");
            full_prompt.push_str("<tool_call>\n{\"name\": \"tool_name\", \"arguments\": { ... }}\n</tool_call>\n");
            full_prompt.push_str("Never emit raw text syntax like `call:tool_name{...}`. Always wrap calls inside `<tool_call>...</tool_call>`.\n");
            full_prompt.push_str("BATCH / PARALLEL CALLS (GROUPING): When a request requires multiple independent queries or operations, emit all <tool_call> blocks sequentially in the SAME response. Atena Studio automatically groups multiple tools for batch approval and execution in the interface, avoiding slow one-by-one roundtrips.\n");
            full_prompt.push_str("Atena Studio will intercept these tags in the interface, request user approval, execute the tools in their ecosystem, and return the consolidated data to you.\n\n");
        } else {
            full_prompt.push_str("Respond directly in natural language text without calling tools.\n\n");
        }

        // 2. Add Atena's active MCP tools schemas
        if let Some(tools) = mcp_tools {
            let tools_text = Self::format_mcp_tools_system_prompt(tools);
            if !tools_text.trim().is_empty() {
                full_prompt.push_str(&tools_text);
                full_prompt.push_str("\n\n");
            }
        }

        // 3. System prompt
        if !system_prompt.trim().is_empty() {
            full_prompt.push_str(&format!("[System Instructions]\n{}\n\n", system_prompt.trim()));
        }

        // 4. Conversation history, multimodal attachments & previous tool results
        for (i, msg) in messages.iter().enumerate() {
            let role_label = match msg.role.as_str() {
                "user" => "User",
                "assistant" => "Assistant",
                "system" => "System",
                other => other,
            };
            let mut msg_text = msg.content.trim().to_string();

            // Persist base64 images/attachments to temp files and append explicit view references
            if let Some(imgs) = &msg.images {
                let mut file_refs = Vec::new();
                for (idx, img) in imgs.iter().enumerate() {
                    if let Some(path) = Self::persist_temp_image(img, idx) {
                        file_refs.push(format!("[Attached file/image from user: {} - Inspect this file using the view_file tool to respond]", path.display()));
                    }
                }
                if !file_refs.is_empty() {
                    if !msg_text.is_empty() {
                        msg_text.push_str("\n\n");
                    }
                    msg_text.push_str(&file_refs.join("\n"));
                }
            }

            full_prompt.push_str(&format!("{}: {}\n\n", role_label, msg_text));

            // If assistant message had executed tool calls, ensure the tool results follow in context
            if msg.role == "assistant" {
                if let Some(tcs) = &msg.tool_calls {
                    let has_results = tcs.iter().any(|t| t.result.is_some() || t.status.as_deref() == Some("completed") || t.status.as_deref() == Some("rejected") || t.status.as_deref() == Some("error"));
                    if has_results {
                        let next_is_tool_return = messages.get(i + 1).map(|next_m| next_m.content.starts_with("[MCP Tools Results]") || next_m.content.starts_with("[Retorno das Ferramentas MCP]")).unwrap_or(false);
                        if !next_is_tool_return {
                            let tool_results_str = Self::format_tool_results_block(tcs);
                            if !tool_results_str.is_empty() {
                                full_prompt.push_str(&format!("System: {}\n\n", tool_results_str));
                            }
                        }
                    }
                }
            }
        }

        full_prompt.trim().to_string()
    }

    pub(crate) fn format_payload_preview(payload: &serde_json::Value) -> String {
        let mut cloned = payload.clone();
        if let Some(msgs) = cloned.get_mut("messages").and_then(|m| m.as_array_mut()) {
            for msg in msgs.iter_mut() {
                if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                    let char_count = content.chars().count();
                    if char_count > 120 {
                        let prefix: String = content.chars().take(50).collect();
                        let suffix: String = content.chars().rev().take(40).collect::<Vec<_>>().into_iter().rev().collect();
                        msg["content"] = serde_json::json!(format!("{}... <Truncated in logs> ...{}", prefix, suffix));
                    }
                }
            }
        }
        serde_json::to_string_pretty(&cloned).unwrap_or_else(|_| payload.to_string())
    }

    /// Stops and aborts any currently active prompt processing ticker.
    pub fn stop_active_prompt_ticker() {
        if let Ok(mut guard) = ACTIVE_PROMPT_TICKER.lock() {
            if let Some((is_done, abort_handle)) = guard.take() {
                is_done.store(true, Ordering::Relaxed);
                abort_handle.abort();
            }
        }
    }

    /// Aborts any currently active inference generation immediately.
    pub fn abort_active_inference() {
        if let Ok(mut guard) = ACTIVE_INFERENCE_ABORT.lock() {
            if let Some(flag) = guard.take() {
                flag.store(true, Ordering::Relaxed);
            }
        }
        Self::stop_active_prompt_ticker();
    }

    /// Launches a smooth background progress ticker for prompt processing (prefill) that logs in real time.
    fn start_prompt_progress_ticker(
        log_id: String,
        model_name: String,
        est_prompt_tokens: usize,
    ) -> (Arc<AtomicBool>, tokio::task::JoinHandle<()>) {
        Self::stop_active_prompt_ticker();

        let is_done = Arc::new(AtomicBool::new(false));
        let done_clone = is_done.clone();
        let est_tokens = est_prompt_tokens.max(1);

        let handle = tokio::spawn(async move {
            let start = std::time::Instant::now();
            // Estimate prefill duration: ~180 tokens/sec baseline for local models
            let est_duration_secs = (est_tokens as f32 / 180.0).clamp(0.4, 60.0);

            // Initial zero progress
            crate::services::server_ctl::LocalServerController::update_or_record_dev_log(
                &log_id,
                "INFO",
                Some(&model_name),
                "Prompt processing progress: 0.0%",
                None,
            );
            crate::services::server_ctl::LocalServerController::emit_prompt_progress(0.0);

            while !done_clone.load(Ordering::Relaxed) {
                tokio::time::sleep(Duration::from_millis(100)).await;
                if done_clone.load(Ordering::Relaxed) {
                    break;
                }
                let elapsed_secs = start.elapsed().as_secs_f32();
                let ratio = elapsed_secs / est_duration_secs;
                let pct = if ratio < 1.0 {
                    (ratio * 88.0).clamp(1.0, 88.0)
                } else {
                    let extra = ratio - 1.0;
                    88.0 + (10.5 * (1.0 - (-extra * 0.9).exp()))
                };
                let pct = pct.min(98.5);

                crate::services::server_ctl::LocalServerController::update_or_record_dev_log(
                    &log_id,
                    "INFO",
                    Some(&model_name),
                    &format!("Prompt processing progress: {:.1}%", pct),
                    None,
                );
                crate::services::server_ctl::LocalServerController::emit_prompt_progress(pct);
            }
        });

        if let Ok(mut guard) = ACTIVE_PROMPT_TICKER.lock() {
            *guard = Some((is_done.clone(), handle.abort_handle()));
        }

        (is_done, handle)
    }

    /// Executes inference with streaming tokens, metrics, real-time thinking and tool_calls extraction.
    pub async fn execute_stream_callback<F>(
        model: &ModelInfo,
        system_prompt: &str,
        messages: &[crate::core::model::ChatMessage],
        params: &InferenceParams,
        mlx_host: &str,
        mlx_port: u16,
        ollama_host: &str,
        ollama_port: u16,
        mut on_chunk: F,
    ) -> Result<(), String>
    where
        F: FnMut(String, Option<String>, Option<Vec<crate::core::mcp::McpToolCall>>, bool, u64, Option<crate::core::model::GenerationMetrics>),
    {
        Self::abort_active_inference();
        let abort_flag = Arc::new(AtomicBool::new(false));
        let abort_clone = abort_flag.clone();
        if let Ok(mut guard) = ACTIVE_INFERENCE_ABORT.lock() {
            *guard = Some(abort_flag);
        }
        let start = std::time::Instant::now();
        let mut first_token_time: Option<std::time::Instant> = None;
        let enable_thinking = params.enable_thinking.unwrap_or(true);
        // Baseline token estimates and sliding window guard
        let max_ctx = if params.context_length > 0 {
            params.context_length
        } else if model.context_length > 0 {
            model.context_length
        } else {
            8192
        };
        let max_allowed_prompt_tokens = max_ctx.saturating_sub(params.max_tokens.max(512) + 256);

        let prompt_chars = system_prompt.len() + messages.iter().map(|m| m.content.len()).sum::<usize>();
        let est_prompt_tokens = (prompt_chars as f32 / 3.7).ceil() as usize;

        let safe_messages = if est_prompt_tokens > max_allowed_prompt_tokens && messages.len() > 2 {
            let mut pruned = Vec::new();
            let mut acc_chars = system_prompt.len();
            let last_idx = messages.len() - 1;
            acc_chars += messages[last_idx].content.len();
            pruned.push(messages[last_idx].clone());

            for msg in messages[..last_idx].iter().rev() {
                let msg_chars = msg.content.len();
                let est_tok = ((acc_chars + msg_chars) as f32 / 3.7) as usize;
                if est_tok < max_allowed_prompt_tokens {
                    acc_chars += msg_chars;
                    pruned.insert(0, msg.clone());
                } else {
                    break;
                }
            }
            pruned
        } else {
            messages.to_vec()
        };

        let json_messages = Self::build_messages_payload(system_prompt, &safe_messages, params.mcp_tools.as_deref(), params.enable_thinking);

        let prev_chars = if safe_messages.len() > 1 {
            system_prompt.len() + safe_messages[..safe_messages.len() - 1].iter().map(|m| m.content.len()).sum::<usize>()
        } else {
            0
        };
        let est_cached_tokens = if safe_messages.len() > 1 {
            (prev_chars as f32 / 3.7).ceil() as usize
        } else {
            0
        };

        let mut reported_prompt: Option<usize> = None;
        let mut reported_cached: Option<usize> = None;
        let mut reported_comp: Option<usize> = None;
        let mut reported_prefill_tps: Option<f32> = None;
        let mut reported_gen_tps: Option<f32> = None;
        let mut reported_finish_reason: Option<String> = None;

        match model.backend {
            BackendType::MlxLm => {
                let endpoint = format!("http://{}:{}/v1/chat/completions", mlx_host, mlx_port);
                if let Ok(client) = reqwest::Client::builder()
                    .http1_only()
                    .connect_timeout(Duration::from_secs(15))
                    .timeout(Duration::from_secs(1800))
                    .build()
                {
                    let mut payload = json!({
                        "model": model.local_path.clone().unwrap_or_else(|| model.id.clone()),
                        "messages": json_messages,
                        "temperature": params.temperature,
                        "top_p": params.top_p,
                        "max_tokens": params.max_tokens,
                        "stream": true,
                        "stream_options": { "include_usage": true },
                        "cache_prompt": params.enable_prompt_cache.unwrap_or(true)
                    });

                    if let Some(enable_think) = params.enable_thinking {
                        payload["enable_thinking"] = json!(enable_think);
                        payload["chat_template_args"] = json!({ "enable_thinking": enable_think });
                    }

                    if let Some(tools) = &params.mcp_tools {
                        let tools_payload: Vec<serde_json::Value> = tools.iter().filter(|t| t.enabled).map(|t| {
                            json!({
                                "type": "function",
                                "function": {
                                    "name": t.tool.name,
                                    "description": t.tool.description.clone().unwrap_or_default(),
                                    "parameters": t.tool.input_schema
                                }
                            })
                        }).collect();
                        if !tools_payload.is_empty() {
                            payload["tools"] = json!(tools_payload);
                            payload["tool_choice"] = json!("auto");
                        }
                    }

                    log::info!("🚀 [MLX] Enviando requisição SSE para {}", endpoint);
                    let model_name = model.name.clone();
                    let body_preview = Self::format_payload_preview(&payload);
                    let messages_count = safe_messages.len();

                    crate::services::server_ctl::LocalServerController::record_dev_log(
                        "DEBUG",
                        None,
                        &format!("Received request: POST to /v1/chat/completions with body  {}", body_preview),
                        Some(&body_preview),
                    );
                    crate::services::server_ctl::LocalServerController::record_dev_log(
                        "INFO",
                        Some(&model_name),
                        &format!("Running chat completion on conversation with {} messages.", messages_count),
                        None,
                    );
                    crate::services::server_ctl::LocalServerController::record_dev_log(
                        "INFO",
                        Some(&model_name),
                        "Streaming response...",
                        None,
                    );

                    let req_num = REQ_STREAM_LOG_COUNTER.fetch_add(1, Ordering::Relaxed);
                    let prompt_log_id = format!("dev-prompt-{}", req_num);
                    let token_log_id = format!("dev-token-{}", req_num);
                    let (is_prompt_done, prompt_ticker) = Self::start_prompt_progress_ticker(
                        prompt_log_id.clone(),
                        model_name.clone(),
                        est_prompt_tokens,
                    );
                    let mut last_token_gen_log = std::time::Instant::now();

                    let t_req = std::time::Instant::now();
                    let mut post_res = client.post(&endpoint).json(&payload).send().await;
                    if let Ok(ref r) = post_res {
                        if r.status().as_u16() == 422 || r.status().as_u16() == 400 {
                            log::warn!("⚠️ [MLX] Server rejected payload with HTTP {}, retrying with standard streaming payload...", r.status());
                            let mut fallback_payload = json!({
                                "model": model.local_path.clone().unwrap_or_else(|| model.id.clone()),
                                "messages": json_messages,
                                "temperature": params.temperature,
                                "top_p": params.top_p,
                                "max_tokens": params.max_tokens,
                                "stream": true
                            });
                            if let Some(enable_think) = params.enable_thinking {
                                fallback_payload["enable_thinking"] = json!(enable_think);
                                fallback_payload["chat_template_args"] = json!({ "enable_thinking": enable_think });
                            }
                            if let Some(tools) = &params.mcp_tools {
                                let tools_payload: Vec<serde_json::Value> = tools.iter().filter(|t| t.enabled).map(|t| {
                                    json!({
                                        "type": "function",
                                        "function": {
                                            "name": t.tool.name,
                                            "description": t.tool.description.clone().unwrap_or_default(),
                                            "parameters": t.tool.input_schema
                                        }
                                    })
                                }).collect();
                                if !tools_payload.is_empty() {
                                    fallback_payload["tools"] = json!(tools_payload);
                                    fallback_payload["tool_choice"] = json!("auto");
                                }
                            }
                            post_res = client.post(&endpoint).json(&fallback_payload).send().await;
                        }
                    }

                    match post_res {
                        Ok(resp) => {
                            let status = resp.status();
                            let latency = t_req.elapsed().as_millis() as u64;
                            crate::services::server_ctl::LocalServerController::record_log_detailed(
                                "POST",
                                &endpoint,
                                status.as_u16(),
                                latency,
                                est_prompt_tokens,
                                0,
                                Some(&model_name),
                                Some(&body_preview),
                            );

                            if !status.is_success() {
                                is_prompt_done.store(true, Ordering::Relaxed);
                                prompt_ticker.abort();
                                Self::stop_active_prompt_ticker();
                                let err_body = resp.text().await.unwrap_or_default();
                                println!("❌ [MLX STREAM ERRO HTTP {}]: {}", status, err_body);
                            } else {
                                let mut stream = resp.bytes_stream();
                                let mut state = StreamingThinkingState::new(false);
                                let mut buffer = String::new();
                                let mut accumulated_delta_tools: Vec<serde_json::Value> = Vec::new();

                                let mut is_stream_done = false;
                                while let Some(chunk_result) = stream.next().await {
                                    if abort_clone.load(Ordering::Relaxed) {
                                        log::info!("🛑 [MLX] Inference streaming aborted by user.");
                                        break;
                                    }
                                    if is_stream_done {
                                        break;
                                    }
                                    match chunk_result {
                                        Ok(chunk_bytes) => {
                                            let text = String::from_utf8_lossy(&chunk_bytes);
                                            buffer.push_str(&text);

                                            while let Some(line_end) = buffer.find('\n') {
                                                let line = buffer[..line_end].trim().to_string();
                                                buffer.drain(..line_end + 1);

                                                if line.starts_with("data: ") {
                                                    let data_str = line.trim_start_matches("data: ").trim();
                                                    if data_str == "[DONE]" {
                                                        println!("🏁 [MLX SSE] Recebido [DONE]");
                                                        is_stream_done = true;
                                                        break;
                                                    }
                                                    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(data_str) {
                                                        if let Some(usage) = json_val.get("usage") {
                                                            if let Some(pt) = usage.get("prompt_tokens").and_then(|v| v.as_u64()) {
                                                                reported_prompt = Some(pt as usize);
                                                            }
                                                            if let Some(ct) = usage.get("completion_tokens").and_then(|v| v.as_u64()) {
                                                                reported_comp = Some(ct as usize);
                                                            }
                                                            if let Some(cd) = usage.get("prompt_tokens_details").and_then(|d| d.get("cached_tokens")).and_then(|v| v.as_u64()) {
                                                                reported_cached = Some(cd as usize);
                                                            }
                                                        }

                                                        if let Some(choices_arr) = json_val.get("choices").and_then(|c| c.as_array()) {
                                                            if let Some(first_choice) = choices_arr.first() {
                                                                if let Some(fr) = first_choice.get("finish_reason").and_then(|r| r.as_str()) {
                                                                    if !fr.is_empty() && fr != "null" {
                                                                        reported_finish_reason = Some(fr.to_string());
                                                                    }
                                                                }
                                                            }
                                                        }

                                                        if let Some(delta) = json_val.get("choices")
                                                            .and_then(|c| c.as_array())
                                                            .and_then(|a| a.first())
                                                            .and_then(|c| c.get("delta"))
                                                        {
                                                            let reasoning = delta.get("reasoning_content")
                                                                .or_else(|| delta.get("reasoning"))
                                                                .or_else(|| delta.get("thinking"))
                                                                .and_then(|s| s.as_str());
                                                            let content = delta.get("content").and_then(|s| s.as_str());

                                                            let has_first_token = content.map(|c| !c.is_empty()).unwrap_or(false)
                                                                || reasoning.map(|r| !r.is_empty()).unwrap_or(false)
                                                                || delta.get("tool_calls").and_then(|t| t.as_array()).map(|a| !a.is_empty()).unwrap_or(false);

                                                            if first_token_time.is_none() && has_first_token {
                                                                first_token_time = Some(std::time::Instant::now());
                                                                is_prompt_done.store(true, Ordering::Relaxed);
                                                                prompt_ticker.abort();
                                                                Self::stop_active_prompt_ticker();
                                                                crate::services::server_ctl::LocalServerController::emit_prompt_progress(100.0);
                                                                println!("⚡ [MLX SSE FIRST TOKEN] Recebido em {:?}", start.elapsed());
                                                                crate::services::server_ctl::LocalServerController::update_or_record_dev_log(
                                                                    &prompt_log_id,
                                                                    "INFO",
                                                                    Some(&model_name),
                                                                    "Prompt processing progress: 100.0%",
                                                                    None,
                                                                );
                                                            }

                                                            if let Some(tool_calls_arr) = delta.get("tool_calls").and_then(|t| t.as_array()) {
                                                                for tc_val in tool_calls_arr {
                                                                    accumulated_delta_tools.push(tc_val.clone());
                                                                }
                                                            }

                                                            state.push_delta(reasoning, content);
                                                            let (c_acc, t_acc) = state.current(enable_thinking);
                                                            let dt_ref = if accumulated_delta_tools.is_empty() { None } else { Some(&accumulated_delta_tools) };
                                                            let (clean_content, tool_calls) = Self::extract_tool_calls(&c_acc, dt_ref, params.mcp_tools.as_deref());
                                                            
                                                            let metrics = Self::compute_generation_metrics(
                                                                params,
                                                                est_prompt_tokens,
                                                                est_cached_tokens,
                                                                reported_prompt,
                                                                reported_cached,
                                                                reported_comp,
                                                                &clean_content,
                                                                t_acc.as_deref(),
                                                                start,
                                                                first_token_time,
                                                                reported_prefill_tps,
                                                                reported_gen_tps,
                                                                reported_finish_reason.clone(),
                                                            );

                                                            if let Some(ft) = first_token_time {
                                                                let current_toks = reported_comp.unwrap_or_else(|| {
                                                                    let total_chars = c_acc.len() + t_acc.as_deref().map(|t| t.len()).unwrap_or(0);
                                                                    (total_chars as f32 / 3.7).ceil() as usize
                                                                });
                                                                if current_toks > 0 && last_token_gen_log.elapsed() >= Duration::from_millis(250) {
                                                                    last_token_gen_log = std::time::Instant::now();
                                                                    let elapsed_sec = ft.elapsed().as_secs_f32().max(0.01);
                                                                    let live_tps = reported_gen_tps.unwrap_or_else(|| (current_toks as f32 / elapsed_sec).max(0.1));
                                                                    crate::services::server_ctl::LocalServerController::update_or_record_dev_log(
                                                                        &token_log_id,
                                                                        "INFO",
                                                                        Some(&model_name),
                                                                        &format!("Generating response: {} tokens ({:.1} tok/s)...", current_toks, live_tps),
                                                                        None,
                                                                    );
                                                                }
                                                            }

                                                            on_chunk(clean_content, t_acc, tool_calls, false, start.elapsed().as_millis() as u64, Some(metrics));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        Err(chunk_err) => {
                                            log::warn!("❌ [MLX CHUNK ERROR] {:?}", chunk_err);
                                        }
                                    }
                                }

                                is_prompt_done.store(true, Ordering::Relaxed);
                                prompt_ticker.abort();
                                Self::stop_active_prompt_ticker();

                                if abort_clone.load(Ordering::Relaxed) {
                                    return Ok(());
                                }

                                crate::services::server_ctl::LocalServerController::emit_prompt_progress(100.0);

                                let (final_content, final_thinking) = state.finalize(enable_thinking);
                                let dt_ref = if accumulated_delta_tools.is_empty() { None } else { Some(&accumulated_delta_tools) };
                                let (clean_final, final_tool_calls) = Self::extract_tool_calls(&final_content, dt_ref, params.mcp_tools.as_deref());
                                log::info!("🏁 [MLX] Streaming finalizado com sucesso ({} caracteres)", clean_final.len());
                                
                                let total_latency = start.elapsed().as_millis() as u64;
                                let comp_tokens = reported_comp.unwrap_or_else(|| {
                                    (clean_final.len() as f32 / 3.7).ceil() as usize
                                });
                                let prompt_toks = reported_prompt.unwrap_or(est_prompt_tokens);
                                let final_tps_val = reported_gen_tps.unwrap_or_else(|| {
                                    if let Some(ft) = first_token_time {
                                        let elapsed_sec = ft.elapsed().as_secs_f32();
                                        if elapsed_sec > 0.05 { (comp_tokens as f32 / elapsed_sec).max(0.1) } else { 0.0 }
                                    } else {
                                        0.0
                                    }
                                });
                                let tps_info = if final_tps_val > 0.0 {
                                    format!(" ({:.1} tok/s)", final_tps_val)
                                } else {
                                    String::new()
                                };

                                crate::services::server_ctl::LocalServerController::update_or_record_dev_log(
                                    &token_log_id,
                                    "INFO",
                                    Some(&model_name),
                                    &format!("Finished streaming response: {} tokens generated{} in {}ms", comp_tokens, tps_info, total_latency),
                                    None,
                                );
                                crate::services::server_ctl::LocalServerController::record_log_detailed(
                                    "POST",
                                    &endpoint,
                                    200,
                                    total_latency,
                                    prompt_toks,
                                    comp_tokens,
                                    Some(&model_name),
                                    Some(&body_preview),
                                );

                                let final_metrics = Self::compute_generation_metrics(
                                    params,
                                    est_prompt_tokens,
                                    est_cached_tokens,
                                    reported_prompt,
                                    reported_cached,
                                    reported_comp,
                                    &clean_final,
                                    final_thinking.as_deref(),
                                    start,
                                    first_token_time,
                                    reported_prefill_tps,
                                    reported_gen_tps,
                                    reported_finish_reason.clone(),
                                );

                                on_chunk(clean_final, final_thinking, final_tool_calls, true, start.elapsed().as_millis() as u64, Some(final_metrics));
                                return Ok(());
                            }
                        }
                        Err(req_err) => {
                            let latency = t_req.elapsed().as_millis() as u64;
                            crate::services::server_ctl::LocalServerController::record_log(
                                "POST",
                                &endpoint,
                                502,
                                latency,
                                est_prompt_tokens,
                                0,
                            );
                            println!("❌ [MLX REQWEST NETWORK ERROR] {:?}", req_err);
                        }
                    }
                }

                // Fallback to non-streaming execution if HTTP stream fails
                println!("⚠️⚠️⚠️ [MLX FALLBACK ACTIVATED] O stream HTTP falhou! Entrando no fallback NÃO-STREAMING execute_stream()!");
                if let Ok((content, thinking, tool_calls)) = Self::execute_stream(model, system_prompt, messages, params, mlx_host, mlx_port, ollama_host, ollama_port).await {
                    println!("⚠️ [MLX FALLBACK EXECUTED] execute_stream retornou resposta completa (não-stream)");
                    let metrics = Self::compute_generation_metrics(
                        params,
                        est_prompt_tokens,
                        est_cached_tokens,
                        None,
                        None,
                        None,
                        &content,
                        thinking.as_deref(),
                        start,
                        None,
                        None,
                        None,
                        None,
                    );
                    on_chunk(content, thinking, tool_calls, true, start.elapsed().as_millis() as u64, Some(metrics));
                    return Ok(());
                }
            }
            BackendType::Ollama | BackendType::LocalNative => {
                // If model is a local GGUF file running via llama-server on mlx_port (or if llama-server is active)
                if model.format == crate::core::model::ModelFormat::Gguf || model.local_path.is_some() {
                    let endpoint = format!("http://{}:{}/v1/chat/completions", mlx_host, mlx_port);
                    if let Ok(client) = reqwest::Client::builder()
                        .http1_only()
                        .connect_timeout(Duration::from_secs(15))
                        .timeout(Duration::from_secs(1800))
                        .build()
                    {
                        let mut payload = json!({
                            "model": model.local_path.clone().unwrap_or_else(|| model.id.clone()),
                            "messages": json_messages,
                            "temperature": params.temperature,
                            "top_p": params.top_p,
                            "max_tokens": params.max_tokens,
                            "stream": true,
                            "stream_options": { "include_usage": true },
                            "cache_prompt": params.enable_prompt_cache.unwrap_or(true)
                        });

                        if let Some(enable_think) = params.enable_thinking {
                            payload["enable_thinking"] = json!(enable_think);
                        }

                        if let Some(tools) = &params.mcp_tools {
                            let tools_payload: Vec<serde_json::Value> = tools.iter().filter(|t| t.enabled).map(|t| {
                                json!({
                                    "type": "function",
                                    "function": {
                                        "name": t.tool.name,
                                        "description": t.tool.description.clone().unwrap_or_default(),
                                        "parameters": t.tool.input_schema
                                    }
                                })
                            }).collect();
                            if !tools_payload.is_empty() {
                                payload["tools"] = json!(tools_payload);
                                payload["tool_choice"] = json!("auto");
                            }
                        }

                        let model_name = model.name.clone();
                        let body_preview = Self::format_payload_preview(&payload);
                        let messages_count = safe_messages.len();

                        crate::services::server_ctl::LocalServerController::record_dev_log(
                            "DEBUG",
                            None,
                            &format!("Received request: POST to /v1/chat/completions with body  {}", body_preview),
                            Some(&body_preview),
                        );
                        crate::services::server_ctl::LocalServerController::record_dev_log(
                            "INFO",
                            Some(&model_name),
                            &format!("Running chat completion on conversation with {} messages.", messages_count),
                            None,
                        );
                        crate::services::server_ctl::LocalServerController::record_dev_log(
                            "INFO",
                            Some(&model_name),
                            "Streaming response...",
                            None,
                        );

                        let req_num = REQ_STREAM_LOG_COUNTER.fetch_add(1, Ordering::Relaxed);
                        let prompt_log_id = format!("dev-prompt-{}", req_num);
                        let token_log_id = format!("dev-token-{}", req_num);
                        let (is_prompt_done, prompt_ticker) = Self::start_prompt_progress_ticker(
                            prompt_log_id.clone(),
                            model_name.clone(),
                            est_prompt_tokens,
                        );
                        let mut last_token_gen_log = std::time::Instant::now();

                        let t_req = std::time::Instant::now();
                        if let Ok(resp) = client.post(&endpoint).json(&payload).send().await {
                            let status = resp.status();
                            let latency = t_req.elapsed().as_millis() as u64;
                            crate::services::server_ctl::LocalServerController::record_log_detailed(
                                "POST",
                                &endpoint,
                                status.as_u16(),
                                latency,
                                est_prompt_tokens,
                                0,
                                Some(&model_name),
                                Some(&body_preview),
                            );
                            if resp.status().is_success() {
                                let mut stream = resp.bytes_stream();
                                let mut state = StreamingThinkingState::new(false);
                                let mut buffer = String::new();
                                let mut accumulated_delta_tools: Vec<serde_json::Value> = Vec::new();

                                while let Some(chunk_result) = stream.next().await {
                                    if abort_clone.load(Ordering::Relaxed) {
                                        log::info!("🛑 [llama-server] Inference streaming aborted by user.");
                                        break;
                                    }
                                    if let Ok(chunk_bytes) = chunk_result {
                                        let text = String::from_utf8_lossy(&chunk_bytes);
                                        buffer.push_str(&text);

                                        while let Some(line_end) = buffer.find('\n') {
                                            let line = buffer[..line_end].trim().to_string();
                                            buffer.drain(..line_end + 1);

                                            if line.starts_with("data: ") {
                                                let data_str = line.trim_start_matches("data: ").trim();
                                                if data_str == "[DONE]" {
                                                    break;
                                                }
                                                if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(data_str) {
                                                    if let Some(usage) = json_val.get("usage") {
                                                        if let Some(pt) = usage.get("prompt_tokens").and_then(|v| v.as_u64()) {
                                                            reported_prompt = Some(pt as usize);
                                                        }
                                                        if let Some(ct) = usage.get("completion_tokens").and_then(|v| v.as_u64()) {
                                                            reported_comp = Some(ct as usize);
                                                        }
                                                        if let Some(cd) = usage.get("prompt_tokens_details").and_then(|d| d.get("cached_tokens")).and_then(|v| v.as_u64()) {
                                                            reported_cached = Some(cd as usize);
                                                        }
                                                    }

                                                    if let Some(timings) = json_val.get("timings") {
                                                        if let Some(pn) = timings.get("prompt_n").and_then(|v| v.as_u64()) {
                                                            reported_prompt = Some(pn as usize);
                                                        }
                                                        if let Some(pps) = timings.get("prompt_per_second").and_then(|v| v.as_f64()) {
                                                            reported_prefill_tps = Some(pps as f32);
                                                        }
                                                        if let Some(pred_n) = timings.get("predicted_n").and_then(|v| v.as_u64()) {
                                                            reported_comp = Some(pred_n as usize);
                                                        }
                                                        if let Some(pred_ps) = timings.get("predicted_per_second").and_then(|v| v.as_f64()) {
                                                            reported_gen_tps = Some(pred_ps as f32);
                                                        }
                                                        if let Some(cached_n) = timings.get("prompt_tokens_cached").or_else(|| timings.get("cache_tokens_eval")).and_then(|v| v.as_u64()) {
                                                            reported_cached = Some(cached_n as usize);
                                                        }
                                                    }

                                                    if let Some(choices_arr) = json_val.get("choices").and_then(|c| c.as_array()) {
                                                        if let Some(first_choice) = choices_arr.first() {
                                                            if let Some(fr) = first_choice.get("finish_reason").and_then(|r| r.as_str()) {
                                                                if !fr.is_empty() && fr != "null" {
                                                                    reported_finish_reason = Some(fr.to_string());
                                                                }
                                                            }
                                                        }
                                                    }

                                                    if let Some(delta) = json_val.get("choices")
                                                        .and_then(|c| c.as_array())
                                                        .and_then(|a| a.first())
                                                        .and_then(|c| c.get("delta"))
                                                    {
                                                        let reasoning = delta.get("reasoning_content")
                                                            .or_else(|| delta.get("reasoning"))
                                                            .or_else(|| delta.get("thinking"))
                                                            .and_then(|s| s.as_str());
                                                        let content = delta.get("content").and_then(|s| s.as_str());

                                                        let has_first_token = content.map(|c| !c.is_empty()).unwrap_or(false)
                                                            || reasoning.map(|r| !r.is_empty()).unwrap_or(false)
                                                            || delta.get("tool_calls").and_then(|t| t.as_array()).map(|a| !a.is_empty()).unwrap_or(false);

                                                        if first_token_time.is_none() && has_first_token {
                                                            first_token_time = Some(std::time::Instant::now());
                                                            is_prompt_done.store(true, Ordering::Relaxed);
                                                            prompt_ticker.abort();
                                                            Self::stop_active_prompt_ticker();
                                                            crate::services::server_ctl::LocalServerController::emit_prompt_progress(100.0);
                                                            crate::services::server_ctl::LocalServerController::update_or_record_dev_log(
                                                                &prompt_log_id,
                                                                "INFO",
                                                                Some(&model_name),
                                                                "Prompt processing progress: 100.0%",
                                                                None,
                                                            );
                                                        }

                                                        if let Some(tool_calls_arr) = delta.get("tool_calls").and_then(|t| t.as_array()) {
                                                            for tc_val in tool_calls_arr {
                                                                accumulated_delta_tools.push(tc_val.clone());
                                                            }
                                                        }

                                                        state.push_delta(reasoning, content);
                                                        let (c_acc, t_acc) = state.current(enable_thinking);
                                                        let dt_ref = if accumulated_delta_tools.is_empty() { None } else { Some(&accumulated_delta_tools) };
                                                        let (clean_content, tool_calls) = Self::extract_tool_calls(&c_acc, dt_ref, params.mcp_tools.as_deref());
                                                        
                                                        let metrics = Self::compute_generation_metrics(
                                                            params,
                                                            est_prompt_tokens,
                                                            est_cached_tokens,
                                                            reported_prompt,
                                                            reported_cached,
                                                            reported_comp,
                                                            &clean_content,
                                                            t_acc.as_deref(),
                                                            start,
                                                            first_token_time,
                                                            reported_prefill_tps,
                                                            reported_gen_tps,
                                                            reported_finish_reason.clone(),
                                                        );

                                                        if let Some(ft) = first_token_time {
                                                            let current_toks = reported_comp.unwrap_or_else(|| {
                                                                let total_chars = c_acc.len() + t_acc.as_deref().map(|t| t.len()).unwrap_or(0);
                                                                (total_chars as f32 / 3.7).ceil() as usize
                                                            });
                                                            if current_toks > 0 && last_token_gen_log.elapsed() >= Duration::from_millis(250) {
                                                                last_token_gen_log = std::time::Instant::now();
                                                                let elapsed_sec = ft.elapsed().as_secs_f32().max(0.01);
                                                                let live_tps = reported_gen_tps.unwrap_or_else(|| (current_toks as f32 / elapsed_sec).max(0.1));
                                                                crate::services::server_ctl::LocalServerController::update_or_record_dev_log(
                                                                    &token_log_id,
                                                                    "INFO",
                                                                    Some(&model_name),
                                                                    &format!("Generating response: {} tokens ({:.1} tok/s)...", current_toks, live_tps),
                                                                    None,
                                                                );
                                                            }
                                                        }

                                                        on_chunk(clean_content, t_acc, tool_calls, false, start.elapsed().as_millis() as u64, Some(metrics));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                is_prompt_done.store(true, Ordering::Relaxed);
                                prompt_ticker.abort();
                                Self::stop_active_prompt_ticker();

                                if abort_clone.load(Ordering::Relaxed) {
                                    return Ok(());
                                }

                                crate::services::server_ctl::LocalServerController::emit_prompt_progress(100.0);

                                let (final_content, final_thinking) = state.finalize(enable_thinking);
                                let dt_ref = if accumulated_delta_tools.is_empty() { None } else { Some(&accumulated_delta_tools) };
                                let (clean_final, final_tool_calls) = Self::extract_tool_calls(&final_content, dt_ref, params.mcp_tools.as_deref());
                                
                                let total_latency = start.elapsed().as_millis() as u64;
                                let comp_tokens = reported_comp.unwrap_or_else(|| {
                                    (clean_final.len() as f32 / 3.7).ceil() as usize
                                });
                                let prompt_toks = reported_prompt.unwrap_or(est_prompt_tokens);
                                let final_tps_val = reported_gen_tps.unwrap_or_else(|| {
                                    if let Some(ft) = first_token_time {
                                        let elapsed_sec = ft.elapsed().as_secs_f32();
                                        if elapsed_sec > 0.05 { (comp_tokens as f32 / elapsed_sec).max(0.1) } else { 0.0 }
                                    } else {
                                        0.0
                                    }
                                });
                                let tps_info = if final_tps_val > 0.0 {
                                    format!(" ({:.1} tok/s)", final_tps_val)
                                } else {
                                    String::new()
                                };

                                crate::services::server_ctl::LocalServerController::update_or_record_dev_log(
                                    &token_log_id,
                                    "INFO",
                                    Some(&model_name),
                                    &format!("Finished streaming response: {} tokens generated{} in {}ms", comp_tokens, tps_info, total_latency),
                                    None,
                                );
                                crate::services::server_ctl::LocalServerController::record_log_detailed(
                                    "POST",
                                    &endpoint,
                                    200,
                                    total_latency,
                                    prompt_toks,
                                    comp_tokens,
                                    Some(&model_name),
                                    Some(&body_preview),
                                );

                                let final_metrics = Self::compute_generation_metrics(
                                    params,
                                    est_prompt_tokens,
                                    est_cached_tokens,
                                    reported_prompt,
                                    reported_cached,
                                    reported_comp,
                                    &clean_final,
                                    final_thinking.as_deref(),
                                    start,
                                    first_token_time,
                                    reported_prefill_tps,
                                    reported_gen_tps,
                                    reported_finish_reason.clone(),
                                );

                                on_chunk(clean_final, final_thinking, final_tool_calls, true, start.elapsed().as_millis() as u64, Some(final_metrics));
                                return Ok(());
                            }
                        }
                    }
                }

                let endpoint = format!("http://{}:{}/api/chat", ollama_host, ollama_port);
                let model_name = model.id.trim_start_matches("ollama/").to_string();
                let ollama_messages = Self::build_ollama_messages_payload(system_prompt, messages, params.mcp_tools.as_deref(), params.enable_thinking);

                #[cfg(target_os = "macos")]
                let num_gpu = params.gpu_layers;

                #[cfg(not(target_os = "macos"))]
                let num_gpu = if crate::core::hardware::SystemHardwareInfo::has_gpu_acceleration() {
                    params.gpu_layers
                } else {
                    0
                };

                if let Ok(client) = reqwest::Client::builder()
                    .connect_timeout(Duration::from_secs(15))
                    .timeout(Duration::from_secs(1800))
                    .build()
                {
                    let mut payload = json!({
                        "model": model_name,
                        "messages": ollama_messages,
                        "options": {
                            "temperature": params.temperature,
                            "top_p": params.top_p,
                            "num_predict": params.max_tokens,
                            "num_ctx": if params.context_length > 0 { params.context_length } else { 8192 },
                            "num_gpu": num_gpu
                        },
                        "stream": true
                    });

                    if let Some(enable_think) = params.enable_thinking {
                        payload["think"] = json!(enable_think);
                        if let Some(opts) = payload.get_mut("options").and_then(|o| o.as_object_mut()) {
                            opts.insert("think".to_string(), json!(enable_think));
                        }
                    }

                    if let Some(tools) = &params.mcp_tools {
                        let tools_payload: Vec<serde_json::Value> = tools.iter().filter(|t| t.enabled).map(|t| {
                            json!({
                                "type": "function",
                                "function": {
                                    "name": t.tool.name,
                                    "description": t.tool.description.clone().unwrap_or_default(),
                                    "parameters": t.tool.input_schema
                                }
                            })
                        }).collect();
                        if !tools_payload.is_empty() {
                            payload["tools"] = json!(tools_payload);
                        }
                    }

                    let req_num = REQ_STREAM_LOG_COUNTER.fetch_add(1, Ordering::Relaxed);
                    let prompt_log_id = format!("dev-prompt-{}", req_num);
                    let token_log_id = format!("dev-token-{}", req_num);
                    let (is_prompt_done, prompt_ticker) = Self::start_prompt_progress_ticker(
                        prompt_log_id.clone(),
                        model_name.clone(),
                        est_prompt_tokens,
                    );
                    let mut last_token_gen_log = std::time::Instant::now();

                    let t_req = std::time::Instant::now();
                    if let Ok(resp) = client.post(&endpoint).json(&payload).send().await {
                        let status = resp.status();
                        let latency = t_req.elapsed().as_millis() as u64;
                        crate::services::server_ctl::LocalServerController::record_log(
                            "POST",
                            &endpoint,
                            status.as_u16(),
                            latency,
                            est_prompt_tokens,
                            0,
                        );
                        if resp.status().is_success() {
                            let mut stream = resp.bytes_stream();
                            let mut state = StreamingThinkingState::new(false);
                            let mut buffer = String::new();
                            let mut accumulated_delta_tools: Vec<serde_json::Value> = Vec::new();

                            while let Some(chunk_result) = stream.next().await {
                                if abort_clone.load(Ordering::Relaxed) {
                                    log::info!("🛑 [Ollama] Inference streaming aborted by user.");
                                    break;
                                }
                                if let Ok(chunk_bytes) = chunk_result {
                                    let text = String::from_utf8_lossy(&chunk_bytes);
                                    buffer.push_str(&text);

                                    while let Some(line_end) = buffer.find('\n') {
                                        let line = buffer[..line_end].trim().to_string();
                                        buffer.drain(..line_end + 1);

                                        if !line.is_empty() {
                                            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&line) {
                                                if let Some(pec) = json_val.get("prompt_eval_count").and_then(|v| v.as_u64()) {
                                                    reported_prompt = Some(pec as usize);
                                                }
                                                if let Some(ec) = json_val.get("eval_count").and_then(|v| v.as_u64()) {
                                                    reported_comp = Some(ec as usize);
                                                }
                                                if let Some(tc) = json_val.get("prompt_tokens_cached").or_else(|| json_val.get("tokens_cached")).and_then(|v| v.as_u64()) {
                                                    reported_cached = Some(tc as usize);
                                                }
                                                if let (Some(pec), Some(ped)) = (json_val.get("prompt_eval_count").and_then(|v| v.as_f64()), json_val.get("prompt_eval_duration").and_then(|v| v.as_f64())) {
                                                    if ped > 0.0 {
                                                        reported_prefill_tps = Some((pec / (ped / 1_000_000_000.0)) as f32);
                                                    }
                                                }
                                                if let (Some(ec), Some(ed)) = (json_val.get("eval_count").and_then(|v| v.as_f64()), json_val.get("eval_duration").and_then(|v| v.as_f64())) {
                                                    if ed > 0.0 {
                                                        reported_gen_tps = Some((ec / (ed / 1_000_000_000.0)) as f32);
                                                    }
                                                }

                                                if let Some(dr) = json_val.get("done_reason").and_then(|v| v.as_str()) {
                                                    if !dr.is_empty() && dr != "null" {
                                                        reported_finish_reason = Some(dr.to_string());
                                                    }
                                                }

                                                let reasoning = json_val.get("message")
                                                    .and_then(|m| m.get("reasoning").or_else(|| m.get("thinking")))
                                                    .and_then(|s| s.as_str());
                                                let content = json_val.get("message")
                                                    .and_then(|m| m.get("content"))
                                                    .and_then(|s| s.as_str());

                                                let has_first_token = content.map(|c| !c.is_empty()).unwrap_or(false)
                                                    || reasoning.map(|r| !r.is_empty()).unwrap_or(false)
                                                    || json_val.get("message").and_then(|m| m.get("tool_calls")).and_then(|t| t.as_array()).map(|a| !a.is_empty()).unwrap_or(false);

                                                if first_token_time.is_none() && has_first_token {
                                                    first_token_time = Some(std::time::Instant::now());
                                                    is_prompt_done.store(true, Ordering::Relaxed);
                                                    prompt_ticker.abort();
                                                    Self::stop_active_prompt_ticker();
                                                    crate::services::server_ctl::LocalServerController::emit_prompt_progress(100.0);
                                                    crate::services::server_ctl::LocalServerController::update_or_record_dev_log(
                                                        &prompt_log_id,
                                                        "INFO",
                                                        Some(&model_name),
                                                        "Prompt processing progress: 100.0%",
                                                        None,
                                                    );
                                                }

                                                if let Some(tc_arr) = json_val.get("message").and_then(|m| m.get("tool_calls")).and_then(|t| t.as_array()) {
                                                    for tc_val in tc_arr {
                                                        accumulated_delta_tools.push(tc_val.clone());
                                                    }
                                                }

                                                state.push_delta(reasoning, content);
                                                let (c_acc, t_acc) = state.current(enable_thinking);
                                                let is_done = json_val.get("done").and_then(|d| d.as_bool()).unwrap_or(false);
                                                let dt_ref = if accumulated_delta_tools.is_empty() { None } else { Some(&accumulated_delta_tools) };
                                                let (clean_content, tool_calls) = Self::extract_tool_calls(&c_acc, dt_ref, params.mcp_tools.as_deref());
                                                
                                                let metrics = Self::compute_generation_metrics(
                                                    params,
                                                    est_prompt_tokens,
                                                    est_cached_tokens,
                                                    reported_prompt,
                                                    reported_cached,
                                                    reported_comp,
                                                    &clean_content,
                                                    t_acc.as_deref(),
                                                    start,
                                                    first_token_time,
                                                    reported_prefill_tps,
                                                    reported_gen_tps,
                                                    reported_finish_reason.clone(),
                                                );

                                                if let Some(ft) = first_token_time {
                                                    let current_toks = reported_comp.unwrap_or_else(|| {
                                                        let total_chars = c_acc.len() + t_acc.as_deref().map(|t| t.len()).unwrap_or(0);
                                                        (total_chars as f32 / 3.7).ceil() as usize
                                                    });
                                                    if current_toks > 0 && last_token_gen_log.elapsed() >= Duration::from_millis(250) {
                                                        last_token_gen_log = std::time::Instant::now();
                                                        let elapsed_sec = ft.elapsed().as_secs_f32().max(0.01);
                                                        let live_tps = reported_gen_tps.unwrap_or_else(|| (current_toks as f32 / elapsed_sec).max(0.1));
                                                        crate::services::server_ctl::LocalServerController::update_or_record_dev_log(
                                                            &token_log_id,
                                                            "INFO",
                                                            Some(&model_name),
                                                            &format!("Generating response: {} tokens ({:.1} tok/s)...", current_toks, live_tps),
                                                            None,
                                                        );
                                                    }
                                                }

                                                on_chunk(clean_content, t_acc, tool_calls, is_done, start.elapsed().as_millis() as u64, Some(metrics));
                                                if is_done {
                                                    is_prompt_done.store(true, Ordering::Relaxed);
                                                    prompt_ticker.abort();
                                                    Self::stop_active_prompt_ticker();
                                                    crate::services::server_ctl::LocalServerController::emit_prompt_progress(100.0);
                                                    return Ok(());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            is_prompt_done.store(true, Ordering::Relaxed);
                            prompt_ticker.abort();
                            Self::stop_active_prompt_ticker();

                            if abort_clone.load(Ordering::Relaxed) {
                                return Ok(());
                            }

                                let (final_content, final_thinking) = state.finalize(enable_thinking);
                                let dt_ref = if accumulated_delta_tools.is_empty() { None } else { Some(&accumulated_delta_tools) };
                                let (clean_final, final_tool_calls) = Self::extract_tool_calls(&final_content, dt_ref, params.mcp_tools.as_deref());
                                
                                let total_latency = start.elapsed().as_millis() as u64;
                                let comp_tokens = reported_comp.unwrap_or_else(|| {
                                    (clean_final.len() as f32 / 3.7).ceil() as usize
                                });
                                let final_tps_val = reported_gen_tps.unwrap_or_else(|| {
                                    if let Some(ft) = first_token_time {
                                        let elapsed_sec = ft.elapsed().as_secs_f32();
                                        if elapsed_sec > 0.05 { (comp_tokens as f32 / elapsed_sec).max(0.1) } else { 0.0 }
                                    } else {
                                        0.0
                                    }
                                });
                                let tps_info = if final_tps_val > 0.0 {
                                    format!(" ({:.1} tok/s)", final_tps_val)
                                } else {
                                    String::new()
                                };
                                crate::services::server_ctl::LocalServerController::update_or_record_dev_log(
                                    &token_log_id,
                                    "INFO",
                                    Some(&model_name),
                                    &format!("Finished streaming response: {} tokens generated{} in {}ms", comp_tokens, tps_info, total_latency),
                                    None,
                                );

                                let final_metrics = Self::compute_generation_metrics(
                                    params,
                                    est_prompt_tokens,
                                    est_cached_tokens,
                                    reported_prompt,
                                    reported_cached,
                                    reported_comp,
                                    &clean_final,
                                    final_thinking.as_deref(),
                                    start,
                                    first_token_time,
                                    reported_prefill_tps,
                                    reported_gen_tps,
                                    reported_finish_reason.clone(),
                                );

                                on_chunk(clean_final, final_thinking, final_tool_calls, true, start.elapsed().as_millis() as u64, Some(final_metrics));
                                return Ok(());
                            } else {
                                is_prompt_done.store(true, Ordering::Relaxed);
                                prompt_ticker.abort();
                            }
                        }
                    }

                // Fallback to non-streaming execution if HTTP stream fails
                if let Ok((content, thinking, tool_calls)) = Self::execute_stream(model, system_prompt, messages, params, mlx_host, mlx_port, ollama_host, ollama_port).await {
                    let metrics = Self::compute_generation_metrics(
                        params,
                        est_prompt_tokens,
                        est_cached_tokens,
                        None,
                        None,
                        None,
                        &content,
                        thinking.as_deref(),
                        start,
                        None,
                        None,
                        None,
                        None,
                    );
                    on_chunk(content, thinking, tool_calls, true, start.elapsed().as_millis() as u64, Some(metrics));
                    return Ok(());
                }
            }
            BackendType::Antigravity => {
                let agy_bin = Self::find_agy_binary().unwrap_or_else(|| std::path::PathBuf::from("agy"));
                let model_name = model.id.trim_start_matches("agy/").to_string();
                let prompt_text = Self::build_antigravity_prompt(system_prompt, messages, params.mcp_tools.as_deref());

                let mut cmd = silent_tokio_command(&agy_bin);
                cmd.env("PATH", Self::augmented_path());
                cmd.arg("--input-format").arg("stream-json");
                cmd.arg("--output-format").arg("stream-json");
                cmd.arg("--dangerously-skip-permissions");
                if !model_name.is_empty() {
                    cmd.arg(format!("--model={}", model_name));
                }
                cmd.stdin(std::process::Stdio::piped());
                cmd.stdout(std::process::Stdio::piped());
                cmd.stderr(std::process::Stdio::piped());

                let mut child = cmd.spawn().map_err(|e| format!("Falha ao iniciar o processo agy: {}", e))?;

                if let Some(mut stdin) = child.stdin.take() {
                    let input_event = json!({
                        "event": "user",
                        "message": {
                            "content": prompt_text
                        }
                    });
                    let mut payload = serde_json::to_vec(&input_event).unwrap_or_default();
                    payload.push(b'\n');
                    let _ = stdin.write_all(&payload).await;
                    let _ = stdin.flush().await;
                    drop(stdin);
                }

                if let Some(stdout) = child.stdout.take() {
                    let mut reader = BufReader::new(stdout).lines();
                    let mut state = StreamingThinkingState::new(false);
                    let enable_thinking = params.enable_thinking.unwrap_or(true);

                    while let Ok(Some(line)) = reader.next_line().await {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                            if let Some(event) = val.get("event").and_then(|e| e.as_str()) {
                                match event {
                                    "step_update" => {
                                        if let Some(step) = val.get("step_update") {
                                            if first_token_time.is_none() {
                                                first_token_time = Some(std::time::Instant::now());
                                            }

                                            // 1. Text delta
                                            let delta = step.get("text_delta").and_then(|d| d.as_str());
                                            if let Some(d) = delta {
                                                state.push_delta(None, Some(d));
                                            }

                                            // 2. If agy still reports tool activity, reflect in thinking so user sees progress
                                            let step_type = step.get("step_type").and_then(|t| t.as_str()).unwrap_or("");
                                            if step_type == "tool" {
                                                if let Some(tname) = step.get("tool_name").and_then(|n| n.as_str()) {
                                                    let st = step.get("state").and_then(|s| s.as_str()).unwrap_or("");
                                                    let note = if st == "ACTIVE" {
                                                        format!("🔧 Executando [{}]...\n", tname)
                                                    } else {
                                                        format!("✔ Concluído [{}].\n", tname)
                                                    };
                                                    state.push_delta(Some(&note), None);
                                                }
                                            }

                                            if let Some(usage) = step.get("usage") {
                                                if let Some(it) = usage.get("input_tokens").and_then(|t| t.as_u64()) {
                                                    reported_prompt = Some(it as usize);
                                                }
                                                if let Some(ot) = usage.get("output_tokens").and_then(|t| t.as_u64()) {
                                                    reported_comp = Some(ot as usize);
                                                }
                                            }

                                            let (c_acc, t_acc) = state.current(enable_thinking);
                                            let (clean_content, tool_calls) = Self::extract_tool_calls(&c_acc, None, params.mcp_tools.as_deref());
                                            let metrics = Self::compute_generation_metrics(
                                                params,
                                                est_prompt_tokens,
                                                est_cached_tokens,
                                                reported_prompt,
                                                reported_cached,
                                                reported_comp,
                                                &clean_content,
                                                t_acc.as_deref(),
                                                start,
                                                first_token_time,
                                                reported_prefill_tps,
                                                reported_gen_tps,
                                                reported_finish_reason.clone(),
                                            );
                                            on_chunk(clean_content, t_acc, tool_calls, false, start.elapsed().as_millis() as u64, Some(metrics));
                                        }
                                    }
                                    "result" => {
                                        if let Some(res) = val.get("result") {
                                            let status = res.get("status").and_then(|s| s.as_str()).unwrap_or("");
                                            if status == "SUCCESS" {
                                                reported_finish_reason = Some("stop".to_string());
                                            }
                                            if let Some(usage) = res.get("usage") {
                                                if let Some(it) = usage.get("input_tokens").and_then(|t| t.as_u64()) {
                                                    reported_prompt = Some(it as usize);
                                                }
                                                if let Some(ot) = usage.get("output_tokens").and_then(|t| t.as_u64()) {
                                                    reported_comp = Some(ot as usize);
                                                }
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }

                    let _ = child.wait().await;

                    let (final_content, final_thinking) = state.finalize(enable_thinking);
                    let (clean_final, final_tool_calls) = Self::extract_tool_calls(&final_content, None, params.mcp_tools.as_deref());
                    let final_metrics = Self::compute_generation_metrics(
                        params,
                        est_prompt_tokens,
                        est_cached_tokens,
                        reported_prompt,
                        reported_cached,
                        reported_comp,
                        &clean_final,
                        final_thinking.as_deref(),
                        start,
                        first_token_time,
                        reported_prefill_tps,
                        reported_gen_tps,
                        reported_finish_reason.clone(),
                    );
                    on_chunk(clean_final, final_thinking, final_tool_calls, true, start.elapsed().as_millis() as u64, Some(final_metrics));
                    return Ok(());
                }
            }
            BackendType::CloudOpenAi => {
                let config = crate::core::config::AppConfig::load();
                let is_gemini = model.id.starts_with("gemini/");
                let is_groq = model.id.starts_with("groq/");
                let (base_url, api_key, model_name, is_openrouter) = if model.id.starts_with("openai/") {
                    (
                        config.cloud_providers.openai_base_url.as_deref().unwrap_or("https://api.openai.com/v1").to_string(),
                        config.cloud_providers.openai_api_key.trim().to_string(),
                        model.id.trim_start_matches("openai/").to_string(),
                        false,
                    )
                } else if model.id.starts_with("openrouter/") {
                    (
                        "https://openrouter.ai/api/v1".to_string(),
                        config.cloud_providers.openrouter_api_key.trim().to_string(),
                        model.id.trim_start_matches("openrouter/").to_string(),
                        true,
                    )
                } else if is_gemini {
                    (
                        "https://generativelanguage.googleapis.com/v1beta/openai".to_string(),
                        config.cloud_providers.gemini_api_key.trim().to_string(),
                        model.id.trim_start_matches("gemini/").to_string(),
                        false,
                    )
                } else if model.id.starts_with("groq/") {
                    (
                        "https://api.groq.com/openai/v1".to_string(),
                        config.cloud_providers.groq_api_key.trim().to_string(),
                        model.id.trim_start_matches("groq/").to_string(),
                        false,
                    )
                } else if model.id.starts_with("custom/") {
                    let rest = model.id.trim_start_matches("custom/");
                    if let Some((provider_id, model_sub)) = rest.split_once('/') {
                        if let Some(prov) = config.cloud_providers.custom_providers.iter().find(|p| p.id == provider_id) {
                            (
                                prov.base_url.trim().to_string(),
                                prov.api_key.trim().to_string(),
                                model_sub.to_string(),
                                false,
                            )
                        } else {
                            (
                                config.cloud_providers.custom_base_url.trim().to_string(),
                                config.cloud_providers.custom_api_key.trim().to_string(),
                                model_sub.to_string(),
                                false,
                            )
                        }
                    } else {
                        (
                            config.cloud_providers.custom_base_url.trim().to_string(),
                            config.cloud_providers.custom_api_key.trim().to_string(),
                            rest.to_string(),
                            false,
                        )
                    }
                } else {
                    let default_url = model.local_path.clone().unwrap_or_else(|| "https://api.openai.com/v1".to_string());
                    (default_url, String::new(), model.id.clone(), false)
                };

                if base_url.is_empty() {
                    return Err("URL Base do provedor em nuvem não configurada.".to_string());
                }

                let endpoint = format!("{}/chat/completions", base_url.trim_end_matches('/'));
                if let Ok(client) = reqwest::Client::builder()
                    .connect_timeout(Duration::from_secs(15))
                    .timeout(Duration::from_secs(1800))
                    .build()
                {
                    let mut payload = json!({
                        "model": model_name,
                        "messages": json_messages,
                        "temperature": params.temperature,
                        "top_p": params.top_p,
                        "max_tokens": params.max_tokens,
                        "stream": true,
                        "stream_options": { "include_usage": true },
                    });

                    // Groq supports reasoning_format on reasoning models (e.g. DeepSeek-R1)
                    if is_groq && model.supports_thinking {
                        if let Some(false) = params.enable_thinking {
                            payload["reasoning_format"] = json!("hidden");
                        } else {
                            payload["reasoning_format"] = json!("parsed");
                        }
                    }

                    if let Some(tools) = &params.mcp_tools {
                        let tools_payload: Vec<serde_json::Value> = tools.iter().filter(|t| t.enabled).map(|t| {
                            json!({
                                "type": "function",
                                "function": {
                                    "name": t.tool.name,
                                    "description": t.tool.description.clone().unwrap_or_default(),
                                    "parameters": t.tool.input_schema
                                }
                            })
                        }).collect();
                        if !tools_payload.is_empty() {
                            payload["tools"] = json!(tools_payload);
                            payload["tool_choice"] = json!("auto");
                        }
                    }

                    let mut req = client.post(&endpoint).json(&payload);
                    if !api_key.is_empty() {
                        req = req.header("Authorization", format!("Bearer {}", api_key));
                    }
                    if is_openrouter {
                        req = req.header("HTTP-Referer", "https://atena.studio")
                                 .header("X-Title", "Atena Studio");
                    }

                    let t_cloud = std::time::Instant::now();
                    let send_res = req.send().await;
                    let latency = t_cloud.elapsed().as_millis() as u64;
                    if let Ok(ref resp) = send_res {
                        crate::services::server_ctl::LocalServerController::record_log(
                            "POST",
                            &endpoint,
                            resp.status().as_u16(),
                            latency,
                            est_prompt_tokens,
                            0,
                        );
                    } else {
                        crate::services::server_ctl::LocalServerController::record_log(
                            "POST",
                            &endpoint,
                            502,
                            latency,
                            est_prompt_tokens,
                            0,
                        );
                    }
                    match send_res {
                        Ok(resp) => {
                            if !resp.status().is_success() {
                                let status_code = resp.status();
                                let err_body = resp.text().await.unwrap_or_default();
                                return Err(format!("Provedor Cloud retornou erro (HTTP {}): {}", status_code, err_body));
                            }

                            let mut stream = resp.bytes_stream();
                            let mut state = StreamingThinkingState::new(false);
                            let mut buffer = String::new();
                            let mut accumulated_delta_tools: Vec<serde_json::Value> = Vec::new();

                            while let Some(chunk_result) = stream.next().await {
                                if let Ok(chunk_bytes) = chunk_result {
                                    let text = String::from_utf8_lossy(&chunk_bytes);
                                    buffer.push_str(&text);

                                    while let Some(line_end) = buffer.find('\n') {
                                        let line = buffer[..line_end].trim().to_string();
                                        buffer.drain(..line_end + 1);

                                        if line.starts_with("data: ") {
                                            let data_str = line.trim_start_matches("data: ").trim();
                                            if data_str == "[DONE]" {
                                                break;
                                            }
                                            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(data_str) {
                                                if let Some(usage) = json_val.get("usage") {
                                                    if let Some(pt) = usage.get("prompt_tokens").and_then(|v| v.as_u64()) {
                                                        reported_prompt = Some(pt as usize);
                                                    }
                                                    if let Some(ct) = usage.get("completion_tokens").and_then(|v| v.as_u64()) {
                                                        reported_comp = Some(ct as usize);
                                                    }
                                                    if let Some(cd) = usage.get("prompt_tokens_details").and_then(|d| d.get("cached_tokens")).and_then(|v| v.as_u64()) {
                                                        reported_cached = Some(cd as usize);
                                                    }
                                                }

                                                if let Some(choices_arr) = json_val.get("choices").and_then(|c| c.as_array()) {
                                                    if let Some(first_choice) = choices_arr.first() {
                                                        if let Some(fr) = first_choice.get("finish_reason").and_then(|r| r.as_str()) {
                                                            if !fr.is_empty() && fr != "null" {
                                                                reported_finish_reason = Some(fr.to_string());
                                                            }
                                                        }
                                                    }
                                                }

                                                if let Some(delta) = json_val.get("choices")
                                                    .and_then(|c| c.as_array())
                                                    .and_then(|a| a.first())
                                                    .and_then(|c| c.get("delta"))
                                                {
                                                    let reasoning = delta.get("reasoning_content")
                                                        .or_else(|| delta.get("reasoning"))
                                                        .or_else(|| delta.get("thinking"))
                                                        .and_then(|s| s.as_str());
                                                    let content = delta.get("content").and_then(|s| s.as_str());

                                                    if first_token_time.is_none() && (content.map(|c| !c.is_empty()).unwrap_or(false) || reasoning.map(|r| !r.is_empty()).unwrap_or(false)) {
                                                        first_token_time = Some(std::time::Instant::now());
                                                    }

                                                    if let Some(tool_calls_arr) = delta.get("tool_calls").and_then(|t| t.as_array()) {
                                                        for tc_val in tool_calls_arr {
                                                            accumulated_delta_tools.push(tc_val.clone());
                                                        }
                                                    }

                                                    state.push_delta(reasoning, content);
                                                    let (c_acc, t_acc) = state.current(enable_thinking);
                                                    let dt_ref = if accumulated_delta_tools.is_empty() { None } else { Some(&accumulated_delta_tools) };
                                                    let (clean_content, tool_calls) = Self::extract_tool_calls(&c_acc, dt_ref, params.mcp_tools.as_deref());

                                                    let metrics = Self::compute_generation_metrics(
                                                        params,
                                                        est_prompt_tokens,
                                                        est_cached_tokens,
                                                        reported_prompt,
                                                        reported_cached,
                                                        reported_comp,
                                                        &clean_content,
                                                        t_acc.as_deref(),
                                                        start,
                                                        first_token_time,
                                                        reported_prefill_tps,
                                                        reported_gen_tps,
                                                        reported_finish_reason.clone(),
                                                    );

                                                    on_chunk(clean_content, t_acc, tool_calls, false, start.elapsed().as_millis() as u64, Some(metrics));
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            let (final_content, final_thinking) = state.finalize(enable_thinking);
                            let dt_ref = if accumulated_delta_tools.is_empty() { None } else { Some(&accumulated_delta_tools) };
                            let (clean_final, final_tool_calls) = Self::extract_tool_calls(&final_content, dt_ref, params.mcp_tools.as_deref());

                            let final_metrics = Self::compute_generation_metrics(
                                params,
                                est_prompt_tokens,
                                est_cached_tokens,
                                reported_prompt,
                                reported_cached,
                                reported_comp,
                                &clean_final,
                                final_thinking.as_deref(),
                                start,
                                first_token_time,
                                reported_prefill_tps,
                                reported_gen_tps,
                                reported_finish_reason.clone(),
                            );
                            on_chunk(clean_final, final_thinking, final_tool_calls, true, start.elapsed().as_millis() as u64, Some(final_metrics));
                            return Ok(());
                        }
                        Err(e) => return Err(format!("Falha ao conectar com o endpoint Cloud ({}): {}", endpoint, e)),
                    }
                }
            }
        }

        Err(format!(
            "Não foi possível obter resposta do modelo {}. Verifique se o modelo está carregado ou inicie o servidor na aba Servidor Local.",
            model.name
        ))
    }

    /// Executes inference against the appropriate backend (MLX-LM or Ollama or direct CLI runner)
    /// Returns (response_content, thinking_content, tool_calls) after parsing any <think> and <tool_call> blocks.
    pub async fn execute_stream(
        model: &ModelInfo,
        system_prompt: &str,
        messages: &[crate::core::model::ChatMessage],
        params: &InferenceParams,
        mlx_host: &str,
        mlx_port: u16,
        ollama_host: &str,
        ollama_port: u16,
    ) -> Result<(String, Option<String>, Option<Vec<crate::core::mcp::McpToolCall>>), String> {
        let json_messages = Self::build_messages_payload(system_prompt, messages, params.mcp_tools.as_deref(), params.enable_thinking);
        match model.backend {
            BackendType::MlxLm => {
                // 1. Try HTTP API if server is up
                let endpoint = format!("http://{}:{}/v1/chat/completions", mlx_host, mlx_port);
                if let Ok(client) = reqwest::Client::builder().timeout(Duration::from_secs(30)).build() {
                    let mut payload = json!({
                        "model": model.local_path.clone().unwrap_or_else(|| model.id.clone()),
                        "messages": json_messages,
                        "temperature": params.temperature,
                        "top_p": params.top_p,
                        "max_tokens": params.max_tokens,
                        "stream": false
                    });

                    if let Some(enable_think) = params.enable_thinking {
                        payload["enable_thinking"] = json!(enable_think);
                        payload["chat_template_args"] = json!({ "enable_thinking": enable_think });
                    }

                    let t_req = std::time::Instant::now();
                    if let Ok(resp) = client.post(&endpoint).json(&payload).send().await {
                        let latency = t_req.elapsed().as_millis() as u64;
                        crate::services::server_ctl::LocalServerController::record_log(
                            "POST",
                            &endpoint,
                            resp.status().as_u16(),
                            latency,
                            json_messages.len(),
                            0,
                        );
                        if resp.status().is_success() {
                            if let Ok(json) = resp.json::<serde_json::Value>().await {
                                if let Some(msg_obj) = json.get("choices")
                                    .and_then(|c| c.as_array())
                                    .and_then(|a| a.first())
                                    .and_then(|c| c.get("message"))
                                {
                                    let content = msg_obj.get("content").and_then(|s| s.as_str()).unwrap_or("");
                                    let explicit_reasoning = msg_obj.get("reasoning")
                                        .or_else(|| msg_obj.get("reasoning_content"))
                                        .or_else(|| msg_obj.get("thinking"))
                                        .and_then(|s| s.as_str())
                                        .map(|s| s.trim().to_string());

                                    let (thinking, clean) = if let Some(r) = explicit_reasoning {
                                        let clean_text = StreamingThinkingState::strip_special_channel_tokens(content).trim().to_string();
                                        (Some(r), clean_text)
                                    } else {
                                        Self::parse_thinking(content)
                                    };

                                    let (clean_text, tool_calls) = Self::extract_tool_calls(&clean, None, params.mcp_tools.as_deref());
                                    return Ok((clean_text, thinking, tool_calls));
                                }
                            }
                        }
                    }
                }

                // 2. Direct CLI execution via uv / mlx_lm.generate / mlx_vlm.generate
                if let Some(path) = &model.local_path {
                    let mut formatted_prompt = String::new();
                    formatted_prompt.push_str("<｜begin of sentence｜>");
                    if !system_prompt.trim().is_empty() {
                        formatted_prompt.push_str(&format!("<｜System｜>{}\n", system_prompt));
                    }
                    for msg in messages {
                        if msg.role == "user" {
                            formatted_prompt.push_str(&format!("<｜User｜>{}\n", msg.content));
                        } else if msg.role == "assistant" {
                            formatted_prompt.push_str(&format!("<｜Assistant｜>{}\n", msg.content));
                        }
                    }
                    formatted_prompt.push_str("<｜Assistant｜>");

                    let max_toks = params.max_tokens.min(1024).to_string();
                    let temp_str = format!("{:.2}", params.temperature);
                    let aug_path = Self::augmented_path();
                    let is_vlm = Self::is_vlm_or_unified_model(path);

                    let module_name = if is_vlm { "mlx_vlm.generate" } else { "mlx_lm.generate" };
                    let output = if let Some(py_path) = crate::services::runtime::RuntimeManager::isolated_python() {
                        Command::new(&py_path)
                            .env("PATH", &aug_path)
                            .arg("-m")
                            .arg(module_name)
                            .arg("--model")
                            .arg(path)
                            .arg("--prompt")
                            .arg(&formatted_prompt)
                            .arg("--max-tokens")
                            .arg(&max_toks)
                            .arg("--temp")
                            .arg(&temp_str)
                            .output()
                            .await
                    } else if let Some(uv_path) = crate::services::runtime::RuntimeManager::resolve_binary("uv") {
                        Command::new(&uv_path)
                            .env("PATH", &aug_path)
                            .arg("run")
                            .arg("--with")
                            .arg(if is_vlm { "mlx-vlm" } else { "mlx-lm" })
                            .arg("--with")
                            .arg("jinja2")
                            .arg("python3")
                            .arg("-m")
                            .arg(module_name)
                            .arg("--model")
                            .arg(path)
                            .arg("--prompt")
                            .arg(&formatted_prompt)
                            .arg("--max-tokens")
                            .arg(&max_toks)
                            .arg("--temp")
                            .arg(&temp_str)
                            .output()
                            .await
                    } else {
                        Command::new("python3")
                            .env("PATH", &aug_path)
                            .arg("-m")
                            .arg(module_name)
                            .arg("--model")
                            .arg(path)
                            .arg("--prompt")
                            .arg(&formatted_prompt)
                            .arg("--max-tokens")
                            .arg(&max_toks)
                            .arg("--temp")
                            .arg(&temp_str)
                            .output()
                            .await
                    };

                    if let Ok(out) = output {
                        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                        if !stdout.is_empty() {
                            let parsed = Self::parse_mlx_generate_output(&stdout);
                            if !parsed.is_empty() {
                                let (thinking, clean) = Self::parse_thinking(&parsed);
                                let (clean_text, tool_calls) = Self::extract_tool_calls(&clean, None, params.mcp_tools.as_deref());
                                return Ok((clean_text, thinking, tool_calls));
                            }
                        }
                    }
                }
            }
            BackendType::Ollama => {
                // 1. Try Ollama HTTP API
                let endpoint = format!("http://{}:{}/api/chat", ollama_host, ollama_port);
                let model_name = model.id.trim_start_matches("ollama/").to_string();

                #[cfg(target_os = "macos")]
                let num_gpu = params.gpu_layers;

                #[cfg(not(target_os = "macos"))]
                let num_gpu = if crate::core::hardware::SystemHardwareInfo::has_gpu_acceleration() {
                    params.gpu_layers
                } else {
                    0
                };

                if let Ok(client) = reqwest::Client::builder().timeout(Duration::from_secs(30)).build() {
                    let payload = json!({
                        "model": model_name,
                        "messages": json_messages,
                        "options": {
                            "temperature": params.temperature,
                            "top_p": params.top_p,
                            "num_predict": params.max_tokens,
                            "num_ctx": if params.context_length > 0 { params.context_length } else { 8192 },
                            "num_gpu": num_gpu
                        },

                        "stream": false
                    });

                    let t_req = std::time::Instant::now();
                    if let Ok(resp) = client.post(&endpoint).json(&payload).send().await {
                        let latency = t_req.elapsed().as_millis() as u64;
                        crate::services::server_ctl::LocalServerController::record_log(
                            "POST",
                            &endpoint,
                            resp.status().as_u16(),
                            latency,
                            json_messages.len(),
                            0,
                        );
                        if resp.status().is_success() {
                            if let Ok(json) = resp.json::<serde_json::Value>().await {
                                if let Some(content) = json.get("message").and_then(|m| m.get("content")).and_then(|s| s.as_str()) {
                                    let (thinking, clean) = Self::parse_thinking(content);
                                    let (clean_text, tool_calls) = Self::extract_tool_calls(&clean, None, params.mcp_tools.as_deref());
                                    return Ok((clean_text, thinking, tool_calls));
                                }
                            }
                        }
                    }
                }

                // 2. Direct CLI fallback: ollama run <model> <prompt>
                let last_user_prompt = messages
                    .iter()
                    .rev()
                    .find(|m| m.role == "user")
                    .map(|m| m.content.as_str())
                    .unwrap_or("");

                let aug_path = Self::augmented_path();
                if let Ok(out) = silent_tokio_command("ollama")
                    .env("PATH", &aug_path)
                    .arg("run")
                    .arg(&model_name)
                    .arg(last_user_prompt)
                    .output()
                    .await
                {
                    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if !stdout.is_empty() {
                        let (thinking, clean) = Self::parse_thinking(&stdout);
                        let (clean_text, tool_calls) = Self::extract_tool_calls(&clean, None, params.mcp_tools.as_deref());
                        return Ok((clean_text, thinking, tool_calls));
                    }
                }
            }
            BackendType::LocalNative => {}
            BackendType::Antigravity => {
                let agy_bin = Self::find_agy_binary().unwrap_or_else(|| std::path::PathBuf::from("agy"));
                let model_name = model.id.trim_start_matches("agy/").to_string();
                let prompt_text = Self::build_antigravity_prompt(system_prompt, messages, params.mcp_tools.as_deref());

                let mut cmd = silent_tokio_command(&agy_bin);
                cmd.env("PATH", Self::augmented_path());
                cmd.arg("--output-format=json");
                cmd.arg("--dangerously-skip-permissions");
                if !model_name.is_empty() {
                    cmd.arg(format!("--model={}", model_name));
                }
                cmd.arg(format!("-p={}", prompt_text));

                if let Ok(out) = cmd.output().await {
                    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if !stdout.is_empty() {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&stdout) {
                            if let Some(resp) = val.get("response").and_then(|r| r.as_str()) {
                                let (thinking, clean) = Self::parse_thinking(resp);
                                let (clean_text, tool_calls) = Self::extract_tool_calls(&clean, None, params.mcp_tools.as_deref());
                                return Ok((clean_text, thinking, tool_calls));
                            }
                        }
                        let (thinking, clean) = Self::parse_thinking(&stdout);
                        let (clean_text, tool_calls) = Self::extract_tool_calls(&clean, None, params.mcp_tools.as_deref());
                        return Ok((clean_text, thinking, tool_calls));
                    }
                }
            }
            BackendType::CloudOpenAi => {
                let config = crate::core::config::AppConfig::load();
                let is_groq = model.id.starts_with("groq/");
                let (base_url, api_key, model_name, is_openrouter) = if model.id.starts_with("openai/") {
                    (
                        config.cloud_providers.openai_base_url.as_deref().unwrap_or("https://api.openai.com/v1").to_string(),
                        config.cloud_providers.openai_api_key.trim().to_string(),
                        model.id.trim_start_matches("openai/").to_string(),
                        false,
                    )
                } else if model.id.starts_with("openrouter/") {
                    (
                        "https://openrouter.ai/api/v1".to_string(),
                        config.cloud_providers.openrouter_api_key.trim().to_string(),
                        model.id.trim_start_matches("openrouter/").to_string(),
                        true,
                    )
                } else if model.id.starts_with("gemini/") {
                    (
                        "https://generativelanguage.googleapis.com/v1beta/openai".to_string(),
                        config.cloud_providers.gemini_api_key.trim().to_string(),
                        model.id.trim_start_matches("gemini/").to_string(),
                        false,
                    )
                } else if model.id.starts_with("groq/") {
                    (
                        "https://api.groq.com/openai/v1".to_string(),
                        config.cloud_providers.groq_api_key.trim().to_string(),
                        model.id.trim_start_matches("groq/").to_string(),
                        false,
                    )
                } else if model.id.starts_with("custom/") {
                    let rest = model.id.trim_start_matches("custom/");
                    if let Some((provider_id, model_sub)) = rest.split_once('/') {
                        if let Some(prov) = config.cloud_providers.custom_providers.iter().find(|p| p.id == provider_id) {
                            (
                                prov.base_url.trim().to_string(),
                                prov.api_key.trim().to_string(),
                                model_sub.to_string(),
                                false,
                            )
                        } else {
                            (
                                config.cloud_providers.custom_base_url.trim().to_string(),
                                config.cloud_providers.custom_api_key.trim().to_string(),
                                model_sub.to_string(),
                                false,
                            )
                        }
                    } else {
                        (
                            config.cloud_providers.custom_base_url.trim().to_string(),
                            config.cloud_providers.custom_api_key.trim().to_string(),
                            rest.to_string(),
                            false,
                        )
                    }
                } else {
                    let default_url = model.local_path.clone().unwrap_or_else(|| "https://api.openai.com/v1".to_string());
                    (default_url, String::new(), model.id.clone(), false)
                };

                let endpoint = format!("{}/chat/completions", base_url.trim_end_matches('/'));
                let client = reqwest::Client::builder().timeout(Duration::from_secs(60)).build()
                    .map_err(|e| format!("Erro ao criar cliente HTTP: {}", e))?;

                let mut payload = json!({
                    "model": model_name,
                    "messages": json_messages,
                    "temperature": params.temperature,
                    "top_p": params.top_p,
                    "max_tokens": params.max_tokens,
                });

                if is_groq && model.supports_thinking {
                    if let Some(false) = params.enable_thinking {
                        payload["reasoning_format"] = json!("hidden");
                    } else {
                        payload["reasoning_format"] = json!("parsed");
                    }
                }

                let mut req = client.post(&endpoint).json(&payload);
                if !api_key.is_empty() {
                    req = req.header("Authorization", format!("Bearer {}", api_key));
                }
                if is_openrouter {
                    req = req.header("HTTP-Referer", "https://atena.studio")
                             .header("X-Title", "Atena Studio");
                }

                let t_cloud = std::time::Instant::now();
                let resp = req.send().await.map_err(|e| format!("Erro de rede: {}", e))?;
                let latency = t_cloud.elapsed().as_millis() as u64;
                crate::services::server_ctl::LocalServerController::record_log(
                    "POST",
                    &endpoint,
                    resp.status().as_u16(),
                    latency,
                    json_messages.len(),
                    0,
                );
                if !resp.status().is_success() {
                    let status = resp.status();
                    let err = resp.text().await.unwrap_or_default();
                    return Err(format!("Erro Cloud ({}): {}", status, err));
                }

                let val: serde_json::Value = resp.json().await.map_err(|e| format!("Invalid JSON: {}", e))?;
                let msg_obj = val.get("choices")
                    .and_then(|c| c.as_array())
                    .and_then(|a| a.first())
                    .and_then(|c| c.get("message"));

                let content = msg_obj
                    .and_then(|m| m.get("content"))
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string();

                let explicit_reasoning = msg_obj
                    .and_then(|m| m.get("reasoning").or_else(|| m.get("reasoning_content")).or_else(|| m.get("thinking")))
                    .and_then(|s| s.as_str())
                    .map(|s| s.trim().to_string());

                let (thinking, clean) = if let Some(r) = explicit_reasoning {
                    let clean_text = StreamingThinkingState::strip_special_channel_tokens(&content).trim().to_string();
                    (Some(r), clean_text)
                } else {
                    Self::parse_thinking(&content)
                };
                let (clean_text, tool_calls) = Self::extract_tool_calls(&clean, None, params.mcp_tools.as_deref());
                return Ok((clean_text, thinking, tool_calls));
            }
        }

        // Return a clear helpful error if the backend execution couldn't produce an output
        Err(format!(
            "Não foi possível obter resposta do modelo {}. Verifique se o modelo está carregado ou inicie o servidor na aba Servidor Local.",
            model.name
        ))
    }

    /// Tests connection to a cloud provider
    pub async fn test_cloud_provider_connection(
        provider: &str,
        api_key: &str,
        base_url: Option<&str>,
    ) -> Result<String, String> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(8))
            .build()
            .map_err(|e| format!("Erro ao inicializar cliente HTTP: {}", e))?;

        match provider {
            "openai" => {
                let url = format!("{}/models", base_url.unwrap_or("https://api.openai.com/v1").trim_end_matches('/'));
                let mut req = client.get(&url);
                if !api_key.trim().is_empty() {
                    req = req.header("Authorization", format!("Bearer {}", api_key.trim()));
                }
                let resp = req.send().await.map_err(|e| format!("Falha de conexão: {}", e))?;
                if resp.status().is_success() {
                    Ok("Conexão com OpenAI estabelecida com sucesso! API Key válida.".to_string())
                } else if resp.status().as_u16() == 401 {
                    Err("Chave de API inválida ou expirada (HTTP 401 Unauthorized).".to_string())
                } else {
                    Err(format!("OpenAI retornou status HTTP {}: {}", resp.status(), resp.text().await.unwrap_or_default()))
                }
            }
            "gemini" => {
                let url = "https://generativelanguage.googleapis.com/v1beta/openai/models";
                let req = client.get(url)
                    .header("Authorization", format!("Bearer {}", api_key.trim()));
                let resp = req.send().await.map_err(|e| format!("Falha de conexão com Google Gemini: {}", e))?;
                if resp.status().is_success() {
                    Ok("Conexão com Google Gemini estabelecida com sucesso! API Key válida.".to_string())
                } else if resp.status().as_u16() == 400 || resp.status().as_u16() == 401 || resp.status().as_u16() == 403 {
                    Err("Chave de API do Gemini inválida ou sem permissão (HTTP 401/403).".to_string())
                } else {
                    Err(format!("Google Gemini retornou status HTTP {}: {}", resp.status(), resp.text().await.unwrap_or_default()))
                }
            }
            "openrouter" => {
                let url = "https://openrouter.ai/api/v1/auth/key";
                let req = client.get(url)
                    .header("Authorization", format!("Bearer {}", api_key.trim()))
                    .header("HTTP-Referer", "https://atena.studio")
                    .header("X-Title", "Atena Studio");
                let resp = req.send().await.map_err(|e| format!("Falha de conexão: {}", e))?;
                if resp.status().is_success() {
                    Ok("Conexão com OpenRouter autenticada com sucesso!".to_string())
                } else if resp.status().as_u16() == 401 {
                    Err("Chave de API do OpenRouter inválida (HTTP 401).".to_string())
                } else {
                    Err(format!("OpenRouter retornou status HTTP {}: {}", resp.status(), resp.text().await.unwrap_or_default()))
                }
            }
            "groq" => {
                let url = "https://api.groq.com/openai/v1/models";
                let req = client.get(url)
                    .header("Authorization", format!("Bearer {}", api_key.trim()));
                let resp = req.send().await.map_err(|e| format!("Falha de conexão com Groq: {}", e))?;
                if resp.status().is_success() {
                    Ok("Conexão com Groq Cloud estabelecida com sucesso! API Key válida.".to_string())
                } else if resp.status().as_u16() == 401 {
                    Err("Chave de API da Groq inválida (HTTP 401 Unauthorized).".to_string())
                } else {
                    Err(format!("Groq retornou status HTTP {}: {}", resp.status(), resp.text().await.unwrap_or_default()))
                }
            }
            "ollama" => {
                let host = base_url.unwrap_or("http://127.0.0.1:11434");
                let url = format!("{}/api/tags", host.trim_end_matches('/'));
                let resp = client.get(&url).send().await.map_err(|e| format!("Ollama indisponível em {}: {}", host, e))?;
                if resp.status().is_success() {
                    Ok(format!("Instância do Ollama online e respondendo em {}!", host))
                } else {
                    Err(format!("Ollama retornou status HTTP {}.", resp.status()))
                }
            }
            "antigravity" => {
                let status = Self::check_agy_session().await;
                if status.authenticated {
                    Ok("Sessão do Google Antigravity autenticada e pronta!".to_string())
                } else if let Some(err) = status.error_message {
                    Err(err)
                } else {
                    Err("Antigravity não conectado. Execute o login pelo terminal.".to_string())
                }
            }
            "custom" => {
                let host = base_url.ok_or_else(|| "URL Base não informada para o provedor customizado.".to_string())?;
                let url = format!("{}/models", host.trim_end_matches('/'));
                let mut req = client.get(&url);
                if !api_key.trim().is_empty() {
                    req = req.header("Authorization", format!("Bearer {}", api_key.trim()));
                }
                let resp = req.send().await.map_err(|e| format!("Falha de conexão com {}: {}", host, e))?;
                if resp.status().is_success() {
                    Ok(format!("Endpoint customizado respondendo em {}!", host))
                } else {
                    Err(format!("Endpoint retornou status HTTP {}.", resp.status()))
                }
            }
            _ => Err(format!("Provedor '{}' não reconhecido.", provider)),
        }
    }

    /// Parses the raw output from mlx_lm.generate
    fn parse_mlx_generate_output(raw: &str) -> String {
        // Output typically has format:
        // [transformers] ...
        // ==========
        // <think>
        // ...
        // </think>
        // Actual answer
        // ==========
        // Prompt: X tokens ...
        // Generation: Y tokens ...
        let parts: Vec<&str> = raw.split("==========").collect();
        if parts.len() >= 3 {
            let body = parts[1].trim();
            return body.to_string();
        } else if parts.len() == 2 {
            let body = parts[1].trim();
            return body.to_string();
        }

        // If no delimiters found, return raw trimmed
        raw.lines()
            .filter(|line| !line.starts_with("[transformers]") && !line.starts_with("Prompt:") && !line.starts_with("Generation:") && !line.starts_with("Peak memory:"))
            .collect::<Vec<&str>>()
            .join("\n")
            .trim()
            .to_string()
    }

    /// Extracts thinking/reasoning content from model output.
    /// Supports `<|channel>thought...<channel|>`, `<thought>...</thought>`, `<think>...</think>` and `<thinking>...</thinking>` tag formats.
    /// Returns `(Option<thinking_content>, clean_response)`.
    pub fn parse_thinking(raw: &str) -> (Option<String>, String) {
        let tag_pairs: &[(&str, &str)] = &[
            ("<|channel>thought", "<channel|>"),
            ("<|channel|>thought", "<|channel|>"),
            ("<|thought|>", "<|/thought|>"),
            ("<|thought|>", "<|end_of_thought|>"),
            ("<|start_thought|>", "<|end_thought|>"),
            ("<thought>", "</thought>"),
            ("<think>", "</think>"),
            ("<thinking>", "</thinking>"),
        ];

        for &(start_tag, end_tag) in tag_pairs {
            if let Some(start) = raw.find(start_tag) {
                if let Some(end) = raw[start + start_tag.len()..].find(end_tag) {
                    let actual_end = start + start_tag.len() + end;
                    let think_content = raw[start + start_tag.len()..actual_end].trim();
                    let before = &raw[..start];
                    let after = &raw[actual_end + end_tag.len()..];
                    let clean_raw = format!("{}{}", before, after);
                    let clean = StreamingThinkingState::strip_special_channel_tokens(&clean_raw).trim().to_string();
                    let clean_thinking = StreamingThinkingState::strip_special_channel_tokens(think_content).trim().to_string();
                    let thinking = if clean_thinking.is_empty() {
                        None
                    } else {
                        Some(clean_thinking)
                    };
                    return (thinking, clean);
                }
            }
        }

        // Fallback: se o modelo começou em pensamento sem tag de abertura no corpo gerado, mas fechou com </think> ou similar
        for &close_tag in StreamingThinkingState::CLOSE_TAGS {
            if let Some(end_pos) = raw.find(close_tag) {
                let think_content = raw[..end_pos].trim();
                let after = &raw[end_pos + close_tag.len()..];
                let clean = StreamingThinkingState::strip_special_channel_tokens(after).trim().to_string();
                let clean_thinking = StreamingThinkingState::strip_special_channel_tokens(think_content).trim().to_string();
                let thinking = if clean_thinking.is_empty() {
                    None
                } else {
                    Some(clean_thinking)
                };
                return (thinking, clean);
            }
        }

        let clean = StreamingThinkingState::strip_special_channel_tokens(raw).trim().to_string();
        (None, clean)
    }
}

impl Drop for BackendManager {
    fn drop(&mut self) {
        self.stop_all_sync();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::mcp::{McpToolDefinition, McpToolWithServer};
    use crate::core::model::ChatMessage;

    #[test]
    fn streams_reasoning_when_the_template_opened_think_in_the_prompt() {
        let mut state = StreamingThinkingState::new(true);

        state.push_delta(None, Some("Vou analisar a pergunta."));
        let (content, thinking) = state.current(true);
        assert!(content.is_empty());
        assert_eq!(thinking.as_deref(), Some("Vou analisar a pergunta."));

        state.push_delta(None, Some("\n</think>\nResposta final."));
        let (content, thinking) = state.finalize(true);
        assert_eq!(content, "Resposta final.");
        assert_eq!(thinking.as_deref(), Some("Vou analisar a pergunta."));
    }

    #[test]
    fn explicit_reasoning_deltas_switch_to_final_content() {
        let mut state = StreamingThinkingState::new(true);

        state.push_delta(Some("Raciocínio do servidor."), None);
        state.push_delta(None, Some("Resposta final."));

        let (content, thinking) = state.finalize(true);
        assert_eq!(content, "Resposta final.");
        assert_eq!(thinking.as_deref(), Some("Raciocínio do servidor."));
    }

    #[test]
    fn test_build_antigravity_prompt_with_mcp_tools() {
        let dummy_tool = McpToolWithServer {
            server_id: "s1".to_string(),
            server_name: "Zero Mind".to_string(),
            permission_mode: "ask".to_string(),
            enabled: true,
            tool: McpToolDefinition {
                name: "get_current_datetime".to_string(),
                description: Some("Retorna a data e hora atual".to_string()),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {}
                }),
                label: None,
                field_labels: std::collections::HashMap::new(),
            },
        };

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "What time is it?".to_string(),
            images: None,
            tool_calls: None,
            tool_call_id: None,
        }];

        let prompt = BackendManager::build_antigravity_prompt(
            "You are Atena.",
            &messages,
            Some(&[dummy_tool]),
        );

        assert!(prompt.contains("[ATENA STUDIO OPERATION DIRECTIVE]"));
        assert!(prompt.contains("EXCLUSIVAMENTE as ferramentas do Atena Studio") || prompt.contains("EXCLUSIVELY use the Atena Studio tools"));
        assert!(prompt.contains("<tool_call>"));
        assert!(prompt.contains("get_current_datetime"));
        assert!(prompt.contains("Zero Mind"));
        assert!(prompt.contains("What time is it?"));
    }

    #[test]
    fn test_build_antigravity_prompt_without_tools() {
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
            images: None,
            tool_calls: None,
            tool_call_id: None,
        }];

        let prompt = BackendManager::build_antigravity_prompt(
            "You are Atena.",
            &messages,
            None,
        );

        assert!(prompt.contains("[ATENA STUDIO OPERATION DIRECTIVE]"));
        assert!(prompt.contains("Respond directly in natural language text without calling tools."));
        assert!(!prompt.contains("<tool_call>"));
        assert!(prompt.contains("Hello"));
    }

    #[test]
    fn test_build_antigravity_prompt_with_images() {
        let dummy_base64 = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "What is in this image?".to_string(),
            images: Some(vec![dummy_base64.to_string()]),
            tool_calls: None,
            tool_call_id: None,
        }];

        let prompt = BackendManager::build_antigravity_prompt(
            "You are Atena.",
            &messages,
            None,
        );

        assert!(prompt.contains("[ATENA STUDIO OPERATION DIRECTIVE]"));
        assert!(prompt.contains("view_file"));
        assert!(prompt.contains("Attached file/image from user:"));
        assert!(prompt.contains("atena_attachment_"));
        assert!(prompt.contains("What is in this image?"));
    }

    #[test]
    fn handles_duplicated_think_tag_from_server() {
        let mut state = StreamingThinkingState::new(true);

        // Servidor ou modelo reenvia a tag <think> no início do stream
        state.push_delta(None, Some("<think>\n"));
        state.push_delta(None, Some("Analisando a mensagem da Maria."));
        state.push_delta(None, Some("\n</think>\n"));
        state.push_delta(None, Some("Olá Maria! Como posso ajudar?"));

        let (content, thinking) = state.finalize(true);
        assert_eq!(content, "Olá Maria! Como posso ajudar?");
        assert_eq!(thinking.as_deref(), Some("Analisando a mensagem da Maria."));
        assert!(!thinking.unwrap().contains("<think>"));
    }

    #[test]
    fn cleans_residual_tags_on_finalize() {
        let mut state = StreamingThinkingState::new(false);

        // Modelo gera tags explicitamente no fluxo
        state.push_delta(None, Some("<think>Processando pedido do Carlos</think>"));
        state.push_delta(None, Some("Tudo certo, Carlos!"));

        let (content, thinking) = state.finalize(true);
        assert_eq!(content, "Tudo certo, Carlos!");
        assert_eq!(thinking.as_deref(), Some("Processando pedido do Carlos"));
    }

    #[test]
    fn mlx_modern_reasoning_and_content_separation() {
        let mut state = StreamingThinkingState::new(true);

        // Novo mlx-lm 0.31+ envia delta.reasoning e depois delta.content
        state.push_delta(Some("Pensamento interno"), None);
        state.push_delta(Some(" sobre a pergunta."), None);
        state.push_delta(None, Some("Resposta final clara."));

        let (content, thinking) = state.finalize(true);
        assert_eq!(content, "Resposta final clara.");
        assert_eq!(thinking.as_deref(), Some("Pensamento interno sobre a pergunta."));
    }

    #[test]
    fn handles_closing_tag_only_when_prompt_injected_open_tag() {
        let mut state = StreamingThinkingState::new(false);

        // O modelo começou gerando direto o raciocínio sem <think>, mas fecha com </think>
        state.push_delta(None, Some("A Ana está saudando com 'oi'. "));
        state.push_delta(None, Some("Vou responder calorosamente. </think> "));
        state.push_delta(None, Some("Olá Ana! Como posso te ajudar hoje?"));

        let (content, thinking) = state.finalize(true);
        assert_eq!(content, "Olá Ana! Como posso te ajudar hoje?");
        assert_eq!(thinking.as_deref(), Some("A Ana está saudando com 'oi'. Vou responder calorosamente."));
    }

    #[test]
    fn parse_thinking_handles_only_closing_tag() {
        let raw = "Pensando sobre a pergunta da Ana... </think>\n\nOlá Ana! Tudo bem?";
        let (thinking, clean) = BackendManager::parse_thinking(raw);
        assert_eq!(clean, "Olá Ana! Tudo bem?");
        assert_eq!(thinking.as_deref(), Some("Pensando sobre a pergunta da Ana..."));
    }

    #[test]
    fn streams_content_directly_when_no_tags_present() {
        let mut state = StreamingThinkingState::new(false);

        // O modelo gerou resposta direta sem tags
        state.push_delta(None, Some("Olá Carlos! Tudo bem com você? "));
        state.push_delta(None, Some("Aqui está a resposta direta."));

        let (content, thinking) = state.finalize(true);
        assert_eq!(content, "Olá Carlos! Tudo bem com você? Aqui está a resposta direta.");
        assert!(thinking.is_none());
    }

    #[test]
    fn keeps_thinking_separated_when_close_tag_is_emitted() {
        let mut state = StreamingThinkingState::new(true);

        state.push_delta(None, Some("Pensando sobre o pedido... </think> "));
        state.push_delta(None, Some("Aqui está o resultado final!"));

        let (content, thinking) = state.finalize(true);
        assert_eq!(content, "Aqui está o resultado final!");
        assert_eq!(thinking.as_deref(), Some("Pensando sobre o pedido..."));
    }

    #[test]
    fn test_extract_tool_calls_with_skill_command() {
        let text = "I will check the weather.\n<tool_call>\n{\"name\": \"run_skill_command\", \"arguments\": {\"command\": \"curl -s wttr.in/?format=3\"}}\n</tool_call>\nDone.";
        let (clean, calls) = BackendManager::extract_tool_calls(text, None, None);
        assert_eq!(clean, "I will check the weather.\n\nDone.");
        let tool_calls = calls.expect("should have extracted tool calls");
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].name, "run_command");
        assert_eq!(tool_calls[0].server_id.as_deref(), Some("skills"));
        assert_eq!(tool_calls[0].arguments.get("command").and_then(|v| v.as_str()), Some("curl -s wttr.in/?format=3"));
    }

    #[test]
    fn test_extract_tool_calls_with_execute_command_tag() {
        let text = "Checking weather!\n<execute_command>\ncommand: curl -s \"wttr.in/Araripina,PE/?format=3\"\n</execute_command>";
        let (clean, calls) = BackendManager::extract_tool_calls(text, None, None);
        assert_eq!(clean, "Checking weather!");
        let tool_calls = calls.expect("should have extracted execute_command as tool call");
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].name, "run_command");
        assert_eq!(tool_calls[0].server_id.as_deref(), Some("skills"));
        assert_eq!(tool_calls[0].arguments.get("command").and_then(|v| v.as_str()), Some("curl -s \"wttr.in/Araripina,PE/?format=3\""));
    }

    #[test]
    fn test_extract_tool_calls_with_multiple_execute_commands() {
        let text = "Vou verificar o clima atual para você!\n\n<execute_command>\ncommand: curl -s \"wttr.in/Araripina,PE/?format=3\"\n</execute_command>\n\n<execute_command>\ncommand: curl -s \"wttr.in/Araripina,PE/?0&format=j1\"\n</execute_command>";
        let (clean, calls) = BackendManager::extract_tool_calls(text, None, None);
        assert_eq!(clean, "Vou verificar o clima atual para você!");
        let tool_calls = calls.expect("should have extracted 2 execute_command tags");
        assert_eq!(tool_calls.len(), 2);
        assert_eq!(tool_calls[0].arguments.get("command").and_then(|v| v.as_str()), Some("curl -s \"wttr.in/Araripina,PE/?format=3\""));
        assert_eq!(tool_calls[1].arguments.get("command").and_then(|v| v.as_str()), Some("curl -s \"wttr.in/Araripina,PE/?0&format=j1\""));
    }

    #[test]
    fn test_extract_tool_calls_with_create_procedural_skill() {
        let text = "Creating skill!\n<tool_call>\n{\n  \"name\": \"create_procedural_skill\",\n  \"arguments\": {\n    \"name\": \"Disk Storage Summary\",\n    \"description\": \"Lists disk space\",\n    \"triggers\": [\"disk space\"],\n    \"steps\": [{\"order\": 1, \"instruction\": \"Run df -h\", \"command\": \"df -h\"}],\n    \"scripts\": []\n  }\n}\n</tool_call>";
        let (clean, calls) = BackendManager::extract_tool_calls(text, None, None);
        assert_eq!(clean.trim(), "Creating skill!");
        let tool_calls = calls.expect("should have extracted create_procedural_skill");
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].name, "create_procedural_skill");
        assert_eq!(tool_calls[0].server_id.as_deref(), Some("skills"));
        assert_eq!(tool_calls[0].arguments.get("name").and_then(|v| v.as_str()), Some("Disk Storage Summary"));
    }

    #[test]
    fn test_extract_tool_calls_with_update_procedural_skill() {
        let text = "Updating skill!\n<tool_call>\n{\n  \"name\": \"update_procedural_skill\",\n  \"arguments\": {\n    \"id\": \"skill-disk-storage-summary\",\n    \"name\": \"Disk Storage Summary\",\n    \"description\": \"Updated disk space check\",\n    \"refinement_note\": \"Added timeout\"\n  }\n}\n</tool_call>";
        let (clean, calls) = BackendManager::extract_tool_calls(text, None, None);
        assert_eq!(clean.trim(), "Updating skill!");
        let tool_calls = calls.expect("should have extracted update_procedural_skill");
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].name, "update_procedural_skill");
        assert_eq!(tool_calls[0].server_id.as_deref(), Some("skills"));
        assert_eq!(tool_calls[0].arguments.get("refinement_note").and_then(|v| v.as_str()), Some("Added timeout"));
    }

    #[test]
    fn test_extract_tool_calls_with_flat_skill_payload() {
        let text = "Vou criar essa skill agora!\n<tool_call>\n{\n  \"name\": \"Disk Storage Summary\",\n  \"description\": \"Lists total and used space\",\n  \"triggers\": [\"disk space\"],\n  \"steps\": [{\"order\": 1, \"instruction\": \"Run df\", \"command\": \"df -h\"}]\n}\n</tool_call>";
        let (clean, calls) = BackendManager::extract_tool_calls(text, None, None);
        assert_eq!(clean.trim(), "Vou criar essa skill agora!");
        let tool_calls = calls.expect("should have normalized flat skill tool call");
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].name, "create_procedural_skill");
        assert_eq!(tool_calls[0].server_id.as_deref(), Some("skills"));
        assert_eq!(tool_calls[0].arguments.get("name").and_then(|v| v.as_str()), Some("Disk Storage Summary"));
        assert_eq!(tool_calls[0].arguments.get("description").and_then(|v| v.as_str()), Some("Lists total and used space"));
    }

    #[test]
    fn test_extract_tool_calls_streaming_unclosed_tag() {
        let text = "I am preparing the command:\n<tool_call>\n{\n  \"name\": \"run_skill_command\",\n  \"arguments\": {\"command\": \"df -h\"";
        let (clean, calls) = BackendManager::extract_tool_calls(text, None, None);
        assert_eq!(clean.trim(), "I am preparing the command:");
        let tool_calls = calls.expect("should have extracted streaming tool call placeholder");
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].name, "run_command");
        assert_eq!(tool_calls[0].status.as_deref(), Some("streaming"));
        assert_eq!(tool_calls[0].server_id.as_deref(), Some("skills"));
    }

    #[test]
    fn test_extract_tool_calls_streaming_unclosed_execute_command() {
        let text = "Executing command now:\n<execute_command>\ncommand: df -h";
        let (clean, calls) = BackendManager::extract_tool_calls(text, None, None);
        assert_eq!(clean.trim(), "Executing command now:");
        let tool_calls = calls.expect("should have extracted streaming execute_command placeholder");
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].name, "run_command");
        assert_eq!(tool_calls[0].status.as_deref(), Some("streaming"));
        assert_eq!(tool_calls[0].server_id.as_deref(), Some("skills"));
    }

    #[test]
    fn test_extract_tool_calls_with_gemini_call_syntax() {
        let text = "I will check the weather.\ncall:run_skill_command{\"command\": \"curl -s \\\"wttr.in/?format=3\\\"\"}\nDone.";
        let (clean, calls) = BackendManager::extract_tool_calls(text, None, None);
        assert_eq!(clean.trim(), "I will check the weather.\n\nDone.");
        let tool_calls = calls.expect("should have extracted tool call from call:run_skill_command");
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].name, "run_command");
        assert_eq!(tool_calls[0].server_id.as_deref(), Some("skills"));
        assert_eq!(tool_calls[0].arguments.get("command").and_then(|v| v.as_str()), Some("curl -s \"wttr.in/?format=3\""));
    }

    #[test]
    fn test_extract_tool_calls_with_gemini_curly_quotes() {
        let text = "call:run_skill_command{“command”: “curl -s \"wttr.in/?format=3\"”}";
        let (clean, calls) = BackendManager::extract_tool_calls(text, None, None);
        assert_eq!(clean.trim(), "");
        let tool_calls = calls.expect("should have extracted tool call with curly quotes");
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].name, "run_command");
        assert_eq!(tool_calls[0].server_id.as_deref(), Some("skills"));
        assert_eq!(tool_calls[0].arguments.get("command").and_then(|v| v.as_str()), Some("curl -s \"wttr.in/?format=3\""));
    }

    #[test]
    fn test_extract_tool_calls_with_gemini_action_call_syntax() {
        let text = "Action: call:run_skill_command{\"command\": \"curl -s wttr.in/?format=3\"}";
        let (clean, calls) = BackendManager::extract_tool_calls(text, None, None);
        assert_eq!(clean.trim(), "");
        let tool_calls = calls.expect("should have extracted tool call with Action: prefix");
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].name, "run_command");
        assert_eq!(tool_calls[0].arguments.get("command").and_then(|v| v.as_str()), Some("curl -s wttr.in/?format=3"));
    }

    #[test]
    fn test_extract_tool_calls_streaming_unclosed_call_syntax() {
        let text = "Checking status:\ncall:run_skill_command{\"command\": \"curl";
        let (clean, calls) = BackendManager::extract_tool_calls(text, None, None);
        assert_eq!(clean.trim(), "Checking status:");
        let tool_calls = calls.expect("should have extracted streaming placeholder for unclosed call:");
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].name, "run_command");
        assert_eq!(tool_calls[0].status.as_deref(), Some("streaming"));
        assert_eq!(tool_calls[0].server_id.as_deref(), Some("skills"));
    }
}


