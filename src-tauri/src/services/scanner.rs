use std::path::{Path, PathBuf};
use serde_json::Value;
use tokio::fs;
use base64::Engine as _;

use crate::core::model::{BackendType, ModelFormat, ModelInfo, ModelStatus};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedModelDirectory {
    pub name: String,
    pub path: String,
    pub exists: bool,
    pub source: String, // "atena", "lmstudio", "ollama", "huggingface"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskModelCacheEntry {
    pub path: String,
    pub mtime: u64,
    pub size_bytes: u64,
    pub model: ModelInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelsDiskCache {
    pub updated_at: u64,
    pub models: Vec<ModelInfo>,
    #[serde(default)]
    pub file_entries: std::collections::HashMap<String, DiskModelCacheEntry>,
}

pub struct ModelScanner;

impl ModelScanner {
    pub fn load_models_cache_data() -> ModelsDiskCache {
        let path = crate::core::config::AppConfig::get_models_cache_filepath();
        if let Ok(data) = std::fs::read_to_string(&path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            ModelsDiskCache::default()
        }
    }

    pub fn is_model_disabled(model_id: &str, disabled_models: &[String]) -> bool {
        let clean_id = model_id.trim();
        let raw_id = clean_id.split_once('/').map(|(_, r)| r).unwrap_or(clean_id);

        disabled_models.iter().any(|d| {
            let clean_d = d.trim();
            let raw_d = clean_d.split_once('/').map(|(_, r)| r).unwrap_or(clean_d);

            if clean_id == clean_d || raw_id == raw_d {
                return true;
            }

            // AGY or Gemini variants matching (e.g. agy/gemini-3.8-flash matches agy/gemini-3.8-flash-high)
            if (clean_id.starts_with("agy/") || clean_id.starts_with("gemini/"))
                && (clean_d.starts_with("agy/") || clean_d.starts_with("gemini/"))
            {
                let base_id = clean_id
                    .trim_end_matches("-high")
                    .trim_end_matches("-medium")
                    .trim_end_matches("-low");
                let base_d = clean_d
                    .trim_end_matches("-high")
                    .trim_end_matches("-medium")
                    .trim_end_matches("-low");
                if base_id == base_d {
                    return true;
                }
            }

            false
        })
    }

    pub fn load_cached_models() -> Vec<ModelInfo> {
        let mut models = Self::load_models_cache_data().models;
        let config = crate::core::config::AppConfig::load();
        if !config.cloud_providers.disabled_models.is_empty() {
            models.retain(|m| !Self::is_model_disabled(&m.id, &config.cloud_providers.disabled_models));
        }
        models
    }

    pub fn save_models_cache_data(cache: &ModelsDiskCache) {
        let path = crate::core::config::AppConfig::get_models_cache_filepath();
        if let Ok(json) = serde_json::to_string(cache) {
            let _ = std::fs::write(&path, json);
        }
    }

    /// Updates or inserts a specific model entry into the disk cache
    pub fn update_cached_model(fresh: &ModelInfo) {
        let mut disk_cache = Self::load_models_cache_data();
        if let Some(idx) = disk_cache.models.iter().position(|m| m.id == fresh.id || (m.local_path.is_some() && m.local_path == fresh.local_path)) {
            disk_cache.models[idx] = fresh.clone();
        } else {
            disk_cache.models.push(fresh.clone());
        }
        Self::save_models_cache_data(&disk_cache);
    }

    /// Re-inspects the user-selected model directly from disk, refreshing capabilities and cache
    pub async fn refresh_single_model(model: &ModelInfo) -> ModelInfo {
        // AGY and Cloud models are remote-backed — their local_path points to the CLI binary,
        // not a model file. Never re-inspect them from disk.
        match model.backend {
            BackendType::Antigravity | BackendType::CloudOpenAi => return model.clone(),
            _ => {}
        }
        match model.format {
            ModelFormat::Agy | ModelFormat::Cloud => return model.clone(),
            _ => {}
        }

        if let Some(ref path_str) = model.local_path {
            let p = Path::new(path_str);
            if p.is_dir() {
                if let Some(mut fresh) = Self::inspect_mlx_directory(p).await {
                    if fresh.author_avatar_url.is_none() && model.author_avatar_url.is_some() {
                        fresh.author_avatar_url = model.author_avatar_url.clone();
                    }
                    Self::update_cached_model(&fresh);
                    return fresh;
                }
            } else if p.is_file() {
                if let Some(mut fresh) = Self::inspect_gguf_file(p).await {
                    if fresh.author_avatar_url.is_none() && model.author_avatar_url.is_some() {
                        fresh.author_avatar_url = model.author_avatar_url.clone();
                    }
                    Self::update_cached_model(&fresh);
                    return fresh;
                }
            }
        }
        model.clone()
    }

    /// Detects standard directories used by AI tools on the host system
    pub fn detect_common_model_dirs() -> Vec<DetectedModelDirectory> {
        let mut list = Vec::new();

        // 1. Atena internal default directory
        let atena_dir = crate::core::config::default_models_directory();
        let resolved_atena = crate::core::config::resolve_path(&atena_dir);
        list.push(DetectedModelDirectory {
            name: "Atena Studio Default".to_string(),
            path: crate::core::config::contract_path(&atena_dir),
            exists: resolved_atena.exists(),
            source: "atena".to_string(),
        });

        // 2. LM Studio locations
        #[cfg(target_os = "windows")]
        {
            let userprofile = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".to_string());
            let p1 = PathBuf::from(&userprofile).join(".lmstudio").join("models");
            if p1.exists() {
                list.push(DetectedModelDirectory {
                    name: "LM Studio Models".to_string(),
                    path: crate::core::config::contract_path(&p1.to_string_lossy()),
                    exists: true,
                    source: "lmstudio".to_string(),
                });
            }
            let p2 = PathBuf::from(&userprofile).join(".cache").join("lm-studio").join("models");
            let p2_contracted = crate::core::config::contract_path(&p2.to_string_lossy());
            if p2.exists() && !list.iter().any(|d| d.path == p2_contracted || d.path == p2.to_string_lossy()) {
                list.push(DetectedModelDirectory {
                    name: "LM Studio Cache".to_string(),
                    path: p2_contracted,
                    exists: true,
                    source: "lmstudio".to_string(),
                });
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let p1 = PathBuf::from(&home).join(".lmstudio/models");
            if p1.exists() {
                list.push(DetectedModelDirectory {
                    name: "LM Studio Models".to_string(),
                    path: crate::core::config::contract_path(&p1.to_string_lossy()),
                    exists: true,
                    source: "lmstudio".to_string(),
                });
            }
            let p2 = PathBuf::from(&home).join(".cache/lm-studio/models");
            let p2_contracted = crate::core::config::contract_path(&p2.to_string_lossy());
            if p2.exists() && !list.iter().any(|d| d.path == p2_contracted || d.path == p2.to_string_lossy()) {
                list.push(DetectedModelDirectory {
                    name: "LM Studio Cache".to_string(),
                    path: p2_contracted,
                    exists: true,
                    source: "lmstudio".to_string(),
                });
            }
        }

        // 3. Ollama models directory
        #[cfg(target_os = "windows")]
        {
            let userprofile = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".to_string());
            let p_ollama = PathBuf::from(&userprofile).join(".ollama").join("models");
            if p_ollama.exists() {
                list.push(DetectedModelDirectory {
                    name: "Ollama Models".to_string(),
                    path: crate::core::config::contract_path(&p_ollama.to_string_lossy()),
                    exists: true,
                    source: "ollama".to_string(),
                });
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let p_ollama = PathBuf::from(&home).join(".ollama/models");
            if p_ollama.exists() {
                list.push(DetectedModelDirectory {
                    name: "Ollama Models".to_string(),
                    path: crate::core::config::contract_path(&p_ollama.to_string_lossy()),
                    exists: true,
                    source: "ollama".to_string(),
                });
            }
            let p_system = PathBuf::from("/usr/share/ollama/.ollama/models");
            let p_system_contracted = crate::core::config::contract_path(&p_system.to_string_lossy());
            if p_system.exists() && !list.iter().any(|d| d.path == p_system_contracted || d.path == p_system.to_string_lossy()) {
                list.push(DetectedModelDirectory {
                    name: "Ollama System Models".to_string(),
                    path: p_system_contracted,
                    exists: true,
                    source: "ollama".to_string(),
                });
            }
        }

        // 4. HuggingFace cache
        let hf_path = if cfg!(target_os = "windows") {
            let userprofile = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".to_string());
            PathBuf::from(&userprofile).join(".cache").join("huggingface").join("hub")
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(&home).join(".cache/huggingface/hub")
        };
        if hf_path.exists() {
            list.push(DetectedModelDirectory {
                name: "Hugging Face Hub Cache".to_string(),
                path: crate::core::config::contract_path(&hf_path.to_string_lossy()),
                exists: true,
                source: "huggingface".to_string(),
            });
        }

        list
    }

    /// Scans multiple root model directories, deduplicating files and
    /// querying Ollama and Antigravity services to build the catalog.
    pub async fn scan_all_directories(models_dirs: &[String], ollama_host: &str, ollama_port: u16) -> Vec<ModelInfo> {
        let disk_cache = Self::load_models_cache_data();
        let cache_lookup = disk_cache.file_entries;
        let mut new_entries = std::collections::HashMap::new();

        let mut models = Vec::new();

        for dir in models_dirs {
            let trimmed = dir.trim();
            if trimmed.is_empty() {
                continue;
            }
            let path = crate::core::config::resolve_path(trimmed);
            if path.exists() {
                Self::scan_directory_recursive(&path, &path, &mut models, &cache_lookup, &mut new_entries).await;
            }
        }

        // On non-macOS, filter out MLX models (they can't be executed)
        #[cfg(not(target_os = "macos"))]
        {
            models.retain(|m| m.format != ModelFormat::Mlx);
        }

        // Filter out unsupported/deprecated GGUF quantizations (e.g. Q4_0_4_4, Q4_0_4_8, Q4_0_8_8 removed from llama.cpp)
        models.retain(|m| {
            let q = m.quantization.to_lowercase();
            let name = m.name.to_lowercase();
            let id = m.id.to_lowercase();
            !q.contains("q4_0_4_4") && !q.contains("q4_0_4_8") && !q.contains("q4_0_8_8") &&
            !name.contains("q4_0_4_4") && !name.contains("q4_0_4_8") && !name.contains("q4_0_8_8") &&
            !id.contains("q4_0_4_4") && !id.contains("q4_0_4_8") && !id.contains("q4_0_8_8")
        });

        // Deduplicate local models by canonical path or id
        let mut unique_models: Vec<ModelInfo> = Vec::new();
        for m in models {
            let is_duplicate = unique_models.iter().any(|existing| {
                if let (Some(ref p1), Some(ref p2)) = (&existing.local_path, &m.local_path) {
                    p1 == p2
                } else {
                    existing.id == m.id
                }
            });

            if !is_duplicate {
                unique_models.push(m);
            }
        }
        let mut models = unique_models;

        let config = crate::core::config::AppConfig::load();

        // Query remote/cloud and Ollama providers concurrently
        let (ollama_models, agy_models, openai_models, openrouter_models, gemini_models, groq_models) = tokio::join!(
            Self::scan_ollama(ollama_host, ollama_port),
            async {
                if config.cloud_providers.antigravity_enabled
                    || crate::services::plugins::PluginManager::is_plugin_enabled("atena-plugin-agy")
                    || crate::services::plugins::PluginManager::is_plugin_enabled("atena-plugin-cloud")
                {
                    Self::scan_antigravity().await
                } else {
                    Vec::new()
                }
            },
            async {
                if config.cloud_providers.openai_enabled {
                    Self::scan_openai(&config.cloud_providers.openai_api_key, config.cloud_providers.openai_base_url.as_deref()).await
                } else {
                    Vec::new()
                }
            },
            async {
                if config.cloud_providers.openrouter_enabled {
                    Self::scan_openrouter(&config.cloud_providers.openrouter_api_key).await
                } else {
                    Vec::new()
                }
            },
            async {
                if config.cloud_providers.gemini_enabled {
                    Self::scan_gemini(&config.cloud_providers.gemini_api_key).await
                } else {
                    Vec::new()
                }
            },
            async {
                if config.cloud_providers.groq_enabled {
                    Self::scan_groq(&config.cloud_providers.groq_api_key).await
                } else {
                    Vec::new()
                }
            }
        );

        for om in ollama_models {
            if !models.iter().any(|m| m.id == om.id) {
                models.push(om);
            }
        }

        for am in agy_models {
            if !models.iter().any(|m| m.id == am.id) {
                models.push(am);
            }
        }

        for om in openai_models {
            if !models.iter().any(|m| m.id == om.id) {
                models.push(om);
            }
        }

        for om in openrouter_models {
            if !models.iter().any(|m| m.id == om.id) {
                models.push(om);
            }
        }

        for gm in gemini_models {
            if !models.iter().any(|m| m.id == gm.id) {
                models.push(gm);
            }
        }

        for gm in groq_models {
            if !models.iter().any(|m| m.id == gm.id) {
                models.push(gm);
            }
        }

        // Multiple Custom OpenAI Endpoint models
        for custom_p in &config.cloud_providers.custom_providers {
            if custom_p.enabled && !custom_p.base_url.trim().is_empty() {
                let custom_models = Self::scan_custom_provider(custom_p).await;
                for cm in custom_models {
                    if !models.iter().any(|m| m.id == cm.id) {
                        models.push(cm);
                    }
                }
            }
        }

        // Also query legacy Custom OpenAI Endpoint models if enabled
        if config.cloud_providers.custom_enabled && !config.cloud_providers.custom_base_url.trim().is_empty() {
            let custom_models = Self::scan_custom(
                &config.cloud_providers.custom_name,
                &config.cloud_providers.custom_base_url,
                &config.cloud_providers.custom_api_key,
                config.cloud_providers.custom_selected_model.as_deref(),
            ).await;
            for cm in custom_models {
                if !models.iter().any(|m| m.id == cm.id) {
                    models.push(cm);
                }
            }
        }

        // Incorporate user-cached discovered models
        for (prov_id, cached_list) in &config.cloud_providers.cached_models {
            let is_active = match prov_id.as_str() {
                "antigravity" => config.cloud_providers.antigravity_enabled,
                "openai" => config.cloud_providers.openai_enabled,
                "gemini" => config.cloud_providers.gemini_enabled,
                "openrouter" => config.cloud_providers.openrouter_enabled,
                "groq" => config.cloud_providers.groq_enabled,
                "ollama" => true,
                _ => config.cloud_providers.custom_providers.iter().any(|p| p.id == *prov_id && p.enabled),
            };
            if is_active {
                for cm in cached_list {
                    if !models.iter().any(|m| m.id == cm.id) {
                        models.push(cm.clone());
                    }
                }
            }
        }

        // Filter out user-disabled models
        if !config.cloud_providers.disabled_models.is_empty() {
            models.retain(|m| !Self::is_model_disabled(&m.id, &config.cloud_providers.disabled_models));
        }

        // Persist updated cache to disk
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let updated_cache = ModelsDiskCache {
            updated_at: now,
            models: models.clone(),
            file_entries: new_entries,
        };
        Self::save_models_cache_data(&updated_cache);

        models
    }

    /// Scans a root models directory and queries the Ollama service
    pub async fn scan_all(models_dir: &str, ollama_host: &str, ollama_port: u16) -> Vec<ModelInfo> {
        Self::scan_all_directories(&[models_dir.to_string()], ollama_host, ollama_port).await
    }

    /// Checks if a model or file is already downloaded and present in any model directory.
    pub async fn check_model_exists_on_disk(
        models_dirs: &[String],
        repo_id: &str,
        files: &[String],
        is_mlx_bundle: bool,
    ) -> Option<ModelInfo> {
        let repo_clean = repo_id.trim();
        if repo_clean.is_empty() {
            return None;
        }

        let extract_slug = |s: &str| -> String {
            let last_part = s.split('/').last().unwrap_or(s);
            let last_part = last_part.split("___").last().unwrap_or(last_part);
            last_part
                .to_lowercase()
                .replace(".gguf", "")
                .replace("mlx", "")
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect()
        };

        let target_slug = extract_slug(repo_clean);

        for dir_str in models_dirs {
            let base = crate::core::config::resolve_path(dir_str.trim());
            if !base.exists() {
                continue;
            }

            if is_mlx_bundle {
                let sanitized_repo = repo_clean.replace('/', "___");
                let repo_leaf = repo_clean.split('/').last().unwrap_or(repo_clean);
                let candidate_paths = vec![
                    base.join(&sanitized_repo),
                    base.join(repo_clean),
                    base.join("lmstudio-community").join(repo_leaf),
                    base.join("mlx-community").join(repo_leaf),
                ];

                for p in candidate_paths {
                    if p.join("config.json").exists() {
                        if let Some(m) = Self::inspect_mlx_directory(&p).await {
                            return Some(m);
                        }
                    }
                }

                if let Ok(mut rd) = fs::read_dir(&base).await {
                    while let Ok(Some(entry)) = rd.next_entry().await {
                        let p = entry.path();
                        if p.is_dir() {
                            if p.join("config.json").exists() {
                                let folder_slug = extract_slug(&p.file_name().unwrap_or_default().to_string_lossy());
                                if !target_slug.is_empty() && (folder_slug == target_slug || folder_slug.starts_with(&target_slug) || target_slug.starts_with(&folder_slug)) {
                                    if let Some(m) = Self::inspect_mlx_directory(&p).await {
                                        return Some(m);
                                    }
                                }
                            }

                            if let Ok(mut sub_rd) = fs::read_dir(&p).await {
                                while let Ok(Some(sub_entry)) = sub_rd.next_entry().await {
                                    let sub_p = sub_entry.path();
                                    if sub_p.is_dir() && sub_p.join("config.json").exists() {
                                        let sub_slug = extract_slug(&sub_p.file_name().unwrap_or_default().to_string_lossy());
                                        if !target_slug.is_empty() && (sub_slug == target_slug || sub_slug.starts_with(&target_slug) || target_slug.starts_with(&sub_slug)) {
                                            if let Some(m) = Self::inspect_mlx_directory(&sub_p).await {
                                                return Some(m);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                for file_name in files {
                    if !file_name.ends_with(".gguf") {
                        continue;
                    }
                    let p_direct = base.join(file_name);
                    if p_direct.exists() {
                        if let Some(m) = Self::inspect_gguf_file(&p_direct).await {
                            return Some(m);
                        }
                    }

                    if let Ok(mut rd) = fs::read_dir(&base).await {
                        while let Ok(Some(entry)) = rd.next_entry().await {
                            let p = entry.path();
                            if p.is_dir() {
                                let sub_candidate = p.join(file_name);
                                if sub_candidate.exists() {
                                    if let Some(m) = Self::inspect_gguf_file(&sub_candidate).await {
                                        return Some(m);
                                    }
                                }
                                if let Ok(mut sub_rd) = fs::read_dir(&p).await {
                                    while let Ok(Some(sub_entry)) = sub_rd.next_entry().await {
                                        let sub_p = sub_entry.path();
                                        if sub_p.is_dir() {
                                            let sub2_candidate = sub_p.join(file_name);
                                            if sub2_candidate.exists() {
                                                if let Some(m) = Self::inspect_gguf_file(&sub2_candidate).await {
                                                    return Some(m);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }


    /// Recursively traverses a folder looking for MLX packages and GGUF files with disk caching.
    async fn scan_directory_recursive(
        base_path: &Path,
        current_path: &Path,
        results: &mut Vec<ModelInfo>,
        cache_lookup: &std::collections::HashMap<String, DiskModelCacheEntry>,
        new_entries: &mut std::collections::HashMap<String, DiskModelCacheEntry>,
    ) {
        let mut read_dir = match fs::read_dir(current_path).await {
            Ok(rd) => rd,
            Err(_) => return,
        };

        // Check if the current directory itself is an MLX model directory (has config.json and safetensors)
        let config_file = current_path.join("config.json");
        if config_file.exists() && current_path != base_path {
            let current_path_str = current_path.to_string_lossy().to_string();
            let config_meta = fs::metadata(&config_file).await.ok();
            let mtime = config_meta.as_ref()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let size_bytes = config_meta.as_ref().map(|m| m.len()).unwrap_or(0);

            if let Some(cached) = cache_lookup.get(&current_path_str) {
                if cached.mtime == mtime && cached.size_bytes == size_bytes {
                    results.push(cached.model.clone());
                    new_entries.insert(current_path_str, cached.clone());
                    return;
                }
            }

            if let Some(mlx_model) = Self::inspect_mlx_directory(current_path).await {
                let entry = DiskModelCacheEntry {
                    path: current_path_str.clone(),
                    mtime,
                    size_bytes,
                    model: mlx_model.clone(),
                };
                new_entries.insert(current_path_str, entry);
                results.push(mlx_model);
                return; // Don't inspect children of a model bundle
            }
        }

        while let Ok(Some(entry)) = read_dir.next_entry().await {
            let path = entry.path();
            if let Ok(file_type) = entry.file_type().await {
                if file_type.is_dir() {
                    Box::pin(Self::scan_directory_recursive(base_path, &path, results, cache_lookup, new_entries)).await;
                } else if file_type.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext.to_string_lossy().to_lowercase() == "gguf" {
                            let path_str = path.to_string_lossy().to_string();
                            let meta = entry.metadata().await.ok();
                            let mtime = meta.as_ref()
                                .and_then(|m| m.modified().ok())
                                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                .map(|d| d.as_secs())
                                .unwrap_or(0);
                            let size_bytes = meta.as_ref().map(|m| m.len()).unwrap_or(0);

                            if let Some(cached) = cache_lookup.get(&path_str) {
                                if cached.mtime == mtime && cached.size_bytes == size_bytes {
                                    results.push(cached.model.clone());
                                    new_entries.insert(path_str, cached.clone());
                                    continue;
                                }
                            }

                            if let Some(gguf_model) = Self::inspect_gguf_file(&path).await {
                                let disk_entry = DiskModelCacheEntry {
                                    path: path_str.clone(),
                                    mtime,
                                    size_bytes,
                                    model: gguf_model.clone(),
                                };
                                new_entries.insert(path_str, disk_entry);
                                results.push(gguf_model);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Normalizes raw model IDs or filenames into human-friendly, clean model names
    pub fn format_model_name(raw: &str) -> String {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        // 1. Take last segment if it has path or ___ separator
        let stem = trimmed.split('/').last().unwrap_or(trimmed);
        let stem = stem.split("___").last().unwrap_or(stem);

        // 2. Remove common file extensions
        let stem = stem
            .strip_suffix(".gguf")
            .or_else(|| stem.strip_suffix(".safetensors"))
            .or_else(|| stem.strip_suffix(".bin"))
            .unwrap_or(stem);

        // 3. Remove quant patterns and format prefixes/suffixes
        let mut cleaned = stem.to_string();
        for pattern in &[
            "-MLX-4bit", "-MLX-8bit", "-mlx-4bit", "-mlx-8bit",
            "-MLX", "-mlx", "_MLX", "_mlx",
            "-GGUF", "-gguf", "_GGUF", "_gguf",
            "-safetensors", "_safetensors",
        ] {
            cleaned = cleaned.replace(pattern, " ");
        }

        cleaned = cleaned.replace(['-', '_'], " ");

        let parts: Vec<&str> = cleaned.split_whitespace().collect();
        let mut formatted_parts = Vec::new();

        for part in parts {
            if part.is_empty() {
                continue;
            }
            let upper = part.to_uppercase();
            if matches!(upper.as_str(), "AI" | "GGUF" | "MLX" | "QAT" | "MTP" | "VLM" | "DPO" | "RLHF" | "IT" | "BF16" | "FP16" | "F16" | "F32") {
                formatted_parts.push(upper);
            } else if (upper.ends_with('B') || upper.ends_with('M') || upper.ends_with('K')) && upper[..upper.len() - 1].parse::<f64>().is_ok() {
                formatted_parts.push(upper);
            } else if upper.ends_with("BIT") && upper[..upper.len() - 3].parse::<f64>().is_ok() {
                formatted_parts.push(upper.to_lowercase()); // "1bit", "4bit", "8bit"
            } else if part.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()) && part.len() >= 2 {
                formatted_parts.push(part.to_string());
            } else {
                let mut chars = part.chars();
                if let Some(first) = chars.next() {
                    formatted_parts.push(format!("{}{}", first.to_uppercase(), chars.as_str()));
                }
            }
        }

        let result = formatted_parts.join(" ");
        if result.is_empty() {
            trimmed.to_string()
        } else {
            result
        }
    }

    /// Searches for a locally stored logo image in the directory or assets subfolder.
    /// Returns a base64 data URI if found, ensuring instant offline loading.
    pub async fn find_local_logo(dir: &Path) -> Option<String> {
        let candidates = [
            dir.join("logo.png"),
            dir.join("logo.svg"),
            dir.join("avatar.png"),
            dir.join("avatar.svg"),
            dir.join("logo.jpg"),
            dir.join("logo.jpeg"),
            dir.join("logo.webp"),
            dir.join("assets").join("logo.png"),
            dir.join("assets").join("logo.svg"),
            dir.join("assets").join("bonsai-logo.svg"),
            dir.join("assets").join("avatar.png"),
        ];

        for c in &candidates {
            if c.is_file() {
                if let Ok(bytes) = fs::read(c).await {
                    if !bytes.is_empty() {
                        let ext = c.extension().and_then(|s| s.to_str()).unwrap_or("png").to_lowercase();
                        let mime = match ext.as_str() {
                            "svg" => "image/svg+xml",
                            "jpg" | "jpeg" => "image/jpeg",
                            "webp" => "image/webp",
                            _ => "image/png",
                        };
                        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                        return Some(format!("data:{};base64,{}", mime, b64));
                    }
                }
            }
        }

        // Inspect assets/ directory for any file containing "logo", "avatar", or "icon"
        let assets_dir = dir.join("assets");
        if assets_dir.is_dir() {
            if let Ok(mut entries) = fs::read_dir(&assets_dir).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let path = entry.path();
                    if path.is_file() {
                        let name_lower = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
                        if name_lower.contains("logo") || name_lower.contains("avatar") || name_lower.contains("icon") {
                            if let Ok(bytes) = fs::read(&path).await {
                                if !bytes.is_empty() {
                                    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("png").to_lowercase();
                                    let mime = match ext.as_str() {
                                        "svg" => "image/svg+xml",
                                        "jpg" | "jpeg" => "image/jpeg",
                                        "webp" => "image/webp",
                                        _ => "image/png",
                                    };
                                    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                                    return Some(format!("data:{};base64,{}", mime, b64));
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Resolves logo from local folder or fetches from HF and downloads it to folder
    pub async fn resolve_and_save_logo(dir: &Path, author: &str, model_id: &str, display_name: &str) -> Option<String> {
        let is_quantizer = crate::services::downloader::ModelDownloader::is_community_quantizer(author);

        // If it's an aggregator/quantizer (e.g. lmstudio-community, mlx-community),
        // check if the model family is identified (Gemma, Qwen, DeepSeek, etc.)
        if is_quantizer {
            if let Some(family_avatar) = crate::services::downloader::ModelDownloader::get_model_family_avatar_static(model_id, display_name) {
                let target_logo = dir.join("logo.png");
                if let Ok(client) = reqwest::Client::builder().timeout(std::time::Duration::from_secs(4)).build() {
                    if let Ok(resp) = client.get(&family_avatar).send().await {
                        if resp.status().is_success() {
                            if let Ok(bytes) = resp.bytes().await {
                                let _ = fs::write(&target_logo, &bytes).await;
                                let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                                return Some(format!("data:image/png;base64,{}", b64));
                            }
                        }
                    }
                }
                return Some(family_avatar);
            }
        }

        // 1. Check local file on disk first for non-quantizers
        if !is_quantizer {
            if let Some(local_data_uri) = Self::find_local_logo(dir).await {
                return Some(local_data_uri);
            }
        }

        // 2. Check static known logos
        let static_avatar = crate::services::downloader::ModelDownloader::get_author_avatar_static(author);

        // 3. If not static, query HF overview API
        let avatar_url = if let Some(s) = static_avatar {
            Some(s)
        } else {
            let clean_author = author.trim();
            if clean_author.is_empty() || clean_author == "Local" || clean_author == "models" {
                None
            } else {
                let client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(3))
                    .build()
                    .ok()?;

                let mut found_url = None;
                let org_url = format!("https://huggingface.co/api/organizations/{}/overview", clean_author);
                if let Ok(resp) = client.get(&org_url).send().await {
                    if resp.status().is_success() {
                        if let Ok(json) = resp.json::<Value>().await {
                            if let Some(av) = json.get("avatarUrl").and_then(|v| v.as_str()) {
                                found_url = Some(crate::services::downloader::ModelDownloader::normalize_avatar_url(av));
                            }
                        }
                    }
                }

                if found_url.is_none() {
                    let user_url = format!("https://huggingface.co/api/users/{}/overview", clean_author);
                    if let Ok(resp) = client.get(&user_url).send().await {
                        if resp.status().is_success() {
                            if let Ok(json) = resp.json::<Value>().await {
                                if let Some(av) = json.get("avatarUrl").and_then(|v| v.as_str()) {
                                    found_url = Some(crate::services::downloader::ModelDownloader::normalize_avatar_url(av));
                                }
                            }
                        }
                    }
                }

                found_url
            }
        };

        if let Some(url) = avatar_url {
            let target_logo = dir.join("logo.png");
            if let Ok(client) = reqwest::Client::builder().timeout(std::time::Duration::from_secs(4)).build() {
                if let Ok(resp) = client.get(&url).send().await {
                    if resp.status().is_success() {
                        if let Ok(bytes) = resp.bytes().await {
                            let _ = fs::write(&target_logo, &bytes).await;
                            let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                            return Some(format!("data:image/png;base64,{}", b64));
                        }
                    }
                }
            }
            return Some(url);
        }

        None
    }

    /// Inspects an MLX model folder (safetensors + config.json)
    async fn inspect_mlx_directory(dir: &Path) -> Option<ModelInfo> {
        let dir_name = dir.file_name()?.to_string_lossy().to_string();
        let parent_name = dir.parent()
            .and_then(|p| p.file_name())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "Local".to_string());

        let (author_name, raw_repo_name) = match dir_name.split_once("___") {
            Some((auth, repo)) => (auth.to_string(), repo.to_string()),
            None => {
                let a = if parent_name != "models" && !parent_name.is_empty() {
                    parent_name.clone()
                } else {
                    "Local".to_string()
                };
                (a, dir_name.clone())
            }
        };

        let model_id = format!("{}/{}", author_name, raw_repo_name);

        let config_path = dir.join("config.json");
        if !config_path.exists() {
            return None;
        }
        let (arch, ctx_len, model_type) = if let Ok(content) = fs::read_to_string(&config_path).await {
            if let Ok(json) = serde_json::from_str::<Value>(&content) {
                let arch = json.get("architectures")
                    .and_then(|a| a.as_array())
                    .and_then(|a| a.first())
                    .and_then(|v| v.as_str())
                    .map(|s| s.replace("ForCausalLM", "").replace("ForConditionalGeneration", "").to_lowercase())
                    .or_else(|| json.get("model_type").and_then(|v| v.as_str()).map(|s| s.to_string()))
                    .unwrap_or_else(|| "transformer".to_string());

                let ctx = json.get("max_position_embeddings")
                    .or_else(|| json.get("context_length"))
                    .or_else(|| json.get("seq_length"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(32768) as usize;

                let m_type = json.get("model_type").and_then(|v| v.as_str()).unwrap_or("mlx").to_string();
                (arch, ctx, m_type)
            } else {
                ("mlx".to_string(), 32768, "mlx".to_string())
            }
        } else {
            return None;
        };

        // Compute total folder size in GB
        let mut total_bytes: u64 = 0;
        let mut has_weights = false;
        if let Ok(mut entries) = fs::read_dir(dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(meta) = entry.metadata().await {
                    if meta.is_file() {
                        let fname = entry.file_name().to_string_lossy().to_lowercase();
                        if !fname.ends_with(".part") {
                            total_bytes += meta.len();
                            if fname.ends_with(".safetensors") || fname.ends_with(".npz") || fname.ends_with(".bin") {
                                has_weights = true;
                            }
                        }
                    }
                }
            }
        }
        if !has_weights || total_bytes < 50_000_000 {
            return None;
        }
        let size_gb = (total_bytes as f32) / (1024.0 * 1024.0 * 1024.0);

        // Detect quantization
        let dir_lower = dir_name.to_lowercase();
        let quantization = if dir_lower.contains("1bit") || dir_lower.contains("1-bit") {
            "1-bit MLX".to_string()
        } else if dir_lower.contains("4bit") || dir_lower.contains("4-bit") {
            "4-bit MLX".to_string()
        } else if dir_lower.contains("8bit") || dir_lower.contains("8-bit") {
            "8-bit MLX".to_string()
        } else if dir_lower.contains("bf16") || dir_lower.contains("fp16") {
            "16-bit MLX".to_string()
        } else {
            "MLX Quantized".to_string()
        };

        // Human readable display name matching HF model title
        let display_name = Self::format_model_name(&raw_repo_name);

        let arch_final = if arch.is_empty() { model_type } else { arch };
        let (supports_tools, supports_vision, supports_thinking) =
            Self::detect_capabilities(&model_id, &display_name, &arch_final, &dir_name);

        // Resolve logo from local disk or download from HF
        let author_avatar_url = Self::resolve_and_save_logo(dir, &author_name, &model_id, &display_name).await;

        Some(ModelInfo {
            id: model_id,
            name: display_name,
            author: author_name,
            architecture: arch_final,
            quantization,
            size_gb: if size_gb > 0.05 { size_gb } else { 4.0 },
            context_length: ctx_len,
            description: "MLX model optimized for Apple Silicon with Metal acceleration.".into(),
            status: ModelStatus::Downloaded,
            downloads_count: 50000,
            is_featured: true,
            format: ModelFormat::Mlx,
            backend: BackendType::MlxLm,
            local_path: Some(dir.to_string_lossy().to_string()),
            supports_tools,
            supports_vision,
            supports_thinking,
            author_avatar_url,
        })
    }

    /// Inspects a GGUF file
    async fn inspect_gguf_file(file: &Path) -> Option<ModelInfo> {
        // 1. Require .gguf extension — reject any file that is not a GGUF model
        if let Some(ext) = file.extension() {
            if ext.to_string_lossy().to_lowercase() != "gguf" {
                return None;
            }
        } else {
            // No extension at all — definitely not a GGUF model file
            return None;
        }

        // 2. Validate GGUF magic bytes (first 4 bytes must be "GGUF")
        if let Ok(mut f) = tokio::fs::File::open(file).await {
            use tokio::io::AsyncReadExt;
            let mut magic = [0u8; 4];
            if f.read_exact(&mut magic).await.is_err() || &magic != b"GGUF" {
                log::debug!("Ignored file (not a valid GGUF): {:?}", file);
                return None;
            }
        } else {
            return None;
        }

        let file_name = file.file_stem()?.to_string_lossy().to_string();
        let file_lower = file_name.to_lowercase();

        // Multimodal Projectors (mmproj) are companion projection weights for vision, not standalone LLM models
        if file_lower.starts_with("mmproj") || file_lower.contains("mmproj-") || file_lower.contains("mmproj_") {
            return None;
        }

        // Deprecated / removed GGUF quantization formats that llama.cpp no longer supports (e.g. Q4_0_4_4, Q4_0_4_8, Q4_0_8_8)
        if file_lower.contains("q4_0_4_4") || file_lower.contains("q4_0_4_8") || file_lower.contains("q4_0_8_8") {
            log::info!("Ignoring model with deprecated/removed quantization format: {:?}", file);
            return None;
        }

        let parent_dir = file.parent()
            .and_then(|p| p.file_name())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "Local".to_string());

        let (author_name, raw_name) = match file_name.split_once("___") {
            Some((auth, model_part)) => (auth.to_string(), model_part.to_string()),
            None => {
                let a = if parent_dir.contains("___") {
                    parent_dir.split_once("___").unwrap().0.to_string()
                } else if parent_dir != "models" && !parent_dir.is_empty() {
                    parent_dir.clone()
                } else {
                    "Local".to_string()
                };
                (a, file_name.clone())
            }
        };

        let meta = fs::metadata(file).await.ok()?;
        let size_gb = (meta.len() as f32) / (1024.0 * 1024.0 * 1024.0);

        // Detect quantization from filename (e.g. Q4_K_M, Q5_K_S, Q8_0, IQ3_M, etc.)
        let quantization = if let Some(pos) = file_name.find("-Q") {
            file_name[pos + 1..].to_string()
        } else if let Some(pos) = file_name.find("_Q") {
            file_name[pos + 1..].to_string()
        } else if let Some(pos) = file_name.find("Q4_") {
            file_name[pos..].to_string()
        } else if let Some(pos) = file_name.find("Q8_") {
            file_name[pos..].to_string()
        } else if let Some(pos) = file_name.find("QAT-") {
            file_name[pos..].to_string()
        } else {
            "GGUF".to_string()
        };

        let clean_name = Self::format_model_name(&raw_name);

        let name_lower = clean_name.to_lowercase();
        let architecture = if name_lower.contains("gemma") {
            "gemma".to_string()
        } else if name_lower.contains("granite") {
            "granite".to_string()
        } else if name_lower.contains("nemotron") {
            "nemotron".to_string()
        } else if name_lower.contains("llama") {
            "llama".to_string()
        } else if name_lower.contains("qwen") {
            "qwen2".to_string()
        } else if name_lower.contains("deepseek") {
            "deepseek".to_string()
        } else if name_lower.contains("mistral") || name_lower.contains("mixtral") {
            "mistral".to_string()
        } else if name_lower.contains("phi") {
            "phi".to_string()
        } else {
            "gguf".to_string()
        };

        let (supports_tools, supports_vision, supports_thinking) =
            Self::detect_capabilities(&file_name, &clean_name, &architecture, &parent_dir);

        let is_quantizer = crate::services::downloader::ModelDownloader::is_community_quantizer(&author_name);
        let family_avatar = if is_quantizer {
            crate::services::downloader::ModelDownloader::get_model_family_avatar_static(&raw_name, &clean_name)
        } else {
            None
        };

        let gguf_logo_candidate = file.with_extension("logo.png");
        let author_avatar_url = if let Some(fav) = family_avatar {
            Some(fav)
        } else if gguf_logo_candidate.is_file() && !is_quantizer {
            if let Ok(bytes) = fs::read(&gguf_logo_candidate).await {
                let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                Some(format!("data:image/png;base64,{}", b64))
            } else {
                None
            }
        } else if let Some(parent) = file.parent() {
            Self::resolve_and_save_logo(parent, &author_name, &format!("{}/{}", author_name, raw_name), &clean_name).await
        } else {
            None
        };

        Some(ModelInfo {
            id: format!("{}/{}", author_name, raw_name),
            name: clean_name,
            author: author_name,
            architecture,
            quantization,
            size_gb,
            context_length: 32768,
            description: "GGUF model ready for execution via Ollama or Atena Native Server.".into(),
            status: ModelStatus::Downloaded,
            downloads_count: 25000,
            is_featured: false,
            format: ModelFormat::Gguf,
            backend: BackendType::Ollama,
            local_path: Some(file.to_string_lossy().to_string()),
            supports_tools,
            supports_vision,
            supports_thinking,
            author_avatar_url,
        })
    }

    /// Queries Ollama /api/tags endpoint to discover models already registered in Ollama
    pub async fn scan_ollama(host: &str, port: u16) -> Vec<ModelInfo> {
        let url = format!("http://{}:{}/api/tags", host, port);
        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(800))
            .build()
        {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        let t_start = std::time::Instant::now();
        let response = match client.get(&url).send().await {
            Ok(r) => {
                let latency = t_start.elapsed().as_millis() as u64;
                crate::services::server_ctl::LocalServerController::record_log(
                    "GET",
                    &url,
                    r.status().as_u16(),
                    latency,
                    0,
                    0,
                );
                r
            }
            Err(_) => return Vec::new(),
        };

        let mut models = Vec::new();
        if let Ok(json) = response.json::<Value>().await {
            if let Some(list) = json.get("models").and_then(|m| m.as_array()) {
                for item in list {
                    let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    if name.is_empty() {
                        continue;
                    }
                    let size_bytes = item.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
                    let size_gb = (size_bytes as f32) / (1024.0 * 1024.0 * 1024.0);

                    let details = item.get("details");
                    let family = details
                        .and_then(|d| d.get("family"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("ollama")
                        .to_string();
                    let quant = details
                        .and_then(|d| d.get("quantization_level"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("Q4_0")
                        .to_string();

                    let (supports_tools, supports_vision, supports_thinking) =
                        Self::detect_capabilities(&name, &name, &family, "Ollama");

                    models.push(ModelInfo {
                        id: format!("ollama/{}", name),
                        name: name.clone(),
                        author: "Ollama".to_string(),
                        architecture: family,
                        quantization: quant,
                        size_gb: if size_gb > 0.1 { size_gb } else { 4.0 },
                        context_length: 32768,
                        description: "Model managed via local Ollama service.".into(),
                        status: ModelStatus::Downloaded,
                        downloads_count: 100000,
                        is_featured: false,
                        format: ModelFormat::Ollama,
                        backend: BackendType::Ollama,
                        local_path: None,
                        supports_tools,
                        supports_vision,
                        supports_thinking,
                        author_avatar_url: None,
                    });
                }
            }
        }

        models
    }

    /// Helper to detect thinking, vision and tool capabilities from model identifiers and architecture
    pub fn detect_capabilities(id: &str, name: &str, architecture: &str, context: &str) -> (bool, bool, bool) {
        let lower = format!("{} {} {} {}", id, name, architecture, context).to_lowercase();

        // Check if embedding model (non-generative)
        let is_embedding = lower.contains("embed")
            || lower.contains("embedding")
            || lower.contains("bge-")
            || lower.contains("all-minilm");

        if is_embedding {
            return (false, false, false);
        }

        // 1. Thinking / Reasoning capability
        let supports_thinking = lower.contains("deepseek-r1")
            || lower.contains("r1-distill")
            || lower.contains("qwq")
            || lower.contains("qwen3")
            || lower.contains("ornith")
            || lower.contains("reasoning")
            || lower.contains("thinking")
            || lower.contains("cot")
            || lower.contains("gemma-4")
            || lower.contains("gemma 4")
            || lower.contains("gemma4")
            || lower.contains("gemma-3")
            || lower.contains("gemma 3")
            || lower.contains("gemma3")
            || lower.contains("phi-4")
            || lower.contains("phi4")
            || lower.contains("o1")
            || lower.contains("o3")
            || lower.contains("r1");

        // 2. Vision / Multimodal capability (Image + Text)
        let supports_vision = lower.contains("-vl")
            || lower.contains("vision")
            || lower.contains("llava")
            || lower.contains("pixtral")
            || lower.contains("multimodal")
            || lower.contains("clip")
            || lower.contains("qwen2-vl")
            || lower.contains("minicpm-v")
            || lower.contains("gemma-4")
            || lower.contains("gemma 4")
            || lower.contains("gemma4")
            || lower.contains("gemma-3")
            || lower.contains("gemma 3")
            || lower.contains("gemma3")
            || lower.contains("paligemma")
            || lower.contains("mplug")
            || lower.contains("internvl")
            || lower.contains("omni");

        // 3. Tool / Function Calling capability
        let is_base = (lower.contains("base") && !lower.contains("database") && !lower.contains("-it") && !lower.contains("instruct"))
            || (lower.contains("-base") && !lower.contains("-it"));
        let is_pure_deepseek_r1 = lower.contains("deepseek-r1") && !lower.contains("qwen") && !lower.contains("llama");

        let is_instruct_or_chat = lower.contains("-it")
            || lower.contains("_it")
            || lower.contains(" it")
            || lower.contains("-instruct")
            || lower.contains("_instruct")
            || lower.contains("instruct")
            || lower.contains("chat");

        let is_known_tool_family = lower.contains("gemma")
            || lower.contains("granite")
            || lower.contains("qwen")
            || lower.contains("llama")
            || lower.contains("mistral")
            || lower.contains("mixtral")
            || lower.contains("hermes")
            || lower.contains("command-r")
            || lower.contains("functionary")
            || lower.contains("nemotron")
            || lower.contains("firefunction")
            || lower.contains("gorilla")
            || lower.contains("tool")
            || lower.contains("agent")
            || lower.contains("phi-4")
            || lower.contains("phi-3.5")
            || lower.contains("phi-3")
            || lower.contains("smollm2")
            || lower.contains("exaone")
            || lower.contains("solar")
            || lower.contains("deepseek-chat");

        let is_tool_capable = (is_known_tool_family || is_instruct_or_chat) && !is_base && !is_pure_deepseek_r1;

        let supports_tools = is_tool_capable;

        (supports_tools, supports_vision, supports_thinking)
    }

    /// Discovers Antigravity models dynamically by querying the `agy models` CLI command
    pub async fn scan_antigravity() -> Vec<ModelInfo> {
        let agy_path_opt = crate::services::backend::BackendManager::find_agy_binary();
        let bin_str = agy_path_opt
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "agy".to_string());

        let mut models = Vec::new();

        if let Some(ref agy_path) = agy_path_opt {
            // Query the CLI for available models
            let mut cmd = tokio::process::Command::new(agy_path);
            cmd.arg("models");
            cmd.env("PATH", crate::services::backend::BackendManager::augmented_path());
            cmd.stdout(std::process::Stdio::piped());
            cmd.stderr(std::process::Stdio::null());

            let output_res = tokio::time::timeout(
                std::time::Duration::from_secs(12),
                cmd.output()
            ).await;

        if let Ok(Ok(output)) = output_res {
            if output.status.success() {
                let stdout_text = String::from_utf8_lossy(&output.stdout);
                for line in stdout_text.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with("Fetching") {
                        continue;
                    }

                    let (raw_id, raw_name) = if let Some((id, name)) = trimmed.split_once('\t') {
                        (id.trim(), name.trim())
                    } else {
                        let mut sp = trimmed.split_whitespace();
                        let id = sp.next().unwrap_or("");
                        let name = sp.collect::<Vec<_>>().join(" ");
                        (id, if name.is_empty() { id } else { "" })
                    };

                    if raw_id.is_empty() {
                        continue;
                    }

                    let model_name = if raw_name.is_empty() {
                        raw_id.to_string()
                    } else {
                        raw_name.to_string()
                    };

                    let id_lower = raw_id.to_lowercase();
                    let author = if id_lower.starts_with("gemini") {
                        "Google DeepMind"
                    } else if id_lower.starts_with("claude") {
                        "Anthropic"
                    } else if id_lower.starts_with("gpt") {
                        "OpenAI"
                    } else {
                        "Antigravity"
                    };

                    let architecture = if id_lower.contains("gemini-3.8") {
                        "gemini-3.8"
                    } else if id_lower.contains("gemini-3.7") {
                        "gemini-3.7"
                    } else if id_lower.contains("gemini-3.6") {
                        "gemini-3.6"
                    } else if id_lower.contains("gemini-3.1") {
                        "gemini-3.1"
                    } else if id_lower.contains("claude") {
                        "claude"
                    } else {
                        "transformer"
                    };

                    let context_length = if id_lower.contains("gemini-3.1") {
                        2097152
                    } else if id_lower.contains("gemini") {
                        1048576
                    } else if id_lower.contains("claude") {
                        200000
                    } else {
                        131072
                    };

                    let description = if id_lower.contains("3.8") {
                        "Next-generation ultra-high-speed multimodal model with hybrid reasoning and superior technical precision.".into()
                    } else if id_lower.contains("flash") {
                        "Ultra-fast multimodal model with hybrid reasoning and real-time streaming.".into()
                    } else if id_lower.contains("pro") {
                        "Maximum reasoning capacity with a massive 2 million token context window.".into()
                    } else if id_lower.contains("claude") {
                        "Cutting-edge software engineering, technical synthesis, and deep code analysis.".into()
                    } else if id_lower.contains("gpt") {
                        "High-scale model for analytical reasoning and structured code generation.".into()
                    } else {
                        "Cloud language model integrated via Antigravity CLI.".into()
                    };

                    let is_featured = raw_id == "gemini-3.8-flash-medium" || raw_id == "claude-sonnet-4-6";

                    let author_avatar_url = crate::services::downloader::ModelDownloader::get_author_avatar_static(author);

                    models.push(ModelInfo {
                        id: format!("agy/{}", raw_id),
                        name: model_name,
                        author: author.into(),
                        architecture: architecture.into(),
                        quantization: "Antigravity".into(),
                        size_gb: 0.0,
                        context_length,
                        description,
                        status: ModelStatus::Loaded { vram_usage_mb: 0, ram_usage_mb: 0 },
                        downloads_count: 2000000,
                        is_featured,
                        format: ModelFormat::Agy,
                        backend: BackendType::Antigravity,
                        local_path: Some(bin_str.clone()),
                        supports_tools: true,
                        supports_vision: true,
                        supports_thinking: true,
                        author_avatar_url,
                    });
                }
            }
        }
    }

        // If dynamic discovery produced models, return them
        if !models.is_empty() {
            return models;
        }

        // Fallback baseline if CLI execution failed or produced empty list
        vec![
            ModelInfo {
                id: "agy/gemini-3.8-flash-medium".into(),
                name: "Gemini 3.8 Flash (Medium)".into(),
                author: "Google DeepMind".into(),
                architecture: "gemini-3.8".into(),
                quantization: "Antigravity".into(),
                size_gb: 0.0,
                context_length: 1048576,
                description: "Next-generation ultra-high-speed multimodal model with balanced reasoning.".into(),
                status: ModelStatus::Loaded { vram_usage_mb: 0, ram_usage_mb: 0 },
                downloads_count: 2500000,
                is_featured: true,
                format: ModelFormat::Agy,
                backend: BackendType::Antigravity,
                local_path: Some(bin_str.clone()),
                supports_tools: true,
                supports_vision: true,
                supports_thinking: true,
                author_avatar_url: crate::services::downloader::ModelDownloader::get_author_avatar_static("Google DeepMind"),
            },
            ModelInfo {
                id: "agy/gemini-3.8-flash-high".into(),
                name: "Gemini 3.8 Flash (High)".into(),
                author: "Google DeepMind".into(),
                architecture: "gemini-3.8".into(),
                quantization: "Antigravity".into(),
                size_gb: 0.0,
                context_length: 1048576,
                description: "Deep thinking effort for complex logic and coding tasks.".into(),
                status: ModelStatus::Loaded { vram_usage_mb: 0, ram_usage_mb: 0 },
                downloads_count: 1800000,
                is_featured: true,
                format: ModelFormat::Agy,
                backend: BackendType::Antigravity,
                local_path: Some(bin_str.clone()),
                supports_tools: true,
                supports_vision: true,
                supports_thinking: true,
                author_avatar_url: crate::services::downloader::ModelDownloader::get_author_avatar_static("Google DeepMind"),
            },
            ModelInfo {
                id: "agy/claude-sonnet-4-6".into(),
                name: "Claude Sonnet 4.6 (Thinking)".into(),
                author: "Anthropic".into(),
                architecture: "claude-4.6".into(),
                quantization: "Antigravity".into(),
                size_gb: 0.0,
                context_length: 200000,
                description: "Cutting-edge software engineering, reasoning, and deep technical analysis.".into(),
                status: ModelStatus::Loaded { vram_usage_mb: 0, ram_usage_mb: 0 },
                downloads_count: 1950000,
                is_featured: true,
                format: ModelFormat::Agy,
                backend: BackendType::Antigravity,
                local_path: Some(bin_str),
                supports_tools: true,
                supports_vision: true,
                supports_thinking: true,
                author_avatar_url: crate::services::downloader::ModelDownloader::get_author_avatar_static("Anthropic"),
            },
        ]
    }

    /// Discovers OpenAI models dynamically via HTTP or provides curated standard models
    pub async fn scan_openai(api_key: &str, base_url: Option<&str>) -> Vec<ModelInfo> {
        let base = base_url.unwrap_or("https://api.openai.com/v1").trim_end_matches('/');
        let mut discovered = Vec::new();

        if !api_key.trim().is_empty() {
            if let Ok(client) = reqwest::Client::builder().timeout(std::time::Duration::from_secs(6)).build() {
                let url = format!("{}/models", base);
                if let Ok(resp) = client.get(&url).header("Authorization", format!("Bearer {}", api_key.trim())).send().await {
                    if resp.status().is_success() {
                        if let Ok(val) = resp.json::<serde_json::Value>().await {
                            if let Some(arr) = val.get("data").and_then(|d| d.as_array()) {
                                for m in arr {
                                    if let Some(raw_id) = m.get("id").and_then(|i| i.as_str()) {
                                        // Filter to keep chat-compatible models
                                        if raw_id.starts_with("gpt-") || raw_id.starts_with("o1") || raw_id.starts_with("o3") || raw_id.starts_with("chatgpt-") {
                                            let is_featured = raw_id == "gpt-4o" || raw_id == "gpt-4o-mini" || raw_id == "o1" || raw_id == "o3-mini";
                                            let is_reasoning = raw_id.starts_with("o1") || raw_id.starts_with("o3");
                                            discovered.push(ModelInfo {
                                                id: format!("openai/{}", raw_id),
                                                name: format!("OpenAI {}", raw_id.to_uppercase()),
                                                author: "OpenAI".into(),
                                                architecture: if is_reasoning { "o-series".into() } else { "gpt-4".into() },
                                                quantization: "Cloud API".into(),
                                                size_gb: 0.0,
                                                context_length: 128000,
                                                description: format!("Official OpenAI model ({}) running in the cloud.", raw_id),
                                                status: ModelStatus::Loaded { vram_usage_mb: 0, ram_usage_mb: 0 },
                                                downloads_count: 8000000,
                                                is_featured,
                                                format: ModelFormat::Cloud,
                                                backend: BackendType::CloudOpenAi,
                                                local_path: Some(base.to_string()),
                                                supports_tools: true,
                                                supports_vision: raw_id.contains("4o") || raw_id.contains("vision"),
                                                supports_thinking: is_reasoning,
                                                author_avatar_url: crate::services::downloader::ModelDownloader::get_author_avatar_static("OpenAI"),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if !discovered.is_empty() {
            return discovered;
        }

        // Curated default OpenAI catalog
        vec![
            ModelInfo {
                id: "openai/gpt-4o".into(),
                name: "GPT-4o (Omni)".into(),
                author: "OpenAI".into(),
                architecture: "gpt-4o".into(),
                quantization: "Cloud API".into(),
                size_gb: 0.0,
                context_length: 128000,
                description: "Top-of-the-line OpenAI multimodal model with high speed and precision.".into(),
                status: ModelStatus::Loaded { vram_usage_mb: 0, ram_usage_mb: 0 },
                downloads_count: 10000000,
                is_featured: true,
                format: ModelFormat::Cloud,
                backend: BackendType::CloudOpenAi,
                local_path: Some(base.to_string()),
                supports_tools: true,
                supports_vision: true,
                supports_thinking: false,
                author_avatar_url: crate::services::downloader::ModelDownloader::get_author_avatar_static("OpenAI"),
            },
            ModelInfo {
                id: "openai/gpt-4o-mini".into(),
                name: "GPT-4o Mini".into(),
                author: "OpenAI".into(),
                architecture: "gpt-4o".into(),
                quantization: "Cloud API".into(),
                size_gb: 0.0,
                context_length: 128000,
                description: "Ultra-fast and cost-effective version with vision and agile reasoning capabilities.".into(),
                status: ModelStatus::Loaded { vram_usage_mb: 0, ram_usage_mb: 0 },
                downloads_count: 7500000,
                is_featured: true,
                format: ModelFormat::Cloud,
                backend: BackendType::CloudOpenAi,
                local_path: Some(base.to_string()),
                supports_tools: true,
                supports_vision: true,
                supports_thinking: false,
                author_avatar_url: crate::services::downloader::ModelDownloader::get_author_avatar_static("OpenAI"),
            },
            ModelInfo {
                id: "openai/o3-mini".into(),
                name: "OpenAI o3-mini".into(),
                author: "OpenAI".into(),
                architecture: "o-series".into(),
                quantization: "Cloud API".into(),
                size_gb: 0.0,
                context_length: 200000,
                description: "Latest generation deep reasoning model for code, math, and science.".into(),
                status: ModelStatus::Loaded { vram_usage_mb: 0, ram_usage_mb: 0 },
                downloads_count: 4500000,
                is_featured: true,
                format: ModelFormat::Cloud,
                backend: BackendType::CloudOpenAi,
                local_path: Some(base.to_string()),
                supports_tools: true,
                supports_vision: false,
                supports_thinking: true,
                author_avatar_url: crate::services::downloader::ModelDownloader::get_author_avatar_static("OpenAI"),
            },
        ]
    }

    /// Discovers OpenRouter models or provides standard aggregated cloud models
    pub async fn scan_openrouter(api_key: &str) -> Vec<ModelInfo> {
        let mut discovered = Vec::new();

        if !api_key.trim().is_empty() {
            if let Ok(client) = reqwest::Client::builder().timeout(std::time::Duration::from_secs(6)).build() {
                let url = "https://openrouter.ai/api/v1/models";
                if let Ok(resp) = client.get(url)
                    .header("Authorization", format!("Bearer {}", api_key.trim()))
                    .header("HTTP-Referer", "https://atena.studio")
                    .header("X-Title", "Atena Studio")
                    .send().await
                {
                    if resp.status().is_success() {
                        if let Ok(val) = resp.json::<serde_json::Value>().await {
                            if let Some(arr) = val.get("data").and_then(|d| d.as_array()) {
                                for m in arr.iter().take(30) {
                                    if let Some(raw_id) = m.get("id").and_then(|i| i.as_str()) {
                                        let name = m.get("name").and_then(|n| n.as_str()).unwrap_or(raw_id);
                                        let desc = m.get("description").and_then(|d| d.as_str()).unwrap_or("OpenRouter Model");
                                        let ctx = m.get("context_length").and_then(|c| c.as_u64()).unwrap_or(128000) as usize;

                                        discovered.push(ModelInfo {
                                            id: format!("openrouter/{}", raw_id),
                                            name: name.to_string(),
                                            author: "OpenRouter".into(),
                                            architecture: "router".into(),
                                            quantization: "Cloud API".into(),
                                            size_gb: 0.0,
                                            context_length: ctx,
                                            description: desc.to_string(),
                                            status: ModelStatus::Loaded { vram_usage_mb: 0, ram_usage_mb: 0 },
                                            downloads_count: 6000000,
                                            is_featured: raw_id.contains("claude-3.5") || raw_id.contains("deepseek-r1"),
                                            format: ModelFormat::Cloud,
                                            backend: BackendType::CloudOpenAi,
                                            local_path: Some("https://openrouter.ai/api/v1".to_string()),
                                            supports_tools: true,
                                            supports_vision: true,
                                            supports_thinking: raw_id.contains("r1") || raw_id.contains("thinking"),
                                            author_avatar_url: None,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if !discovered.is_empty() {
            return discovered;
        }

        vec![
            ModelInfo {
                id: "openrouter/anthropic/claude-3.5-sonnet".into(),
                name: "Claude 3.5 Sonnet (OpenRouter)".into(),
                author: "Anthropic / OpenRouter".into(),
                architecture: "claude-3.5".into(),
                quantization: "Cloud API".into(),
                size_gb: 0.0,
                context_length: 200000,
                description: "High-precision reasoning and software engineering model from Anthropic.".into(),
                status: ModelStatus::Loaded { vram_usage_mb: 0, ram_usage_mb: 0 },
                downloads_count: 9000000,
                is_featured: true,
                format: ModelFormat::Cloud,
                backend: BackendType::CloudOpenAi,
                local_path: Some("https://openrouter.ai/api/v1".to_string()),
                supports_tools: true,
                supports_vision: true,
                supports_thinking: true,
                author_avatar_url: crate::services::downloader::ModelDownloader::get_author_avatar_static("Anthropic"),
            },
            ModelInfo {
                id: "openrouter/deepseek/deepseek-r1".into(),
                name: "DeepSeek R1 (OpenRouter)".into(),
                author: "DeepSeek / OpenRouter".into(),
                architecture: "deepseek-v3".into(),
                quantization: "Cloud API".into(),
                size_gb: 0.0,
                context_length: 64000,
                description: "State-of-the-art open deep reasoning model running on high-speed cloud infrastructure.".into(),
                status: ModelStatus::Loaded { vram_usage_mb: 0, ram_usage_mb: 0 },
                downloads_count: 5000000,
                is_featured: true,
                format: ModelFormat::Cloud,
                backend: BackendType::CloudOpenAi,
                local_path: Some("https://openrouter.ai/api/v1".to_string()),
                supports_tools: true,
                supports_vision: false,
                supports_thinking: true,
                author_avatar_url: crate::services::downloader::ModelDownloader::get_author_avatar_static("DeepSeek"),
            },
        ]
    }

    /// Discovers Groq Cloud models or provides high-speed curated LPU models
    pub async fn scan_groq(api_key: &str) -> Vec<ModelInfo> {
        let mut discovered = Vec::new();

        if !api_key.trim().is_empty() {
            if let Ok(client) = reqwest::Client::builder().timeout(std::time::Duration::from_secs(6)).build() {
                let url = "https://api.groq.com/openai/v1/models";
                if let Ok(resp) = client.get(url)
                    .header("Authorization", format!("Bearer {}", api_key.trim()))
                    .send().await
                {
                    if resp.status().is_success() {
                        if let Ok(val) = resp.json::<serde_json::Value>().await {
                            if let Some(arr) = val.get("data").and_then(|d| d.as_array()) {
                                for m in arr {
                                    if let Some(raw_id) = m.get("id").and_then(|i| i.as_str()) {
                                        let is_active = m.get("active").and_then(|a| a.as_bool()).unwrap_or(true);
                                        if !is_active {
                                            continue;
                                        }

                                        let lower = raw_id.to_lowercase();
                                        // Filter out audio, whisper, embedding, and safety guard models
                                        if lower.contains("whisper") || lower.contains("embed") || lower.contains("guard") || lower.contains("tts") || lower.contains("rerank") {
                                            continue;
                                        }

                                        let is_reasoning = lower.contains("r1") || lower.contains("qwq");
                                        let is_vision = lower.contains("vision") || lower.contains("vl");
                                        let is_featured = lower.contains("70b-versatile") || lower.contains("8b-instant") || lower.contains("r1");

                                        let (author, avatar_name) = if lower.contains("llama") {
                                            ("Meta", "Meta")
                                        } else if lower.contains("deepseek") {
                                            ("DeepSeek", "DeepSeek")
                                        } else if lower.contains("mixtral") || lower.contains("mistral") {
                                            ("Mistral AI", "Mistral AI")
                                        } else if lower.contains("gemma") {
                                            ("Google", "Google")
                                        } else if lower.contains("qwen") {
                                            ("Qwen", "Qwen")
                                        } else {
                                            ("Groq", "Groq")
                                        };

                                        let ctx_len = if lower.contains("32768") {
                                            32768
                                        } else if lower.contains("8192") || lower.contains("gemma") {
                                            8192
                                        } else {
                                            128000
                                        };

                                        discovered.push(ModelInfo {
                                            id: format!("groq/{}", raw_id),
                                            name: format!("{} (Groq)", raw_id),
                                            author: author.into(),
                                            architecture: "groq-lpu".into(),
                                            quantization: "Groq LPU".into(),
                                            size_gb: 0.0,
                                            context_length: ctx_len,
                                            description: format!("Ultra-high-speed inference on Groq LPU hardware ({}).", raw_id),
                                            status: ModelStatus::Loaded { vram_usage_mb: 0, ram_usage_mb: 0 },
                                            downloads_count: 7000000,
                                            is_featured,
                                            format: ModelFormat::Cloud,
                                            backend: BackendType::CloudOpenAi,
                                            local_path: Some("https://api.groq.com/openai/v1".to_string()),
                                            supports_tools: !lower.contains("distill") || lower.contains("llama"),
                                            supports_vision: is_vision,
                                            supports_thinking: is_reasoning,
                                            author_avatar_url: crate::services::downloader::ModelDownloader::get_author_avatar_static(avatar_name),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if !discovered.is_empty() {
            return discovered;
        }

        vec![
            ModelInfo {
                id: "groq/llama-3.3-70b-versatile".into(),
                name: "Meta Llama 3.3 70B Versatile".into(),
                author: "Meta".into(),
                architecture: "llama-3.3".into(),
                quantization: "Groq LPU".into(),
                size_gb: 0.0,
                context_length: 128000,
                description: "Flagship open-weights model accelerated on Groq LPUs at up to 500 tokens/sec.".into(),
                status: ModelStatus::Loaded { vram_usage_mb: 0, ram_usage_mb: 0 },
                downloads_count: 10000000,
                is_featured: true,
                format: ModelFormat::Cloud,
                backend: BackendType::CloudOpenAi,
                local_path: Some("https://api.groq.com/openai/v1".to_string()),
                supports_tools: true,
                supports_vision: false,
                supports_thinking: false,
                author_avatar_url: crate::services::downloader::ModelDownloader::get_author_avatar_static("Meta"),
            },
            ModelInfo {
                id: "groq/llama-3.1-8b-instant".into(),
                name: "Meta Llama 3.1 8B Instant".into(),
                author: "Meta".into(),
                architecture: "llama-3.1".into(),
                quantization: "Groq LPU".into(),
                size_gb: 0.0,
                context_length: 128000,
                description: "Blazing fast inference (800+ tokens/sec) for lightweight tasks, tools, and real-time generation.".into(),
                status: ModelStatus::Loaded { vram_usage_mb: 0, ram_usage_mb: 0 },
                downloads_count: 8500000,
                is_featured: true,
                format: ModelFormat::Cloud,
                backend: BackendType::CloudOpenAi,
                local_path: Some("https://api.groq.com/openai/v1".to_string()),
                supports_tools: true,
                supports_vision: false,
                supports_thinking: false,
                author_avatar_url: crate::services::downloader::ModelDownloader::get_author_avatar_static("Meta"),
            },
            ModelInfo {
                id: "groq/deepseek-r1-distill-llama-70b".into(),
                name: "DeepSeek R1 Distill Llama 70B".into(),
                author: "DeepSeek".into(),
                architecture: "deepseek-r1".into(),
                quantization: "Groq LPU".into(),
                size_gb: 0.0,
                context_length: 128000,
                description: "Deep reasoning, logic, and coding model accelerated on Groq LPU hardware.".into(),
                status: ModelStatus::Loaded { vram_usage_mb: 0, ram_usage_mb: 0 },
                downloads_count: 6500000,
                is_featured: true,
                format: ModelFormat::Cloud,
                backend: BackendType::CloudOpenAi,
                local_path: Some("https://api.groq.com/openai/v1".to_string()),
                supports_tools: true,
                supports_vision: false,
                supports_thinking: true,
                author_avatar_url: crate::services::downloader::ModelDownloader::get_author_avatar_static("DeepSeek"),
            },
            ModelInfo {
                id: "groq/mixtral-8x7b-32768".into(),
                name: "Mixtral 8x7B 32k".into(),
                author: "Mistral AI".into(),
                architecture: "moe".into(),
                quantization: "Groq LPU".into(),
                size_gb: 0.0,
                context_length: 32768,
                description: "High-throughput mixture-of-experts model for versatile multilingual workflows.".into(),
                status: ModelStatus::Loaded { vram_usage_mb: 0, ram_usage_mb: 0 },
                downloads_count: 5000000,
                is_featured: false,
                format: ModelFormat::Cloud,
                backend: BackendType::CloudOpenAi,
                local_path: Some("https://api.groq.com/openai/v1".to_string()),
                supports_tools: true,
                supports_vision: false,
                supports_thinking: false,
                author_avatar_url: crate::services::downloader::ModelDownloader::get_author_avatar_static("Mistral AI"),
            },
            ModelInfo {
                id: "groq/gemma2-9b-it".into(),
                name: "Google Gemma 2 9B IT".into(),
                author: "Google".into(),
                architecture: "gemma-2".into(),
                quantization: "Groq LPU".into(),
                size_gb: 0.0,
                context_length: 8192,
                description: "Compact and powerful Google instruction-tuned model running with ultra-low latency.".into(),
                status: ModelStatus::Loaded { vram_usage_mb: 0, ram_usage_mb: 0 },
                downloads_count: 4000000,
                is_featured: false,
                format: ModelFormat::Cloud,
                backend: BackendType::CloudOpenAi,
                local_path: Some("https://api.groq.com/openai/v1".to_string()),
                supports_tools: false,
                supports_vision: false,
                supports_thinking: false,
                author_avatar_url: crate::services::downloader::ModelDownloader::get_author_avatar_static("Google"),
            },
            ModelInfo {
                id: "groq/llama-3.2-11b-vision-preview".into(),
                name: "Meta Llama 3.2 11B Vision".into(),
                author: "Meta".into(),
                architecture: "llama-3.2-vision".into(),
                quantization: "Groq LPU".into(),
                size_gb: 0.0,
                context_length: 128000,
                description: "Multimodal visual reasoning model powered by Groq LPUs.".into(),
                status: ModelStatus::Loaded { vram_usage_mb: 0, ram_usage_mb: 0 },
                downloads_count: 3500000,
                is_featured: false,
                format: ModelFormat::Cloud,
                backend: BackendType::CloudOpenAi,
                local_path: Some("https://api.groq.com/openai/v1".to_string()),
                supports_tools: true,
                supports_vision: true,
                supports_thinking: false,
                author_avatar_url: crate::services::downloader::ModelDownloader::get_author_avatar_static("Meta"),
            },
        ]
    }

    /// Generates models from custom OpenAI-compatible endpoints
    pub async fn scan_custom(name: &str, base_url: &str, _api_key: &str, selected_model: Option<&str>) -> Vec<ModelInfo> {
        let provider_name = if name.trim().is_empty() { "Custom Cloud" } else { name.trim() };
        let model_id = selected_model.unwrap_or("default");

        vec![
            ModelInfo {
                id: format!("custom/{}", model_id),
                name: format!("{} ({})", provider_name, model_id),
                author: provider_name.into(),
                architecture: "openai-compatible".into(),
                quantization: "Cloud API".into(),
                size_gb: 0.0,
                context_length: 65536,
                description: format!("Custom OpenAI-compatible endpoint pointing to {}.", base_url),
                status: ModelStatus::Loaded { vram_usage_mb: 0, ram_usage_mb: 0 },
                downloads_count: 100000,
                is_featured: true,
                format: ModelFormat::Cloud,
                backend: BackendType::CloudOpenAi,
                local_path: Some(base_url.to_string()),
                supports_tools: true,
                supports_vision: true,
                supports_thinking: false,
                author_avatar_url: None,
            }
        ]
    }

    /// Generates or dynamically discovers models from Google Gemini
    pub async fn scan_gemini(api_key: &str) -> Vec<ModelInfo> {
        let clean_key = api_key.trim();
        if !clean_key.is_empty() {
            if let Ok(client) = reqwest::Client::builder().timeout(std::time::Duration::from_secs(6)).build() {
                let url = "https://generativelanguage.googleapis.com/v1beta/openai/models";
                if let Ok(resp) = client.get(url).header("Authorization", format!("Bearer {}", clean_key)).send().await {
                    if resp.status().is_success() {
                        if let Ok(val) = resp.json::<serde_json::Value>().await {
                            if let Some(arr) = val.get("data").and_then(|d| d.as_array()) {
                                let mut discovered = Vec::new();
                                for m in arr {
                                    if let Some(raw_id) = m.get("id").and_then(|i| i.as_str()) {
                                        if raw_id.starts_with("gemini-") && !raw_id.contains("embedding") && !raw_id.contains("aqa") {
                                            let is_thinking = raw_id.contains("thinking") || raw_id.contains("2.5-pro") || raw_id.contains("2.5-flash");
                                            let is_featured = raw_id == "gemini-2.5-pro" || raw_id == "gemini-2.5-flash" || raw_id == "gemini-2.5-flash-lite";
                                            let ctx = if raw_id.contains("pro") { 2097152 } else { 1048576 };
                                            discovered.push(ModelInfo {
                                                id: format!("gemini/{}", raw_id),
                                                name: format!("Gemini {}", Self::format_model_name(raw_id)),
                                                author: "Google".into(),
                                                architecture: "gemini".into(),
                                                quantization: "Cloud API".into(),
                                                size_gb: 0.0,
                                                context_length: ctx,
                                                description: format!("Official Google Gemini model ({}) via Google AI Studio.", raw_id),
                                                status: ModelStatus::Loaded { vram_usage_mb: 0, ram_usage_mb: 0 },
                                                downloads_count: 1000000,
                                                is_featured,
                                                format: ModelFormat::Cloud,
                                                backend: BackendType::CloudOpenAi,
                                                local_path: None,
                                                supports_tools: true,
                                                supports_vision: true,
                                                supports_thinking: is_thinking,
                                                author_avatar_url: None,
                                            });
                                        }
                                    }
                                }
                                if !discovered.is_empty() {
                                    return discovered;
                                }
                            }
                        }
                    }
                }

                // 2. Try native Google AI Studio models endpoint as robust fallback
                let native_url = format!("https://generativelanguage.googleapis.com/v1beta/models?key={}", clean_key);
                if let Ok(resp) = client.get(&native_url).send().await {
                    if resp.status().is_success() {
                        if let Ok(val) = resp.json::<serde_json::Value>().await {
                            if let Some(arr) = val.get("models").and_then(|d| d.as_array()) {
                                let mut discovered = Vec::new();
                                for m in arr {
                                    if let Some(full_name) = m.get("name").and_then(|i| i.as_str()) {
                                        let raw_id = full_name.trim_start_matches("models/");
                                        if raw_id.starts_with("gemini-") && !raw_id.contains("embedding") && !raw_id.contains("aqa") {
                                            let is_thinking = raw_id.contains("thinking") || raw_id.contains("2.5-pro") || raw_id.contains("2.5-flash");
                                            let is_featured = raw_id == "gemini-2.5-pro" || raw_id == "gemini-2.5-flash" || raw_id == "gemini-2.5-flash-lite";
                                            let ctx = m.get("inputTokenLimit").and_then(|c| c.as_u64()).map(|c| c as usize).unwrap_or_else(|| {
                                                if raw_id.contains("pro") { 2097152 } else { 1048576 }
                                            });
                                            let display_name = m.get("displayName").and_then(|d| d.as_str()).unwrap_or("");
                                            let name = if !display_name.is_empty() {
                                                display_name.to_string()
                                            } else {
                                                format!("Gemini {}", Self::format_model_name(raw_id))
                                            };
                                            let desc = m.get("description").and_then(|d| d.as_str()).unwrap_or("");
                                            let description = if !desc.is_empty() {
                                                desc.to_string()
                                            } else {
                                                format!("Official Google Gemini model ({}) via Google AI Studio.", raw_id)
                                            };

                                            discovered.push(ModelInfo {
                                                id: format!("gemini/{}", raw_id),
                                                name,
                                                author: "Google".into(),
                                                architecture: "gemini".into(),
                                                quantization: "Cloud API".into(),
                                                size_gb: 0.0,
                                                context_length: ctx,
                                                description,
                                                status: ModelStatus::Loaded { vram_usage_mb: 0, ram_usage_mb: 0 },
                                                downloads_count: 1000000,
                                                is_featured,
                                                format: ModelFormat::Cloud,
                                                backend: BackendType::CloudOpenAi,
                                                local_path: None,
                                                supports_tools: true,
                                                supports_vision: true,
                                                supports_thinking: is_thinking,
                                                author_avatar_url: None,
                                            });
                                        }
                                    }
                                }
                                if !discovered.is_empty() {
                                    return discovered;
                                }
                            }
                        }
                    }
                }
            }
        }

        vec![
            ModelInfo {
                id: "gemini/gemini-2.5-pro".into(),
                name: "Gemini 2.5 Pro".into(),
                author: "Google".into(),
                architecture: "gemini".into(),
                quantization: "Cloud API".into(),
                size_gb: 0.0,
                context_length: 2097152,
                description: "Google's most capable multimodal model for complex reasoning, code, and math.".into(),
                status: ModelStatus::Loaded { vram_usage_mb: 0, ram_usage_mb: 0 },
                downloads_count: 1000000,
                is_featured: true,
                format: ModelFormat::Cloud,
                backend: BackendType::CloudOpenAi,
                local_path: None,
                supports_tools: true,
                supports_vision: true,
                supports_thinking: true,
                author_avatar_url: None,
            },
            ModelInfo {
                id: "gemini/gemini-2.5-flash".into(),
                name: "Gemini 2.5 Flash".into(),
                author: "Google".into(),
                architecture: "gemini".into(),
                quantization: "Cloud API".into(),
                size_gb: 0.0,
                context_length: 1048576,
                description: "Ultra-fast multimodal model balanced for speed, cost, and general intelligence.".into(),
                status: ModelStatus::Loaded { vram_usage_mb: 0, ram_usage_mb: 0 },
                downloads_count: 850000,
                is_featured: true,
                format: ModelFormat::Cloud,
                backend: BackendType::CloudOpenAi,
                local_path: None,
                supports_tools: true,
                supports_vision: true,
                supports_thinking: true,
                author_avatar_url: None,
            },
            ModelInfo {
                id: "gemini/gemini-2.5-flash-lite".into(),
                name: "Gemini 2.5 Flash-Lite".into(),
                author: "Google".into(),
                architecture: "gemini".into(),
                quantization: "Cloud API".into(),
                size_gb: 0.0,
                context_length: 1048576,
                description: "Ultra-lightweight and economical lowest-latency version of the Gemini 2.5 family.".into(),
                status: ModelStatus::Loaded { vram_usage_mb: 0, ram_usage_mb: 0 },
                downloads_count: 450000,
                is_featured: false,
                format: ModelFormat::Cloud,
                backend: BackendType::CloudOpenAi,
                local_path: None,
                supports_tools: true,
                supports_vision: true,
                supports_thinking: false,
                author_avatar_url: None,
            },
            ModelInfo {
                id: "gemini/gemini-1.5-pro".into(),
                name: "Gemini 1.5 Pro".into(),
                author: "Google".into(),
                architecture: "gemini".into(),
                quantization: "Cloud API".into(),
                size_gb: 0.0,
                context_length: 2097152,
                description: "Long context window (2M tokens) model for analyzing large datasets.".into(),
                status: ModelStatus::Loaded { vram_usage_mb: 0, ram_usage_mb: 0 },
                downloads_count: 600000,
                is_featured: false,
                format: ModelFormat::Cloud,
                backend: BackendType::CloudOpenAi,
                local_path: None,
                supports_tools: true,
                supports_vision: true,
                supports_thinking: false,
                author_avatar_url: None,
            },
            ModelInfo {
                id: "gemini/gemini-1.5-flash".into(),
                name: "Gemini 1.5 Flash".into(),
                author: "Google".into(),
                architecture: "gemini".into(),
                quantization: "Cloud API".into(),
                size_gb: 0.0,
                context_length: 1048576,
                description: "Fast, lightweight high-efficiency model from the previous Google Gemini generation.".into(),
                status: ModelStatus::Loaded { vram_usage_mb: 0, ram_usage_mb: 0 },
                downloads_count: 500000,
                is_featured: false,
                format: ModelFormat::Cloud,
                backend: BackendType::CloudOpenAi,
                local_path: None,
                supports_tools: true,
                supports_vision: true,
                supports_thinking: false,
                author_avatar_url: None,
            },
        ]
    }

    /// Discovers models directly from a custom OpenAI-compatible endpoint
    pub async fn scan_custom_direct(base_url: &str, api_key: &str) -> Result<Vec<ModelInfo>, String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(6))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let clean_base = base_url.trim().trim_end_matches('/');
        let candidates = if clean_base.ends_with("/v1") {
            vec![
                format!("{}/models", clean_base),
                clean_base.to_string(),
            ]
        } else {
            vec![
                format!("{}/models", clean_base),
                format!("{}/v1/models", clean_base),
                format!("{}/api/v1/models", clean_base),
            ]
        };

        let mut last_err = String::new();
        for url in candidates {
            let mut req = client.get(&url);
            if !api_key.trim().is_empty() {
                req = req.header("Authorization", format!("Bearer {}", api_key.trim()));
            }

            match req.send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        if let Ok(val) = resp.json::<serde_json::Value>().await {
                            let mut list = Vec::new();
                            if let Some(arr) = val.get("data").and_then(|d| d.as_array()) {
                                for item in arr {
                                    if let Some(id) = item.get("id").and_then(|i| i.as_str()) {
                                        let name = item.get("name").and_then(|n| n.as_str()).unwrap_or(id);
                                        list.push((id.to_string(), name.to_string()));
                                    }
                                }
                            } else if let Some(arr) = val.get("models").and_then(|d| d.as_array()) {
                                for item in arr {
                                    if let Some(id) = item.get("id").and_then(|i| i.as_str()) {
                                        let name = item.get("name").and_then(|n| n.as_str()).unwrap_or(id);
                                        list.push((id.to_string(), name.to_string()));
                                    } else if let Some(id) = item.as_str() {
                                        list.push((id.to_string(), id.to_string()));
                                    }
                                }
                            } else if let Some(arr) = val.as_array() {
                                for item in arr {
                                    if let Some(id) = item.get("id").and_then(|i| i.as_str()) {
                                        let name = item.get("name").and_then(|n| n.as_str()).unwrap_or(id);
                                        list.push((id.to_string(), name.to_string()));
                                    } else if let Some(id) = item.as_str() {
                                        list.push((id.to_string(), id.to_string()));
                                    }
                                }
                            }

                            if !list.is_empty() {
                                let mut models = Vec::new();
                                for (model_id, display_name) in list {
                                    let is_thinking = model_id.contains("r1") || model_id.contains("thinking") || model_id.contains("reasoning");
                                    let is_vision = model_id.contains("vision") || model_id.contains("vl") || model_id.contains("4o");
                                    models.push(ModelInfo {
                                        id: format!("custom/{}", model_id),
                                        name: display_name,
                                        author: "Custom Provider".into(),
                                        architecture: "openai-compatible".into(),
                                        quantization: "Cloud API".into(),
                                        size_gb: 0.0,
                                        context_length: 65536,
                                        description: format!("Model provided by endpoint {}.", clean_base),
                                        status: ModelStatus::Loaded { vram_usage_mb: 0, ram_usage_mb: 0 },
                                        downloads_count: 50000,
                                        is_featured: true,
                                        format: ModelFormat::Cloud,
                                        backend: BackendType::CloudOpenAi,
                                        local_path: Some(clean_base.to_string()),
                                        supports_tools: true,
                                        supports_vision: is_vision,
                                        supports_thinking: is_thinking,
                                        author_avatar_url: None,
                                    });
                                }
                                return Ok(models);
                            }
                        }
                    } else {
                        last_err = format!("Endpoint returned HTTP {}: {}", resp.status(), resp.text().await.unwrap_or_default());
                    }
                }
                Err(e) => {
                    last_err = format!("Error connecting to {}: {}", url, e);
                }
            }
        }

        if last_err.is_empty() {
            Err("No models found at the endpoint.".to_string())
        } else {
            Err(last_err)
        }
    }

    /// Dynamically fetches models directly from the specified provider API
    pub async fn fetch_provider_models_direct(
        provider: &str,
        api_key: Option<&str>,
        base_url: Option<&str>,
    ) -> Result<Vec<ModelInfo>, String> {
        let key = api_key.unwrap_or("").trim();
        match provider {
            "gemini" => {
                let models = Self::scan_gemini(key).await;
                if models.is_empty() {
                    Err("No models returned for Google Gemini.".into())
                } else {
                    Ok(models)
                }
            }
            "openai" => {
                let models = Self::scan_openai(key, base_url).await;
                if models.is_empty() {
                    Err("No models returned by OpenAI API.".into())
                } else {
                    Ok(models)
                }
            }
            "openrouter" => {
                let models = Self::scan_openrouter(key).await;
                if models.is_empty() {
                    Err("No models returned by OpenRouter API.".into())
                } else {
                    Ok(models)
                }
            }
            "groq" => {
                let models = Self::scan_groq(key).await;
                if models.is_empty() {
                    Err("No models returned by Groq API.".into())
                } else {
                    Ok(models)
                }
            }
            "ollama" => {
                let host_str = base_url.unwrap_or("http://127.0.0.1:11434");
                let cleaned = host_str.trim_start_matches("http://").trim_start_matches("https://").trim_end_matches('/');
                let parts: Vec<&str> = cleaned.split(':').collect();
                let h = parts[0];
                let p = if parts.len() > 1 { parts[1].parse::<u16>().unwrap_or(11434) } else { 11434 };
                let models = Self::scan_ollama(h, p).await;
                if models.is_empty() {
                    Err(format!("No models found in Ollama at {}:{}.", h, p))
                } else {
                    Ok(models)
                }
            }
            "custom" => {
                let url = base_url.ok_or_else(|| "Base URL not provided for the custom endpoint.".to_string())?;
                Self::scan_custom_direct(url, key).await
            }
            "antigravity" => {
                let models = Self::scan_antigravity().await;
                Ok(models)
            }
            _ => Err(format!("Unrecognized provider: {}", provider)),
        }
    }

    /// Generates models from a configured CustomCloudProvider
    pub async fn scan_custom_provider(provider: &crate::core::config::CustomCloudProvider) -> Vec<ModelInfo> {
        let provider_name = if provider.name.trim().is_empty() { "Custom OpenAI" } else { provider.name.trim() };
        let model_id = provider.selected_model.as_deref().unwrap_or("default");

        vec![
            ModelInfo {
                id: format!("custom/{}/{}", provider.id, model_id),
                name: format!("{} ({})", provider_name, model_id),
                author: provider_name.into(),
                architecture: "openai-compatible".into(),
                quantization: "Cloud API".into(),
                size_gb: 0.0,
                context_length: 65536,
                description: format!("OpenAI-compatible endpoint pointing to {}.", provider.base_url),
                status: ModelStatus::Loaded { vram_usage_mb: 0, ram_usage_mb: 0 },
                downloads_count: 100000,
                is_featured: true,
                format: ModelFormat::Cloud,
                backend: BackendType::CloudOpenAi,
                local_path: Some(provider.base_url.clone()),
                supports_tools: true,
                supports_vision: true,
                supports_thinking: false,
                author_avatar_url: None,
            }
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scan_antigravity_returns_models() {
        let models = ModelScanner::scan_antigravity().await;
        if crate::services::backend::BackendManager::find_agy_binary().is_some() {
            assert!(!models.is_empty(), "Should discover models from agy");
            let has_gemini_38 = models.iter().any(|m| m.id.contains("gemini-3.8-flash"));
            assert!(has_gemini_38, "Should contain gemini-3.8-flash in returned models");
            for m in &models {
                assert!(m.id.starts_with("agy/"));
                assert!(m.supports_vision);
                assert!(m.supports_tools);
            }
        }
    }

    #[test]
    fn test_is_model_disabled() {
        let disabled = vec![
            "gemini/gemini-2.5-flash-image".to_string(),
            "agy/gemini-3.8-flash".to_string(),
            "gpt-4o-mini".to_string(),
        ];

        // Exact match
        assert!(ModelScanner::is_model_disabled("gemini/gemini-2.5-flash-image", &disabled));
        // Raw id match
        assert!(ModelScanner::is_model_disabled("gemini-2.5-flash-image", &disabled));
        assert!(ModelScanner::is_model_disabled("openai/gpt-4o-mini", &disabled));
        // AGY variant match
        assert!(ModelScanner::is_model_disabled("agy/gemini-3.8-flash-high", &disabled));
        assert!(ModelScanner::is_model_disabled("agy/gemini-3.8-flash-low", &disabled));
        // Non-disabled model
        assert!(!ModelScanner::is_model_disabled("gemini/gemini-2.5-pro", &disabled));
        assert!(!ModelScanner::is_model_disabled("agy/claude-3.7-sonnet-high", &disabled));
    }
}

