use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppThemeMode {
    DarkCosmic,
    DeepOnyx,
    NordicNight,
    #[serde(other)]
    Dark,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuardrailMode {
    Off,
    Relaxed,
    Balanced,
    Strict,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomCloudProvider {
    pub id: String,
    pub name: String,
    pub base_url: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub selected_model: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_models_directory")]
    pub models_directory: String,
    #[serde(default = "default_models_directories")]
    pub models_directories: Vec<String>,
    #[serde(default = "default_true")]
    pub auto_load_last_model: bool,
    #[serde(default = "default_true", alias = "gpu_offload")]
    pub enable_gpu_offload: bool,
    #[serde(default = "default_thread_count")]
    pub thread_count: usize,
    #[serde(default = "default_theme_mode")]
    pub theme_mode: AppThemeMode,
    #[serde(default = "default_true")]
    pub check_for_updates: bool,
    #[serde(default = "default_mlx_port", alias = "mlx_port")]
    pub mlx_server_port: u16,
    #[serde(default = "default_mlx_host", alias = "mlx_host")]
    pub mlx_server_host: String,
    #[serde(default = "default_ollama_host")]
    pub ollama_host: String,
    #[serde(default = "default_ollama_port")]
    pub ollama_port: u16,
    #[serde(default = "default_timezone")]
    pub timezone: String,
    #[serde(default = "default_false")]
    pub inject_current_date: bool,
    #[serde(default = "default_false")]
    pub inject_message_time: bool,
    #[serde(default = "default_whisper_model")]
    pub whisper_model: String,
    #[serde(default = "default_whisper_language")]
    pub whisper_language: String,
    #[serde(default = "default_false")]
    pub is_setup_completed: bool,
    #[serde(default = "default_guardrail_mode")]
    pub guardrail_mode: GuardrailMode,
    #[serde(default = "default_guardrail_custom_limit_gb")]
    pub guardrail_custom_limit_gb: f32,
    #[serde(default = "default_true")]
    pub enable_cognitive_memory: bool,
    #[serde(default = "default_true")]
    pub enable_facts_memory: bool,
    #[serde(default = "default_true")]
    pub enable_skills_memory: bool,
    #[serde(default = "default_true")]
    pub enable_episodic_memory: bool,
    #[serde(default = "default_true")]
    pub run_in_background: bool,
    #[serde(default = "default_true")]
    pub close_to_tray: bool,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub gateways: GatewaysConfig,
    #[serde(default)]
    pub cloud_providers: CloudProvidersConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct GatewaysConfig {
    #[serde(default)]
    pub telegram: TelegramConfig,
    #[serde(default)]
    pub discord: DiscordConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelegramConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub bot_token: String,
    #[serde(default)]
    pub allowed_user_ids: Vec<i64>,
    #[serde(default = "default_true")]
    pub enable_memory: bool,
    #[serde(default = "default_false")]
    pub enable_tools: bool,
    #[serde(default = "default_sliding_window")]
    pub sliding_window: usize,
    #[serde(default)]
    pub custom_system_prompt: Option<String>,
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bot_token: String::new(),
            allowed_user_ids: Vec::new(),
            enable_memory: true,
            enable_tools: false,
            sliding_window: 4,
            custom_system_prompt: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscordConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub bot_token: String,
    #[serde(default)]
    pub allowed_user_ids: Vec<String>,
    #[serde(default)]
    pub allowed_channel_ids: Vec<String>,
    #[serde(default = "default_true")]
    pub enable_memory: bool,
    #[serde(default = "default_false")]
    pub enable_tools: bool,
    #[serde(default = "default_sliding_window")]
    pub sliding_window: usize,
    #[serde(default)]
    pub custom_system_prompt: Option<String>,
}

impl Default for DiscordConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            bot_token: String::new(),
            allowed_user_ids: Vec::new(),
            allowed_channel_ids: Vec::new(),
            enable_memory: true,
            enable_tools: false,
            sliding_window: 4,
            custom_system_prompt: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CloudProvidersConfig {
    #[serde(default)]
    pub openai_enabled: bool,
    #[serde(default)]
    pub openai_api_key: String,
    #[serde(default)]
    pub openai_base_url: Option<String>,
    #[serde(default)]
    pub openai_selected_model: Option<String>,

    #[serde(default)]
    pub gemini_enabled: bool,
    #[serde(default)]
    pub gemini_api_key: String,
    #[serde(default)]
    pub gemini_selected_model: Option<String>,

    #[serde(default)]
    pub openrouter_enabled: bool,
    #[serde(default)]
    pub openrouter_api_key: String,
    #[serde(default)]
    pub openrouter_selected_model: Option<String>,

    #[serde(default)]
    pub groq_enabled: bool,
    #[serde(default)]
    pub groq_api_key: String,
    #[serde(default)]
    pub groq_selected_model: Option<String>,

    #[serde(default = "default_true")]
    pub antigravity_enabled: bool,
    #[serde(default = "default_antigravity_effort")]
    pub antigravity_default_effort: String,

    #[serde(default)]
    pub custom_providers: Vec<CustomCloudProvider>,

    #[serde(default)]
    pub custom_enabled: bool,
    #[serde(default = "default_custom_provider_name")]
    pub custom_name: String,
    #[serde(default)]
    pub custom_base_url: String,
    #[serde(default)]
    pub custom_api_key: String,
    #[serde(default)]
    pub custom_selected_model: Option<String>,

    #[serde(default)]
    pub disabled_models: Vec<String>,
    #[serde(default)]
    pub cached_models: std::collections::HashMap<String, Vec<crate::core::model::ModelInfo>>,
}

fn default_true() -> bool {
    true
}

fn default_sliding_window() -> usize {
    4
}

fn default_language() -> String {
    "pt-BR".into()
}

fn default_theme_mode() -> AppThemeMode {
    AppThemeMode::DarkCosmic
}

fn default_thread_count() -> usize {
    8
}

fn default_mlx_port() -> u16 {
    8080
}

fn default_mlx_host() -> String {
    "127.0.0.1".into()
}

fn default_ollama_host() -> String {
    "127.0.0.1".into()
}

fn default_ollama_port() -> u16 {
    11434
}

fn default_antigravity_effort() -> String {
    "medium".into()
}

fn default_custom_provider_name() -> String {
    "Custom OpenAI".into()
}

impl Default for CloudProvidersConfig {
    fn default() -> Self {
        Self {
            openai_enabled: false,
            openai_api_key: String::new(),
            openai_base_url: None,
            openai_selected_model: None,
            gemini_enabled: false,
            gemini_api_key: String::new(),
            gemini_selected_model: None,
            openrouter_enabled: false,
            openrouter_api_key: String::new(),
            openrouter_selected_model: None,
            groq_enabled: false,
            groq_api_key: String::new(),
            groq_selected_model: None,
            antigravity_enabled: true,
            antigravity_default_effort: default_antigravity_effort(),
            custom_providers: Vec::new(),
            custom_enabled: false,
            custom_name: default_custom_provider_name(),
            custom_base_url: String::new(),
            custom_api_key: String::new(),
            custom_selected_model: None,
            disabled_models: Vec::new(),
            cached_models: std::collections::HashMap::new(),
        }
    }
}

fn default_guardrail_mode() -> GuardrailMode {
    GuardrailMode::Relaxed
}

fn default_guardrail_custom_limit_gb() -> f32 {
    4.0
}

fn default_whisper_model() -> String {
    "mlx-community/whisper-small-mlx".into()
}

fn default_whisper_language() -> String {
    "pt".into()
}

fn default_timezone() -> String {
    "America/Sao_Paulo".into()
}

fn default_false() -> bool {
    false
}

pub fn default_models_directory() -> String {
    let path = if cfg!(target_os = "windows") {
        let userprofile = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".to_string());
        std::path::PathBuf::from(&userprofile).join(".atena").join("models")
    } else {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        std::path::PathBuf::from(&home).join(".atena").join("models")
    };

    if !path.exists() {
        let _ = std::fs::create_dir_all(&path);
    }

    "~/.atena/models".to_string()
}

pub fn resolve_path(p: &str) -> std::path::PathBuf {
    let trimmed = p.trim();
    if trimmed.starts_with('~') {
        let home = if cfg!(target_os = "windows") {
            std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".to_string())
        } else {
            std::env::var("HOME").unwrap_or_else(|_| ".".to_string())
        };
        let after_tilde = trimmed.strip_prefix('~').unwrap_or("");
        let after_tilde = after_tilde
            .strip_prefix('/')
            .or_else(|| after_tilde.strip_prefix('\\'))
            .unwrap_or(after_tilde);
        std::path::PathBuf::from(home).join(after_tilde)
    } else {
        std::path::PathBuf::from(trimmed)
    }
}

pub fn contract_path(p: &str) -> String {
    let trimmed = p.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.starts_with('~') {
        return trimmed.to_string();
    }

    let home = if cfg!(target_os = "windows") {
        std::env::var("USERPROFILE").unwrap_or_default()
    } else {
        std::env::var("HOME").unwrap_or_default()
    };

    if !home.is_empty() {
        let check_matches = |base: &str| -> Option<String> {
            if base.is_empty() {
                return None;
            }
            let base_trimmed = base.trim_end_matches(['/', '\\']);
            if trimmed == base_trimmed {
                return Some("~".to_string());
            }
            let prefix_slash = format!("{}/", base_trimmed);
            if trimmed.starts_with(&prefix_slash) {
                return Some(format!("~/{}", &trimmed[prefix_slash.len()..]));
            }
            let prefix_backslash = format!("{}\\", base_trimmed);
            if trimmed.starts_with(&prefix_backslash) {
                return Some(format!("~/{}", &trimmed[prefix_backslash.len()..].replace('\\', "/")));
            }
            None
        };

        if let Some(contracted) = check_matches(&home) {
            return contracted;
        }

        if let Ok(canon_home) = std::fs::canonicalize(&home) {
            let canon_str = canon_home.to_string_lossy();
            if let Some(contracted) = check_matches(&canon_str) {
                return contracted;
            }
        }
    }

    trimmed.to_string()
}

pub fn default_models_directories() -> Vec<String> {
    vec![default_models_directory()]
}

impl Default for AppConfig {
    fn default() -> Self {
        let default_dir = default_models_directory();
        Self {
            models_directory: default_dir.clone(),
            models_directories: vec![default_dir],
            auto_load_last_model: true,
            enable_gpu_offload: true,
            thread_count: 8,
            theme_mode: AppThemeMode::DarkCosmic,
            check_for_updates: true,
            mlx_server_port: 8080,
            mlx_server_host: "127.0.0.1".into(),
            ollama_host: "127.0.0.1".into(),
            ollama_port: 11434,
            timezone: default_timezone(),
            inject_current_date: false,
            inject_message_time: false,
            whisper_model: default_whisper_model(),
            whisper_language: default_whisper_language(),
            is_setup_completed: false,
            guardrail_mode: default_guardrail_mode(),
            guardrail_custom_limit_gb: default_guardrail_custom_limit_gb(),
            enable_cognitive_memory: true,
            enable_facts_memory: true,
            enable_skills_memory: true,
            enable_episodic_memory: true,
            run_in_background: true,
            close_to_tray: true,
            language: default_language(),
            gateways: GatewaysConfig::default(),
            cloud_providers: CloudProvidersConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn get_config_filepath() -> std::path::PathBuf {
        let base_dir = if cfg!(target_os = "windows") {
            let userprofile = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".to_string());
            std::path::PathBuf::from(&userprofile).join(".atena")
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            std::path::PathBuf::from(&home).join(".atena")
        };
        if !base_dir.exists() {
            let _ = std::fs::create_dir_all(&base_dir);
        }
        base_dir.join("config.json")
    }

    pub fn get_models_cache_filepath() -> std::path::PathBuf {
        let base_dir = if cfg!(target_os = "windows") {
            let userprofile = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".to_string());
            std::path::PathBuf::from(&userprofile).join(".atena")
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            std::path::PathBuf::from(&home).join(".atena")
        };
        if !base_dir.exists() {
            let _ = std::fs::create_dir_all(&base_dir);
        }
        base_dir.join("models_cache.json")
    }

    pub fn load() -> Self {
        let path = Self::get_config_filepath();
        let mut cfg: Self = if let Ok(data) = std::fs::read_to_string(&path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        };
        cfg.models_directory = contract_path(&cfg.models_directory);
        cfg.models_directories = cfg
            .models_directories
            .into_iter()
            .map(|d| contract_path(&d))
            .collect();
        cfg
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::get_config_filepath();
        let mut to_save = self.clone();
        to_save.models_directory = contract_path(&to_save.models_directory);
        to_save.models_directories = to_save
            .models_directories
            .into_iter()
            .map(|d| contract_path(&d))
            .collect();
        let json = serde_json::to_string_pretty(&to_save)
            .map_err(|e| format!("Erro ao serializar config.json: {}", e))?;
        std::fs::write(&path, json)
            .map_err(|e| format!("Erro ao gravar config.json: {}", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_and_contract_path() {
        let default_dir = default_models_directory();
        assert_eq!(default_dir, "~/.atena/models");

        let resolved = resolve_path(&default_dir);
        assert!(resolved.to_string_lossy().contains(".atena"));
        assert!(!resolved.to_string_lossy().starts_with('~'));

        let contracted = contract_path(&resolved.to_string_lossy());
        assert_eq!(contracted, "~/.atena/models");

        // External paths outside home remain untouched
        let external_path = "/Volumes/ExternalDrive/models";
        assert_eq!(contract_path(external_path), external_path);
    }
}
