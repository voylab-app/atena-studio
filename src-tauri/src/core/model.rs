use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub images: Option<Vec<String>>,
    #[serde(default)]
    pub tool_calls: Option<Vec<crate::core::mcp::McpToolCall>>,
    #[serde(default)]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelFormat {
    Mlx,
    Gguf,
    Ollama,
    Agy,
    Cloud,
}

impl ModelFormat {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Mlx => "MLX",
            Self::Gguf => "GGUF",
            Self::Ollama => "OLLAMA",
            Self::Agy => "AGY",
            Self::Cloud => "CLOUD",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackendType {
    MlxLm,
    Ollama,
    LocalNative,
    Antigravity,
    CloudOpenAi,
}

impl BackendType {
    pub fn label(&self) -> &'static str {
        match self {
            Self::MlxLm => "MLX-LM",
            Self::Ollama => "Ollama",
            Self::LocalNative => "Atena Local",
            Self::Antigravity => "Antigravity (agy)",
            Self::CloudOpenAi => "Cloud API",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelStatus {
    NotDownloaded,
    Downloading { progress: u8, speed_mbps: u32 },
    Downloaded,
    Loading { progress: u8 },
    Loaded { vram_usage_mb: u64, ram_usage_mb: u64 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub author: String,
    pub architecture: String,
    pub quantization: String,
    pub size_gb: f32,
    pub context_length: usize,
    pub description: String,
    pub status: ModelStatus,
    pub downloads_count: u64,
    pub is_featured: bool,
    pub format: ModelFormat,
    pub backend: BackendType,
    pub local_path: Option<String>,
    #[serde(default)]
    pub supports_tools: bool,
    #[serde(default)]
    pub supports_vision: bool,
    #[serde(default)]
    pub supports_thinking: bool,
    #[serde(default)]
    pub author_avatar_url: Option<String>,
}


impl ModelInfo {
    pub fn mock_catalog() -> Vec<Self> {
        vec![
            Self {
                id: "lmstudio-community/DeepSeek-R1-0528-Qwen3-8B-MLX-4bit".into(),
                name: "DeepSeek R1 Qwen3 8B MLX".into(),
                author: "DeepSeek / lmstudio".into(),
                architecture: "qwen2".into(),
                quantization: "4-bit MLX".into(),
                size_gb: 4.88,
                context_length: 65536,
                description: "DeepSeek R1 distilled on Qwen3 8B, optimized for Apple Silicon via MLX-LM.".into(),
                status: ModelStatus::Downloaded,
                downloads_count: 980000,
                is_featured: true,
                format: ModelFormat::Mlx,
                backend: BackendType::MlxLm,
                local_path: None,
                supports_tools: false,
                supports_vision: false,
                supports_thinking: true,
                author_avatar_url: None,
            },
            Self {
                id: "lmstudio-community/Qwen3.5-9B-MLX-4bit".into(),
                name: "Qwen 3.5 9B MLX".into(),
                author: "Qwen / lmstudio".into(),
                architecture: "qwen2".into(),
                quantization: "4-bit MLX".into(),
                size_gb: 5.43,
                context_length: 32768,
                description: "State-of-the-art coding and reasoning model accelerated with MLX on Metal GPU.".into(),
                status: ModelStatus::Downloaded,
                downloads_count: 512300,
                is_featured: true,
                format: ModelFormat::Mlx,
                backend: BackendType::MlxLm,
                local_path: None,
                supports_tools: true,
                supports_vision: false,
                supports_thinking: false,
                author_avatar_url: None,
            },
            Self {
                id: "lmstudio-community/gemma-4-12B-it-MLX-4bit".into(),
                name: "Gemma 4 12B IT MLX".into(),
                author: "Google / lmstudio".into(),
                architecture: "gemma4_unified".into(),
                quantization: "4-bit MLX".into(),
                size_gb: 7.20,
                context_length: 16384,
                description: "Ultra-accurate instruction tuned model built for MLX unified memory with vision and text.".into(),
                status: ModelStatus::Downloaded,
                downloads_count: 672000,
                is_featured: false,
                format: ModelFormat::Mlx,
                backend: BackendType::MlxLm,
                local_path: None,
                supports_tools: true,
                supports_vision: true,
                supports_thinking: true,
                author_avatar_url: None,
            },
            Self {
                id: "lmstudio-community/NVIDIA-Nemotron-3-Nano-4B-GGUF".into(),
                name: "Nemotron 3 Nano 4B GGUF".into(),
                author: "NVIDIA".into(),
                architecture: "nemotron".into(),
                quantization: "Q4_K_M".into(),
                size_gb: 2.45,
                context_length: 8192,
                description: "Fast Nemotron quantized model running via Ollama / GGUF engine.".into(),
                status: ModelStatus::Downloaded,
                downloads_count: 320000,
                is_featured: false,
                format: ModelFormat::Gguf,
                backend: BackendType::Ollama,
                local_path: None,
                supports_tools: true,
                supports_vision: false,
                supports_thinking: false,
                author_avatar_url: None,
            },
        ]
    }
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GenerationMetrics {
    pub prompt_tokens: usize,
    pub prefill_tokens: usize,
    pub cached_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
    pub cache_efficiency_pct: f32,
    pub prefill_speed_tps: Option<f32>,
    pub generation_speed_tps: Option<f32>,
    pub time_to_first_token_ms: Option<u64>,
    pub total_duration_ms: u64,
    pub kv_cache_quant: Option<String>,
    pub vram_saving_pct: Option<f32>,
    pub finish_reason: Option<String>,
}

impl Default for GenerationMetrics {
    fn default() -> Self {
        Self {
            prompt_tokens: 0,
            prefill_tokens: 0,
            cached_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
            cache_efficiency_pct: 0.0,
            prefill_speed_tps: None,
            generation_speed_tps: None,
            time_to_first_token_ms: None,
            total_duration_ms: 0,
            kv_cache_quant: Some("f16".into()),
            vram_saving_pct: Some(0.0),
            finish_reason: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InferenceParams {
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: u32,
    pub max_tokens: usize,
    pub repeat_penalty: f32,
    pub gpu_layers: u32,
    pub context_length: usize,
    pub system_prompt: String,
    pub enable_thinking: Option<bool>,
    pub thinking_budget: Option<u32>,
    pub mcp_tools: Option<Vec<crate::core::mcp::McpToolWithServer>>,
    pub kv_cache_quant: Option<String>,
    pub enable_prompt_cache: Option<bool>,
    pub flash_attention: Option<bool>,
}

impl Default for InferenceParams {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            top_p: 0.9,
            top_k: 40,
            max_tokens: 4096,
            repeat_penalty: 1.1,
            gpu_layers: 33,
            context_length: 32768,
            system_prompt: String::new(),
            enable_thinking: Some(true),
            thinking_budget: Some(2048),
            mcp_tools: None,
            kv_cache_quant: Some("f16".into()),
            enable_prompt_cache: Some(true),
            flash_attention: Some(true),
        }
    }
}

