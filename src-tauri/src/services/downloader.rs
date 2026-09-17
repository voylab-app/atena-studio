use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tokio::sync::{watch, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HfModelSummary {
    pub id: String,
    pub author: Option<String>,
    pub author_avatar_url: Option<String>,
    pub likes: u64,
    pub downloads: u64,
    pub tags: Vec<String>,
    pub pipeline_tag: Option<String>,
    pub last_modified: Option<String>,
    pub is_private: bool,
    pub is_gated: bool,
    pub is_gguf: bool,
    pub is_mlx: bool,
    pub is_safetensors: bool,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_thinking: bool,
    pub estimated_size_bytes: Option<u64>,
    pub param_size: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HfRepoFile {
    pub path: String,
    pub size_bytes: Option<u64>,
    pub is_gguf: bool,
    pub is_safetensors: bool,
    pub is_config: bool,
    pub quant_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HfModelDetail {
    pub id: String,
    pub author: Option<String>,
    pub author_avatar_url: Option<String>,
    pub description: Option<String>,
    pub readme: Option<String>,
    pub likes: u64,
    pub downloads: u64,
    pub tags: Vec<String>,
    pub files: Vec<HfRepoFile>,
    pub is_mlx: bool,
    pub is_gguf: bool,
    pub is_safetensors: bool,
    pub is_gated: bool,
    pub total_size_bytes: u64,
    pub supports_tools: bool,
    pub supports_vision: bool,
    pub supports_thinking: bool,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTaskProgress {
    pub task_id: String,
    pub repo_id: String,
    pub title: String,
    pub file_name: String,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub progress_pct: f32,
    pub speed_bytes_per_sec: f64,
    pub eta_seconds: Option<u64>,
    pub status: String, // "downloading" | "completed" | "error" | "cancelled"
    pub error_message: Option<String>,
    pub destination_path: String,
    pub started_at: u64,
    pub is_mlx_bundle: bool,
    pub total_files_count: usize,
    pub current_file_index: usize,
    #[serde(default)]
    pub selected_files: Vec<String>,
    #[serde(default)]
    pub target_dir: String,
}

pub struct ModelDownloader {
    client: reqwest::Client,
    download_client: reqwest::Client,
    tasks: Arc<Mutex<HashMap<String, DownloadTaskProgress>>>,
    cancel_senders: Arc<Mutex<HashMap<String, watch::Sender<bool>>>>,
    author_avatar_cache: Arc<Mutex<HashMap<String, Option<String>>>>,
}

impl Default for ModelDownloader {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelDownloader {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("Atena-Studio/0.2.0 (macOS; Apple Silicon)")
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let download_client = reqwest::Client::builder()
            .user_agent("Atena-Studio/0.2.0 (macOS; Apple Silicon)")
            .connect_timeout(Duration::from_secs(30))
            .tcp_keepalive(Duration::from_secs(15))
            .pool_idle_timeout(Duration::from_secs(90))
            // Do NOT set a global request timeout; model downloads can be tens of gigabytes
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            client,
            download_client,
            tasks: Arc::new(Mutex::new(HashMap::new())),
            cancel_senders: Arc::new(Mutex::new(HashMap::new())),
            author_avatar_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Resolves the authentic Hugging Face avatar image URL for a given model author or organization statically
    pub fn get_author_avatar_static(author: &str) -> Option<String> {
        let clean_author = author.trim();
        if clean_author.is_empty() {
            return None;
        }

        let lower = clean_author.to_lowercase();
        match lower.as_str() {
            "google" | "google-deepmind" => Some("https://cdn-avatars.huggingface.co/v1/production/uploads/5dd96eb166059660ed1ee413/WtA3YYitedOr9n02eHfJe.png".to_string()),
            "meta-llama" | "facebook" | "meta" => Some("https://cdn-avatars.huggingface.co/v1/production/uploads/646cf8084eefb026fb8fd8bc/oCTqufkdTkjyGodsx1vo1.png".to_string()),
            "mistralai" | "mistral" => Some("https://cdn-avatars.huggingface.co/v1/production/uploads/634c17653d11eaedd88b314d/9OgyfKstSZtbmsmuG8MbU.png".to_string()),
            "deepseek-ai" | "deepseek" => Some("https://cdn-avatars.huggingface.co/v1/production/uploads/6538815d1bdb3c40db94fbfa/xMBly9PUMphrFVMxLX4kq.png".to_string()),
            "microsoft" => Some("https://cdn-avatars.huggingface.co/v1/production/uploads/1583646260758-5e64858c87403103f9f1055d.png".to_string()),
            "qwen" | "alibaba-nlp" => Some("https://cdn-avatars.huggingface.co/v1/production/uploads/6215ca5692c0ecfba9186921/hrRM50-6XcdWgg2AKpENG.jpeg".to_string()),
            "mlx-community" => Some("https://cdn-avatars.huggingface.co/v1/production/uploads/623c830997ddced06d78699b/3qTjC7d3YFCJTwpxd2noq.png".to_string()),
            "bartowski" => Some("https://cdn-avatars.huggingface.co/v1/production/uploads/6435718aaaef013d1aec3b8b/XKf-8MA47tjVAM6SCX0MP.jpeg".to_string()),
            "unsloth" => Some("https://cdn-avatars.huggingface.co/v1/production/uploads/62ecdc18b72a69615d6bd857/E4lkPz1TZNLzIFr_dR273.png".to_string()),
            "thebloke" => Some("https://cdn-avatars.huggingface.co/v1/production/uploads/6426d3f3a7723d62b53c259b/tvPikpAzKTKGN5wrpadOJ.jpeg".to_string()),
            "nvidia" => Some("https://cdn-avatars.huggingface.co/v1/production/uploads/1628172901614-noauth.png".to_string()),
            "huggingface" => Some("https://huggingface.co/datasets/huggingface/brand-assets/resolve/main/hf-logo.png".to_string()),
            "stabilityai" => Some("https://cdn-avatars.huggingface.co/v1/production/uploads/1672322306894-61019ffea7d8d2acae90a980.jpeg".to_string()),
            "cohere" => Some("https://cdn-avatars.huggingface.co/v1/production/uploads/1647444158498-noauth.png".to_string()),
            "01-ai" => Some("https://cdn-avatars.huggingface.co/v1/production/uploads/6540c1ce8d7455d6f44d18ec/gVp7o14Y70nC3Mv185q1F.jpeg".to_string()),
            "thudm" => Some("https://cdn-avatars.huggingface.co/v1/production/uploads/634b8c9d46928eef604d3062/yA1vO_t9T_R_n352u8oOq.png".to_string()),
            "internlm" => Some("https://cdn-avatars.huggingface.co/v1/production/uploads/647990c749975769188a83dc/f_M21gZ2yJv4X_Q8N6F5N.png".to_string()),
            _ => None,
        }
    }

    /// Checks whether an author is an aggregator or quantizer community rather than the foundational model creator
    pub fn is_community_quantizer(author: &str) -> bool {
        let lower = author.trim().to_lowercase();
        lower == "lmstudio-community"
            || lower == "lmstudio"
            || lower == "lm-studio"
            || lower == "mlx-community"
            || lower == "bartowski"
            || lower == "thebloke"
            || lower == "unsloth"
            || lower == "mradermacher"
            || lower == "city96"
            || lower == "local"
            || lower == "models"
            || lower.contains("lmstudio")
    }

    /// Resolves the authentic model family avatar (e.g. Gemma, Qwen, DeepSeek, Mistral, Llama, Phi)
    pub fn get_model_family_avatar_static(model_id: &str, model_name: &str) -> Option<String> {
        let text = format!("{} {}", model_id, model_name).to_lowercase();
        if text.contains("gemma") {
            Some("https://cdn-avatars.huggingface.co/v1/production/uploads/5dd96eb166059660ed1ee413/WtA3YYitedOr9n02eHfJe.png".to_string())
        } else if text.contains("qwen") {
            Some("https://cdn-avatars.huggingface.co/v1/production/uploads/6215ca5692c0ecfba9186921/hrRM50-6XcdWgg2AKpENG.jpeg".to_string())
        } else if text.contains("deepseek") {
            Some("https://cdn-avatars.huggingface.co/v1/production/uploads/6538815d1bdb3c40db94fbfa/xMBly9PUMphrFVMxLX4kq.png".to_string())
        } else if text.contains("mistral") || text.contains("codestral") || text.contains("pixtral") || text.contains("ministral") || text.contains("mixtral") {
            Some("https://cdn-avatars.huggingface.co/v1/production/uploads/634c17653d11eaedd88b314d/9OgyfKstSZtbmsmuG8MbU.png".to_string())
        } else if text.contains("llama") {
            Some("https://cdn-avatars.huggingface.co/v1/production/uploads/646cf8084eefb026fb8fd8bc/oCTqufkdTkjyGodsx1vo1.png".to_string())
        } else if text.contains("phi") {
            Some("https://cdn-avatars.huggingface.co/v1/production/uploads/1583646260758-5e64858c87403103f9f1055d.png".to_string())
        } else {
            None
        }
    }

    pub fn normalize_avatar_url(url: &str) -> String {
        let trimmed = url.trim();
        if trimmed.starts_with('/') {
            format!("https://huggingface.co{}", trimmed)
        } else {
            trimmed.to_string()
        }
    }

    /// Resolves authentic Hugging Face avatar image URLs for multiple model authors concurrently
    pub async fn get_author_avatars(&self, authors: Vec<String>) -> std::collections::HashMap<String, Option<String>> {
        use std::collections::HashSet;
        let mut unique_authors = Vec::new();
        let mut seen = HashSet::new();

        for a in authors {
            let clean = a.trim().to_string();
            if !clean.is_empty() && seen.insert(clean.to_lowercase()) {
                unique_authors.push(clean);
            }
        }

        let tasks: Vec<_> = unique_authors
            .into_iter()
            .map(|author| async move {
                let avatar = self.get_author_avatar(&author).await;
                (author, avatar)
            })
            .collect();

        let results = futures_util::future::join_all(tasks).await;
        results.into_iter().collect()
    }

    /// Resolves the authentic Hugging Face avatar image URL for a given model author or organization
    pub async fn get_author_avatar(&self, author: &str) -> Option<String> {
        let clean_author = author.trim();
        if clean_author.is_empty() {
            return None;
        }

        // 1. Check instant static map
        if let Some(s) = Self::get_author_avatar_static(clean_author) {
            return Some(s);
        }

        let lower = clean_author.to_lowercase();

        // 2. Check in-memory cache
        {
            let cache = self.author_avatar_cache.lock().await;
            if let Some(cached) = cache.get(&lower) {
                return cached.clone();
            }
        }

        // 3. Query Hugging Face organization overview
        let org_url = format!("https://huggingface.co/api/organizations/{}/overview", clean_author);
        if let Ok(resp) = self.client.get(&org_url).send().await {
            if resp.status().is_success() {
                if let Ok(json) = resp.json::<Value>().await {
                    if let Some(avatar) = json.get("avatarUrl").and_then(|v| v.as_str()) {
                        let res = Some(Self::normalize_avatar_url(avatar));
                        let mut cache = self.author_avatar_cache.lock().await;
                        cache.insert(lower.clone(), res.clone());
                        return res;
                    }
                }
            }
        }

        // 4. Query Hugging Face user overview
        let user_url = format!("https://huggingface.co/api/users/{}/overview", clean_author);
        if let Ok(resp) = self.client.get(&user_url).send().await {
            if resp.status().is_success() {
                if let Ok(json) = resp.json::<Value>().await {
                    if let Some(avatar) = json.get("avatarUrl").and_then(|v| v.as_str()) {
                        let res = Some(Self::normalize_avatar_url(avatar));
                        let mut cache = self.author_avatar_cache.lock().await;
                        cache.insert(lower.clone(), res.clone());
                        return res;
                    }
                }
            }
        }

        let mut cache = self.author_avatar_cache.lock().await;
        cache.insert(lower, None);
        None
    }

    /// Searches Hugging Face models using their public API
    async fn fetch_hf_models_raw(
        client: &reqwest::Client,
        query: &str,
        filter_arg: Option<&str>,
        sort_val: &str,
        limit: usize,
    ) -> Result<Vec<Value>, String> {
        let mut url = String::from("https://huggingface.co/api/models?");
        let trimmed_query = query.trim();
        if !trimmed_query.is_empty() {
            let encoded = urlencoding_simple(trimmed_query);
            url.push_str(&format!("search={}&", encoded));
        }
        url.push_str(&format!("sort={}&direction=-1&", sort_val));
        if let Some(f) = filter_arg {
            url.push_str(&format!("filter={}&", f));
        }
        url.push_str(&format!(
            "limit={}&full=false&expand[]=downloads&expand[]=likes&expand[]=safetensors&expand[]=gguf&expand[]=lastModified&expand[]=author&expand[]=tags&expand[]=pipeline_tag&expand[]=siblings&",
            limit.min(100)
        ));

        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Falha ao conectar com Hugging Face API: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Erro retornado pelo Hugging Face: HTTP {}", resp.status()));
        }

        resp.json::<Vec<Value>>()
            .await
            .map_err(|e| format!("Falha ao processar resposta JSON do Hugging Face: {}", e))
    }

    /// Searches Hugging Face models using their public API (strictly GGUF & MLX compatible, no gated)
    pub async fn search_hf_models(
        &self,
        query: &str,
        filter_type: Option<&str>,
        sort: Option<&str>,
        limit: usize,
    ) -> Result<Vec<HfModelSummary>, String> {
        let sort_val = match sort {
            Some("downloads") => "downloads",
            Some("likes") => "likes",
            Some("lastModified") => "lastModified",
            _ => "trendingScore",
        };

        let items: Vec<Value> = match filter_type {
            Some("gguf") => {
                Self::fetch_hf_models_raw(&self.client, query, Some("gguf"), sort_val, limit).await?
            }
            Some("mlx") => {
                Self::fetch_hf_models_raw(&self.client, query, Some("mlx"), sort_val, limit).await?
            }
            _ => {
                // "all" or None: Fetch both GGUF and MLX concurrently to only ever return Atena-runnable models!
                let (gguf_res, mlx_res) = tokio::join!(
                    Self::fetch_hf_models_raw(&self.client, query, Some("gguf"), sort_val, limit),
                    Self::fetch_hf_models_raw(&self.client, query, Some("mlx"), sort_val, limit),
                );

                let mut combined = gguf_res.unwrap_or_default();
                let mut seen_ids: std::collections::HashSet<String> = combined
                    .iter()
                    .filter_map(|v| v.get("id").and_then(|i| i.as_str()).map(|s| s.to_string()))
                    .collect();

                for item in mlx_res.unwrap_or_default() {
                    if let Some(id) = item.get("id").and_then(|i| i.as_str()) {
                        if !seen_ids.contains(id) {
                            seen_ids.insert(id.to_string());
                            combined.push(item);
                        }
                    }
                }
                combined
            }
        };

        let mut results = Vec::new();
        for item in items {
            let id = match item.get("id").and_then(|v| v.as_str()) {
                Some(id) => id.to_string(),
                None => continue,
            };

            let is_gated = item.get("gated").and_then(|v| v.as_bool()).or_else(|| {
                item.get("gated").and_then(|v| v.as_str()).map(|s| s != "false")
            }).unwrap_or(false);

            // Filter out gated models (cannot be downloaded anonymously; require signing terms on HF)
            if is_gated {
                continue;
            }

            let author = item.get("author").and_then(|v| v.as_str()).map(|s| s.to_string())
                .or_else(|| id.split('/').next().map(|s| s.to_string()));

            let likes = item.get("likes").and_then(|v| v.as_u64()).unwrap_or(0);
            let downloads = item.get("downloads").and_then(|v| v.as_u64())
                .or_else(|| item.get("downloadsAllTime").and_then(|v| v.as_u64()))
                .unwrap_or(0);
            
            let tags: Vec<String> = item.get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|t| t.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();

            let pipeline_tag = item.get("pipeline_tag").and_then(|v| v.as_str()).map(|s| s.to_string());
            let last_modified = item.get("lastModified").and_then(|v| v.as_str()).map(|s| s.to_string());
            let is_private = item.get("private").and_then(|v| v.as_bool()).unwrap_or(false);

            let id_lower = id.to_lowercase();

            let siblings_gguf_files: Vec<&str> = item.get("siblings")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|s| s.get("rfilename").and_then(|f| f.as_str()))
                        .filter(|f| f.to_lowercase().ends_with(".gguf"))
                        .collect()
                })
                .unwrap_or_default();

            let is_gguf = tags.iter().any(|t| t.to_lowercase() == "gguf") || id_lower.contains("gguf") || !siblings_gguf_files.is_empty();
            let is_mlx = tags.iter().any(|t| t.to_lowercase() == "mlx") || id_lower.contains("mlx");
            let is_safetensors = item.get("safetensors").is_some() || tags.iter().any(|t| t.to_lowercase() == "safetensors") || id_lower.contains("safetensors");

            // Filter out models that cannot run in Atena Studio (must be GGUF or MLX)
            if !is_gguf && !is_mlx {
                continue;
            }

            let tags_str = tags.join(" ");
            let (supports_tools, supports_vision, supports_thinking) =
                crate::services::scanner::ModelScanner::detect_capabilities(
                    &id,
                    &author.clone().unwrap_or_default(),
                    &pipeline_tag.clone().unwrap_or_default(),
                    &tags_str,
                );

            let mut estimated_size_bytes: Option<u64> = None;
            let mut param_size: Option<String> = None;

            let siblings_gguf_files: Vec<&str> = item.get("siblings")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|s| s.get("rfilename").and_then(|f| f.as_str()))
                        .filter(|f| f.to_lowercase().ends_with(".gguf"))
                        .collect()
                })
                .unwrap_or_default();

            // 1. Check GGUF metadata returned by Hugging Face API
            if let Some(gguf_val) = item.get("gguf") {
                if let Some(total_params) = gguf_val.get("total").and_then(|v| v.as_u64()) {
                    if total_params > 0 {
                        param_size = Some(format_param_count(total_params));
                    }
                }

                // If the repo contains only 1 primary GGUF model file (or 1 model + 1 mmproj file),
                // gguf.totalFileSize is NOT the sum of dozens of quants; it is the exact download size of this model.
                let non_mmproj_count = siblings_gguf_files
                    .iter()
                    .filter(|f| !f.to_lowercase().contains("mmproj"))
                    .count();

                if non_mmproj_count == 1 {
                    if let Some(total_file_size) = gguf_val.get("totalFileSize").and_then(|v| v.as_u64()) {
                        if total_file_size > 0 {
                            estimated_size_bytes = Some(total_file_size);
                        }
                    }
                }

                // If estimated_size_bytes is still None (e.g. repo contains multiple quants like Q4_K_M, Q8_0, etc.,
                // where totalFileSize represents the sum of ALL quants ~80GB), we estimate the download size of a
                // single typical runnable quant (Q4_K_M ~ 0.58 bytes/param).
                if estimated_size_bytes.is_none() {
                    if let Some(total_params) = gguf_val.get("total").and_then(|v| v.as_u64()) {
                        if total_params > 0 {
                            estimated_size_bytes = Some((total_params as f64 * 0.58) as u64);
                        }
                    }
                }
            }

            // 2. Check Safetensors metadata (frequent in MLX and standard PyTorch/Safetensors repos)
            if let Some(st_val) = item.get("safetensors") {
                if let Some(total_params) = st_val.get("total").and_then(|v| v.as_u64()) {
                    if total_params > 0 {
                        if param_size.is_none() {
                            param_size = Some(format_param_count(total_params));
                        }

                        if estimated_size_bytes.is_none() {
                            let is_4bit = id_lower.contains("4bit") || id_lower.contains("4-bit") || id_lower.contains("q4") || id_lower.contains("qat") || id_lower.contains("nvfp4");
                            let is_8bit = id_lower.contains("8bit") || id_lower.contains("8-bit") || id_lower.contains("q8");
                            let bytes_per_param = if is_4bit { 0.58 } else if is_8bit { 1.05 } else { 2.0 };
                            estimated_size_bytes = Some((total_params as f64 * bytes_per_param) as u64);
                        }
                    }
                }
            }

            // 3. Fallback: extract parameter size (e.g., 12B, 7B, 3B) directly from repository name
            if param_size.is_none() {
                if let Some(p) = extract_param_size(&id) {
                    param_size = Some(p.clone());
                    if estimated_size_bytes.is_none() {
                        let num_str = &p[..p.len() - 1];
                        if let Ok(num) = num_str.parse::<f64>() {
                            let total_params = num * 1_000_000_000.0;
                            let is_4bit = id_lower.contains("4bit") || id_lower.contains("4-bit") || id_lower.contains("q4") || id_lower.contains("qat") || id_lower.contains("nvfp4");
                            let is_8bit = id_lower.contains("8bit") || id_lower.contains("8-bit") || id_lower.contains("q8");
                            let bytes_per_param = if is_4bit { 0.58 } else if is_8bit { 1.05 } else { 2.0 };
                            estimated_size_bytes = Some((total_params * bytes_per_param) as u64);
                        }
                    }
                }
            }

            let author_avatar_url = author.as_deref().and_then(Self::get_author_avatar_static);

            results.push(HfModelSummary {
                id,
                author,
                author_avatar_url,
                likes,
                downloads,
                tags,
                pipeline_tag,
                last_modified,
                is_private,
                is_gated,
                is_gguf,
                is_mlx,
                is_safetensors,
                supports_tools,
                supports_vision,
                supports_thinking,
                estimated_size_bytes,
                param_size,
            });
        }

        if filter_type.is_none() || filter_type == Some("all") {
            match sort_val {
                "downloads" => results.sort_by(|a, b| b.downloads.cmp(&a.downloads)),
                "likes" => results.sort_by(|a, b| b.likes.cmp(&a.likes)),
                "lastModified" => results.sort_by(|a, b| b.last_modified.cmp(&a.last_modified)),
                _ => {} // Preserves trending ranking
            }
        }

        Ok(results)
    }

    /// Fetches model details and file list from Hugging Face
    pub async fn get_model_details(&self, repo_id: &str) -> Result<HfModelDetail, String> {
        let clean_repo = repo_id.trim().trim_matches('/');
        let info_url = format!("https://huggingface.co/api/models/{}", clean_repo);
        let tree_url = format!("https://huggingface.co/api/models/{}/tree/main?recursive=true", clean_repo);

        let info_resp = self
            .client
            .get(&info_url)
            .send()
            .await
            .map_err(|e| format!("Erro ao obter informações do modelo: {}", e))?;

        if !info_resp.status().is_success() {
            return Err(format!("Modelo não encontrado ou erro no Hugging Face: HTTP {}", info_resp.status()));
        }

        let info_json: Value = info_resp
            .json()
            .await
            .map_err(|e| format!("Erro ao decodificar JSON do modelo: {}", e))?;

        let author = info_json.get("author").and_then(|v| v.as_str()).map(|s| s.to_string())
            .or_else(|| clean_repo.split('/').next().map(|s| s.to_string()));

        let description = info_json.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
        let likes = info_json.get("likes").and_then(|v| v.as_u64()).unwrap_or(0);
        let downloads = info_json.get("downloads").and_then(|v| v.as_u64()).unwrap_or(0);

        let tags: Vec<String> = info_json.get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|t| t.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        let repo_lower = clean_repo.to_lowercase();

        // Try getting file list from /tree/main API for exact sizes
        let mut files: Vec<HfRepoFile> = Vec::new();
        let mut total_size_bytes = 0u64;

        // Build a size map from the tree API (root-level files with exact sizes)
        let mut tree_sizes: HashMap<String, u64> = HashMap::new();
        if let Ok(tree_resp) = self.client.get(&tree_url).send().await {
            if tree_resp.status().is_success() {
                if let Ok(tree_json) = tree_resp.json::<Vec<Value>>().await {
                    for entry in tree_json {
                        if let (Some(path), Some(size)) = (
                            entry.get("path").and_then(|v| v.as_str()),
                            entry.get("size").and_then(|v| v.as_u64()),
                        ) {
                            tree_sizes.insert(path.to_string(), size);
                        }
                    }
                }
            }
        }

        // Always use siblings from model info API as the primary source (complete file list)
        // Enrich with exact sizes from tree API where available
        if let Some(siblings) = info_json.get("siblings").and_then(|v| v.as_array()) {
            for s in siblings {
                let rfilename = match s.get("rfilename").and_then(|v| v.as_str()) {
                    Some(p) => p.to_string(),
                    None => continue,
                };

                let size = tree_sizes.remove(&rfilename);
                if let Some(s) = size {
                    total_size_bytes += s;
                }

                let path_lower = rfilename.to_lowercase();
                let is_gguf_file = path_lower.ends_with(".gguf");
                let is_safetensors = path_lower.ends_with(".safetensors");
                let is_config = path_lower == "config.json" || path_lower == "tokenizer.json" || path_lower == "tokenizer_config.json";
                let quant_type = Self::detect_quant_type(&rfilename);

                files.push(HfRepoFile {
                    path: rfilename,
                    size_bytes: size,
                    is_gguf: is_gguf_file,
                    is_safetensors,
                    is_config,
                    quant_type,
                });
            }
        }

        // Fallback: if siblings was absent/empty, use tree API entries directly
        if files.is_empty() {
            for (path, size) in &tree_sizes {
                total_size_bytes += size;
                let path_lower = path.to_lowercase();
                let is_gguf_file = path_lower.ends_with(".gguf");
                let is_safetensors = path_lower.ends_with(".safetensors");
                let is_config = path_lower == "config.json" || path_lower == "tokenizer.json" || path_lower == "tokenizer_config.json";
                let quant_type = Self::detect_quant_type(path);

                files.push(HfRepoFile {
                    path: path.clone(),
                    size_bytes: Some(*size),
                    is_gguf: is_gguf_file,
                    is_safetensors,
                    is_config,
                    quant_type,
                });
            }
        }

        // Filter out deprecated / removed GGUF quantization formats (e.g. Q4_0_4_4, Q4_0_4_8, Q4_0_8_8)
        files.retain(|f| {
            let p = f.path.to_uppercase();
            !p.contains("Q4_0_4_4") && !p.contains("Q4_0_4_8") && !p.contains("Q4_0_8_8")
        });

        // Sort files: GGUF and safetensors first
        files.sort_by(|a, b| {
            let a_score = if a.is_gguf { 0 } else if a.is_safetensors { 1 } else { 2 };
            let b_score = if b.is_gguf { 0 } else if b.is_safetensors { 1 } else { 2 };
            a_score.cmp(&b_score).then_with(|| a.path.cmp(&b.path))
        });

        let tags_str = tags.join(" ");
        let (supports_tools, supports_vision, supports_thinking) =
            crate::services::scanner::ModelScanner::detect_capabilities(
                &clean_repo,
                &author.clone().unwrap_or_default(),
                &description.clone().unwrap_or_default(),
                &tags_str,
            );

        let author_avatar_url = match author.as_deref() {
            Some(a) => self.get_author_avatar(a).await,
            None => None,
        };

        let is_gated = info_json.get("gated").and_then(|v| v.as_bool()).or_else(|| {
            info_json.get("gated").and_then(|v| v.as_str()).map(|s| s != "false")
        }).unwrap_or(false);

        let has_gguf_files = files.iter().any(|f| f.is_gguf);
        let is_gguf = has_gguf_files;
        let is_mlx = tags.iter().any(|t| t.to_lowercase() == "mlx") || repo_lower.contains("mlx");
        let has_safetensors_files = files.iter().any(|f| f.is_safetensors);
        let is_safetensors = has_safetensors_files || tags.iter().any(|t| t.to_lowercase() == "safetensors");

        // Fetch model README markdown
        let readme_url = format!("https://huggingface.co/{}/raw/main/README.md", clean_repo);
        let mut readme: Option<String> = None;
        if let Ok(readme_resp) = self.client.get(&readme_url).send().await {
            let status = readme_resp.status();
            if status.is_success() {
                if let Ok(text) = readme_resp.text().await {
                    if !text.trim().is_empty() {
                        readme = Some(text);
                    }
                }
            } else if status.as_u16() == 401 && is_gated {
                readme = Some(format!(
                    "### 🔒 Repositório com Acesso Restrito (Gated Model)\n\nO repositório `{}` exige que você aceite os termos de uso oficiais no Hugging Face para acessar arquivos e documentação.\n\nClique no botão **\"Ver no Hugging Face\"** acima para aceitar os termos e desbloquear o acesso.",
                    clean_repo
                ));
            }
        }

        Ok(HfModelDetail {
            id: clean_repo.to_string(),
            author,
            author_avatar_url,
            description,
            readme,
            likes,
            downloads,
            tags,
            files,
            is_mlx,
            is_gguf,
            is_safetensors,
            is_gated,
            total_size_bytes,
            supports_tools,
            supports_vision,
            supports_thinking,
        })
    }


    fn detect_quant_type(path: &str) -> Option<String> {
        let p_upper = path.to_uppercase();
        let quants = [
            "Q4_K_M", "Q4_K_S", "Q4_K", "Q4_0", "Q4_1",
            "Q5_K_M", "Q5_K_S", "Q5_K", "Q5_0", "Q5_1",
            "Q8_0", "Q8_K", "Q8_1",
            "Q6_K", "Q3_K_M", "Q3_K_L", "Q3_K_S", "Q2_K",
            "IQ4_XS", "IQ4_NL", "IQ3_M", "IQ3_S", "IQ2_M",
            "FP16", "BF16", "4BIT", "8BIT", "3BIT", "2BIT"
        ];

        for q in quants {
            if p_upper.contains(q) {
                return Some(q.to_string());
            }
        }
        None
    }

    /// Initiates a download for either a single file (like GGUF) or a full repository (like MLX bundle)
    pub async fn start_download(
        &self,
        repo_id: String,
        selected_files: Vec<String>,
        target_dir: String,
        is_mlx_bundle: bool,
        model_title: Option<String>,
        author_avatar_url: Option<String>,
    ) -> Result<String, String> {
        if selected_files.is_empty() {
            return Err("Nenhum arquivo selecionado para download".to_string());
        }

        for file in &selected_files {
            let u = file.to_uppercase();
            if u.contains("Q4_0_4_4") || u.contains("Q4_0_4_8") || u.contains("Q4_0_8_8") {
                return Err("Esta quantização (Q4_0_4_4 / Q4_0_X_X) foi descontinuada e removida do llama.cpp. Por favor, selecione formatos padrão como Q4_K_M, Q5_K_M ou Q8_0.".to_string());
            }
        }

        let clean_repo = repo_id.trim().trim_matches('/').to_string();
        let task_id = format!("dl-{}-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(), rand_suffix());
        
        let resolved_target_dir = crate::core::config::resolve_path(&target_dir);
        let destination_path = if is_mlx_bundle {
            let folder_name = clean_repo.replace('/', "___");
            resolved_target_dir.join(&folder_name).to_string_lossy().to_string()
        } else {
            let first_file = &selected_files[0];
            let fname = Path::new(first_file).file_name().unwrap_or_default().to_string_lossy().to_string();
            resolved_target_dir.join(&fname).to_string_lossy().to_string()
        };

        // Proactively download and save model logo into folder for instant offline loading
        let author_name = clean_repo.split('/').next().unwrap_or(&clean_repo).to_string();
        let target_logo_file = if is_mlx_bundle {
            std::path::PathBuf::from(&destination_path).join("logo.png")
        } else {
            let fname = std::path::Path::new(&selected_files[0]).file_stem().unwrap_or_default().to_string_lossy().to_string();
            resolved_target_dir.join(format!("{}.logo.png", fname))
        };

        let client_clone = self.client.clone();
        let avatar_url_opt = author_avatar_url.clone();
        let author_clone = author_name.clone();
        let cache_ref = self.author_avatar_cache.clone();

        tokio::spawn(async move {
            let mut url_to_fetch = avatar_url_opt;
            if url_to_fetch.is_none() {
                if let Some(s) = Self::get_author_avatar_static(&author_clone) {
                    url_to_fetch = Some(s);
                } else {
                    let cache = cache_ref.lock().await;
                    url_to_fetch = cache.get(&author_clone.to_lowercase()).cloned().flatten();
                }
            }

            if let Some(avatar_url) = url_to_fetch {
                if let Ok(resp) = client_clone.get(&avatar_url).send().await {
                    if resp.status().is_success() {
                        if let Ok(bytes) = resp.bytes().await {
                            if let Some(parent) = target_logo_file.parent() {
                                let _ = tokio::fs::create_dir_all(parent).await;
                            }
                            let _ = tokio::fs::write(&target_logo_file, &bytes).await;
                        }
                    }
                }
            }
        });

        let title = match model_title {
            Some(t) if !t.trim().is_empty() => t.trim().to_string(),
            _ => {
                let repo_name = clean_repo.split('/').last().unwrap_or(&clean_repo);
                crate::services::scanner::ModelScanner::format_model_name(repo_name)
            }
        };
        let file_name = if selected_files.len() == 1 {
            selected_files[0].clone()
        } else if is_mlx_bundle {
            format!("{} arquivos (Bundle MLX)", selected_files.len())
        } else {
            format!("{} arquivos (Modelo + Projetor Visão)", selected_files.len())
        };

        let (cancel_tx, cancel_rx) = watch::channel(false);

        let initial_progress = DownloadTaskProgress {
            task_id: task_id.clone(),
            repo_id: clean_repo.clone(),
            title,
            file_name,
            total_bytes: 0,
            downloaded_bytes: 0,
            progress_pct: 0.0,
            speed_bytes_per_sec: 0.0,
            eta_seconds: None,
            status: "downloading".to_string(),
            error_message: None,
            destination_path: destination_path.clone(),
            started_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            is_mlx_bundle,
            total_files_count: selected_files.len(),
            current_file_index: 0,
            selected_files: selected_files.clone(),
            target_dir: target_dir.clone(),
        };

        {
            let mut tasks = self.tasks.lock().await;
            tasks.insert(task_id.clone(), initial_progress);
            let mut cancels = self.cancel_senders.lock().await;
            cancels.insert(task_id.clone(), cancel_tx);
        }

        // Spawn async download worker using the dedicated download_client without request timeouts
        let client = self.download_client.clone();
        let tasks_store = self.tasks.clone();
        let task_id_clone = task_id.clone();
        let repo_id_clone = clean_repo.clone();
        let target_dir_path = resolved_target_dir;

        tokio::spawn(async move {
            let result = Self::download_worker(
                client,
                tasks_store.clone(),
                task_id_clone.clone(),
                repo_id_clone,
                selected_files,
                target_dir_path,
                is_mlx_bundle,
                cancel_rx,
            )
            .await;

            let mut tasks = tasks_store.lock().await;
            if let Some(task) = tasks.get_mut(&task_id_clone) {
                match result {
                    Ok(_) => {
                        task.status = "completed".to_string();
                        task.progress_pct = 100.0;
                        task.speed_bytes_per_sec = 0.0;
                        task.eta_seconds = Some(0);
                    }
                    Err(e) => {
                        if task.status != "cancelled" {
                            task.status = "error".to_string();
                            task.error_message = Some(e);
                            task.speed_bytes_per_sec = 0.0;
                        }
                    }
                }
            }
        });

        Ok(task_id)
    }

    /// Retries or resumes an existing download task from where it left off
    pub async fn retry_download(&self, task_id: &str) -> Result<(), String> {
        let (repo_id, selected_files, is_mlx_bundle, target_dir) = {
            let mut tasks = self.tasks.lock().await;
            let task = tasks.get_mut(task_id).ok_or_else(|| "Tarefa não encontrada".to_string())?;
            if task.status == "downloading" {
                return Err("Download já está em andamento".to_string());
            }
            task.status = "downloading".to_string();
            task.error_message = None;
            task.speed_bytes_per_sec = 0.0;

            let files = if task.selected_files.is_empty() {
                let fname = Path::new(&task.destination_path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                if fname.is_empty() {
                    return Err("Nenhum arquivo associado à tarefa".to_string());
                }
                vec![fname]
            } else {
                task.selected_files.clone()
            };

            let t_dir = if task.target_dir.is_empty() {
                Path::new(&task.destination_path)
                    .parent()
                    .unwrap_or(Path::new("."))
                    .to_string_lossy()
                    .to_string()
            } else {
                task.target_dir.clone()
            };

            (task.repo_id.clone(), files, task.is_mlx_bundle, t_dir)
        };

        let (cancel_tx, cancel_rx) = watch::channel(false);
        {
            let mut cancels = self.cancel_senders.lock().await;
            cancels.insert(task_id.to_string(), cancel_tx);
        }

        let client = self.download_client.clone();
        let tasks_store = self.tasks.clone();
        let task_id_clone = task_id.to_string();
        let target_dir_path = crate::core::config::resolve_path(&target_dir);

        tokio::spawn(async move {
            let result = Self::download_worker(
                client,
                tasks_store.clone(),
                task_id_clone.clone(),
                repo_id,
                selected_files,
                target_dir_path,
                is_mlx_bundle,
                cancel_rx,
            )
            .await;

            let mut tasks = tasks_store.lock().await;
            if let Some(task) = tasks.get_mut(&task_id_clone) {
                match result {
                    Ok(_) => {
                        task.status = "completed".to_string();
                        task.progress_pct = 100.0;
                        task.speed_bytes_per_sec = 0.0;
                        task.eta_seconds = Some(0);
                    }
                    Err(e) => {
                        if task.status != "cancelled" {
                            task.status = "error".to_string();
                            task.error_message = Some(e);
                            task.speed_bytes_per_sec = 0.0;
                        }
                    }
                }
            }
        });

        Ok(())
    }

    async fn download_worker(
        client: reqwest::Client,
        tasks_store: Arc<Mutex<HashMap<String, DownloadTaskProgress>>>,
        task_id: String,
        repo_id: String,
        files: Vec<String>,
        target_dir: PathBuf,
        is_mlx_bundle: bool,
        mut cancel_rx: watch::Receiver<bool>,
    ) -> Result<(), String> {
        let base_dest = if is_mlx_bundle {
            let sanitized = repo_id.replace('/', "___");
            let p = target_dir.join(&sanitized);
            fs::create_dir_all(&p).await.map_err(|e| format!("Erro ao criar pasta destino: {}", e))?;
            p
        } else {
            fs::create_dir_all(&target_dir).await.map_err(|e| format!("Erro ao criar pasta destino: {}", e))?;
            target_dir.clone()
        };

        // Fetch exact file sizes from Hugging Face tree API for all files
        let mut file_sizes: HashMap<String, u64> = HashMap::new();
        let mut overall_total_bytes = 0u64;

        let tree_url = format!("https://huggingface.co/api/models/{}/tree/main", repo_id);
        if let Ok(tree_resp) = client.get(&tree_url).send().await {
            if tree_resp.status().is_success() {
                if let Ok(tree_json) = tree_resp.json::<Vec<Value>>().await {
                    for entry in tree_json {
                        if let (Some(path), Some(size)) = (
                            entry.get("path").and_then(|v| v.as_str()),
                            entry.get("size").and_then(|v| v.as_u64()),
                        ) {
                            if files.iter().any(|f| f == path) {
                                file_sizes.insert(path.to_string(), size);
                                overall_total_bytes += size;
                            }
                        }
                    }
                }
            }
        }

        // If tree API didn't return sizes, try Range request for each missing file
        for f in &files {
            if !file_sizes.contains_key(f) || file_sizes[f] == 0 {
                let file_url = format!("https://huggingface.co/{}/resolve/main/{}", repo_id, f);
                if let Ok(resp) = client.get(&file_url).header("Range", "bytes=0-0").send().await {
                    if let Some(cr) = resp.headers().get("content-range").and_then(|h| h.to_str().ok()) {
                        // Content-Range: bytes 0-0/12345678
                        if let Some(total_str) = cr.split('/').last() {
                            if let Ok(total_val) = total_str.trim().parse::<u64>() {
                                file_sizes.insert(f.clone(), total_val);
                                overall_total_bytes += total_val;
                            }
                        }
                    } else if let Some(cl) = resp.content_length() {
                        file_sizes.insert(f.clone(), cl);
                        overall_total_bytes += cl;
                    }
                }
            }
        }

        // Calculate initially downloaded bytes across all files (completed files + existing .part files)
        let mut overall_downloaded_bytes = 0u64;
        for f in &files {
            let final_p = base_dest.join(f);
            let part_p = base_dest.join(format!("{}.part", f));
            if final_p.exists() {
                if let Ok(m) = fs::metadata(&final_p).await {
                    overall_downloaded_bytes += m.len();
                }
            } else if part_p.exists() {
                if let Ok(m) = fs::metadata(&part_p).await {
                    overall_downloaded_bytes += m.len();
                }
            }
        }

        {
            let mut tasks = tasks_store.lock().await;
            if let Some(t) = tasks.get_mut(&task_id) {
                t.total_bytes = overall_total_bytes;
                t.downloaded_bytes = overall_downloaded_bytes;
                if overall_total_bytes > 0 {
                    t.progress_pct = ((overall_downloaded_bytes as f64 / overall_total_bytes as f64) * 100.0) as f32;
                }
            }
        }

        let mut last_instant = Instant::now();
        let mut bytes_since_last_calc = 0u64;

        for (idx, file_name) in files.iter().enumerate() {
            // Check cancellation
            if *cancel_rx.borrow() {
                let mut tasks = tasks_store.lock().await;
                if let Some(t) = tasks.get_mut(&task_id) {
                    t.status = "cancelled".to_string();
                }
                return Err("Download cancelado pelo usuário".to_string());
            }

            {
                let mut tasks = tasks_store.lock().await;
                if let Some(t) = tasks.get_mut(&task_id) {
                    t.current_file_index = idx + 1;
                    if is_mlx_bundle {
                        t.file_name = format!("({}/{}) {}", idx + 1, files.len(), file_name);
                    }
                }
            }

            let file_url = format!("https://huggingface.co/{}/resolve/main/{}", repo_id, file_name);
            let final_file_path = base_dest.join(file_name);
            
            // Create subdirectories if file has a relative path (e.g. tokenizer/...)
            if let Some(parent) = final_file_path.parent() {
                fs::create_dir_all(parent).await.map_err(|e| format!("Erro ao criar subpasta: {}", e))?;
            }

            let temp_part_path = base_dest.join(format!("{}.part", file_name));
            let expected_file_size = file_sizes.get(file_name).copied().unwrap_or(0);

            // Skip file if already fully downloaded and renamed
            if final_file_path.exists() {
                if let Ok(m) = fs::metadata(&final_file_path).await {
                    if expected_file_size == 0 || m.len() == expected_file_size {
                        continue;
                    }
                }
            }

            // Retry loop with HTTP Range resuming for this specific file
            let mut retry_count = 0;
            const MAX_RETRIES: usize = 6;

            loop {
                if *cancel_rx.borrow() {
                    let _ = fs::remove_file(&temp_part_path).await;
                    let mut tasks = tasks_store.lock().await;
                    if let Some(t) = tasks.get_mut(&task_id) {
                        t.status = "cancelled".to_string();
                    }
                    return Err("Download cancelado pelo usuário".to_string());
                }

                let current_part_len = if temp_part_path.exists() {
                    fs::metadata(&temp_part_path).await.map(|m| m.len()).unwrap_or(0)
                } else {
                    0
                };

                if expected_file_size > 0 && current_part_len >= expected_file_size {
                    // File is already completely saved in the .part file
                    break;
                }

                let mut req = client.get(&file_url);
                if current_part_len > 0 {
                    req = req.header("Range", format!("bytes={}-", current_part_len));
                }

                let response = match req.send().await {
                    Ok(r) => r,
                    Err(e) => {
                        retry_count += 1;
                        if retry_count > MAX_RETRIES {
                            return Err(format!("Erro ao conectar para baixar {}: {}", file_name, e));
                        }
                        tokio::time::sleep(Duration::from_secs(2 * retry_count as u64)).await;
                        continue;
                    }
                };

                let status = response.status();
                if status == reqwest::StatusCode::RANGE_NOT_SATISFIABLE {
                    // Range beyond file length; check if file is complete
                    break;
                }

                if !status.is_success() {
                    retry_count += 1;
                    if retry_count > MAX_RETRIES {
                        return Err(format!("Erro ao baixar {}: HTTP {}", file_name, status));
                    }
                    tokio::time::sleep(Duration::from_secs(2 * retry_count as u64)).await;
                    continue;
                }

                let is_partial = status == reqwest::StatusCode::PARTIAL_CONTENT;

                let mut out_file = if is_partial && current_part_len > 0 {
                    tokio::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&temp_part_path)
                        .await
                        .map_err(|e| format!("Erro ao abrir arquivo temporário para continuar: {}", e))?
                } else {
                    if current_part_len > 0 {
                        overall_downloaded_bytes = overall_downloaded_bytes.saturating_sub(current_part_len);
                    }
                    File::create(&temp_part_path)
                        .await
                        .map_err(|e| format!("Erro ao criar arquivo temporário {:?}: {}", temp_part_path, e))?
                };

                let mut stream = response.bytes_stream();
                let mut stream_interrupted = false;

                loop {
                    let chunk_step = tokio::select! {
                        _ = cancel_rx.changed() => {
                            if *cancel_rx.borrow() {
                                let _ = fs::remove_file(&temp_part_path).await;
                                let mut tasks = tasks_store.lock().await;
                                if let Some(t) = tasks.get_mut(&task_id) {
                                    t.status = "cancelled".to_string();
                                }
                                return Err("Download cancelado pelo usuário".to_string());
                            }
                            continue;
                        }
                        res = tokio::time::timeout(Duration::from_secs(60), stream.next()) => res,
                    };

                    match chunk_step {
                        Ok(Some(Ok(chunk))) => {
                            if let Err(e) = out_file.write_all(&chunk).await {
                                return Err(format!("Erro ao gravar dados no disco: {}", e));
                            }

                            let len = chunk.len() as u64;
                            overall_downloaded_bytes += len;
                            bytes_since_last_calc += len;

                            let elapsed = last_instant.elapsed();
                            if elapsed >= Duration::from_millis(250) {
                                let current_speed = (bytes_since_last_calc as f64) / elapsed.as_secs_f64();
                                bytes_since_last_calc = 0;
                                last_instant = Instant::now();

                                let eta = if current_speed > 0.0 && overall_total_bytes > overall_downloaded_bytes {
                                    let remaining_bytes = overall_total_bytes - overall_downloaded_bytes;
                                    Some((remaining_bytes as f64 / current_speed) as u64)
                                } else {
                                    None
                                };

                                let progress_pct = if overall_total_bytes > 0 {
                                    ((overall_downloaded_bytes as f64 / overall_total_bytes as f64) * 100.0) as f32
                                } else {
                                    0.0
                                };

                                let mut tasks = tasks_store.lock().await;
                                if let Some(t) = tasks.get_mut(&task_id) {
                                    t.total_bytes = overall_total_bytes;
                                    t.downloaded_bytes = overall_downloaded_bytes;
                                    t.progress_pct = progress_pct.min(99.9);
                                    t.speed_bytes_per_sec = current_speed;
                                    t.eta_seconds = eta;
                                }
                            }
                        }
                        Ok(Some(Err(e))) => {
                            eprintln!("Aviso: erro durante streaming de {}: {}. Retomando automaticamente...", file_name, e);
                            stream_interrupted = true;
                            break;
                        }
                        Ok(None) => {
                            // File stream completed normally
                            break;
                        }
                        Err(_) => {
                            // Timeout reading chunk (stalled network connection for 60s)
                            eprintln!("Aviso: conexão estagnada ao baixar {} (60s sem resposta). Retomando...", file_name);
                            stream_interrupted = true;
                            break;
                        }
                    }
                }

                let _ = out_file.flush().await;
                drop(out_file);

                if stream_interrupted {
                    retry_count += 1;
                    if retry_count > MAX_RETRIES {
                        return Err(format!("Erro durante streaming de {}: limite de tentativas excedido. Você pode clicar em 'Retomar' no painel.", file_name));
                    }
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }

                // File completed successfully
                break;
            }

            // Rename .part to actual file
            if final_file_path.exists() {
                let _ = fs::remove_file(&final_file_path).await;
            }
            fs::rename(&temp_part_path, &final_file_path)
                .await
                .map_err(|e| format!("Erro ao renomear arquivo temporário para {:?}: {}", final_file_path, e))?;
        }

        Ok(())
    }

    pub async fn cancel_download(&self, task_id: &str) -> Result<(), String> {
        let cancels = self.cancel_senders.lock().await;
        if let Some(sender) = cancels.get(task_id) {
            let _ = sender.send(true);
        }

        let mut tasks = self.tasks.lock().await;
        if let Some(task) = tasks.get_mut(task_id) {
            task.status = "cancelled".to_string();
            task.speed_bytes_per_sec = 0.0;
        }

        Ok(())
    }

    pub async fn dismiss_task(&self, task_id: &str) -> Result<(), String> {
        let mut tasks = self.tasks.lock().await;
        if let Some(task) = tasks.remove(task_id) {
            let mut cancels = self.cancel_senders.lock().await;
            cancels.remove(task_id);

            // If it was cancelled or failed, remove potential residual .part files
            let dest = PathBuf::from(&task.destination_path);
            if dest.exists() {
                if let Ok(mut rd) = fs::read_dir(&dest).await {
                    let mut is_empty = true;
                    while let Ok(Some(entry)) = rd.next_entry().await {
                        let p = entry.path();
                        if let Some(ext) = p.extension() {
                            if ext == "part" {
                                let _ = fs::remove_file(&p).await;
                                continue;
                            }
                        }
                        is_empty = false;
                    }
                    if is_empty && task.status != "completed" {
                        let _ = fs::remove_dir(&dest).await;
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn clear_finished_tasks(&self) -> Result<(), String> {
        let mut tasks = self.tasks.lock().await;
        let finished_ids: Vec<String> = tasks
            .iter()
            .filter(|(_, t)| t.status != "downloading")
            .map(|(k, _)| k.clone())
            .collect();

        let mut cancels = self.cancel_senders.lock().await;
        for id in finished_ids {
            tasks.remove(&id);
            cancels.remove(&id);
        }
        Ok(())
    }

    pub async fn get_all_tasks(&self) -> Vec<DownloadTaskProgress> {
        let tasks = self.tasks.lock().await;
        let mut list: Vec<DownloadTaskProgress> = tasks.values().cloned().collect();
        list.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        list
    }
}

fn rand_suffix() -> u16 {
    (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_micros() % 10000) as u16
}

fn urlencoding_simple(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.' || b == b'~' || b == b'/' {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{:02X}", b));
        }
    }
    out
}

fn format_param_count(params: u64) -> String {
    if params >= 1_000_000_000 {
        let b = params as f64 / 1_000_000_000.0;
        let s = format!("{:.1}B", b);
        s.replace(".0B", "B")
    } else if params >= 1_000_000 {
        let m = params as f64 / 1_000_000.0;
        let s = format!("{:.0}M", m);
        s
    } else {
        params.to_string()
    }
}

fn extract_param_size(s: &str) -> Option<String> {
    for part in s.split(|c: char| !c.is_alphanumeric() && c != '.') {
        if part.len() >= 2 && (part.ends_with('b') || part.ends_with('B')) {
            let num_str = &part[..part.len() - 1];
            if let Ok(val) = num_str.parse::<f64>() {
                if val > 0.0 && val < 1000.0 {
                    return Some(format!("{}B", num_str.to_uppercase()));
                }
            }
        }
    }
    None
}

