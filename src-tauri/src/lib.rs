pub mod core;
pub mod services;
pub mod server;
pub mod tray;

use serde::{Deserialize, Serialize};
use tauri::{command, ipc::Channel, AppHandle, Manager, State};
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;
use tauri_plugin_window_state::{AppHandleExt, StateFlags};

use crate::core::hardware::SystemHardwareInfo;
use crate::core::mcp::{McpServerConfig, McpToolDefinition, McpToolWithServer};
use crate::core::memory::{CompressionType, GraphOptimizationReport, NodeType, ProceduralSkill, RelationType, SkillCommandResult, SkillScriptFilePayload, SkillStep, SleepConsolidationReport, VigiliaEvent};
use crate::core::model::{BackendType, ChatMessage, InferenceParams, ModelInfo};
#[allow(unused_imports)]
use crate::core::process::{silent_command, silent_tokio_command, SilentCommand};
use crate::core::server::{DeveloperLogEntry, ServerRequestLog};
use crate::services::backend::{BackendManager, ModelLoadProgress};
use crate::services::downloader::{
    DownloadTaskProgress, HfModelDetail, HfModelSummary, ModelDownloader,
};
use crate::services::engine::InferenceEngine;
use crate::services::mcp_service::McpManager;
use crate::services::memory_engine::{EngineStats, EpisodeItem, FullGraphData, MemoryGraphEngine, MemoryQueryResult};
use crate::services::plugins::{PluginInfo, PluginManager};
use crate::services::runtime::{BootstrapProgress, RuntimeManager, RuntimeStatus};
use crate::services::scanner::ModelScanner;
use crate::services::server_ctl::LocalServerController;
use crate::services::skill_runner::SkillScriptRunner;

pub struct AppState {
    pub backend_manager: BackendManager,
    pub mcp_manager: McpManager,
    pub downloader: ModelDownloader,
    pub runtime_manager: RuntimeManager,
    pub hardware_info: Mutex<SystemHardwareInfo>,
    pub server_logs: Mutex<Vec<ServerRequestLog>>,
    pub active_model: Mutex<Option<ModelInfo>>,
    pub memory_engine: Mutex<MemoryGraphEngine>,
    pub db: Arc<crate::services::db::DatabaseService>,
    pub gateways_manager: Arc<crate::services::gateways::GatewaysManager>,
    pub scratchpad: Arc<crate::services::scratchpad::SessionScratchpadManager>,
    pub scheduler: Arc<crate::services::scheduler::BackgroundScheduler>,
}

impl AppState {
    pub fn new() -> Self {
        log::info!("🧠 Loading persistent cognitive memory graph...");
        let initial_memory_engine = MemoryGraphEngine::load_default_or_init();
        RuntimeManager::auto_provision_bundled_tools();

        let db = Arc::new(crate::services::db::DatabaseService::new().unwrap_or_else(|e| {
            log::error!("Failed to initialize SQLite database: {}", e);
            panic!("Failed to initialize database: {}", e);
        }));

        Self {
            backend_manager: BackendManager::new(),
            mcp_manager: McpManager::new(),
            downloader: ModelDownloader::new(),
            runtime_manager: RuntimeManager::new(),
            hardware_info: Mutex::new(SystemHardwareInfo::detect_system()),
            server_logs: Mutex::new(LocalServerController::mock_initial_logs()),
            active_model: Mutex::new(None),
            memory_engine: Mutex::new(initial_memory_engine),
            db,
            gateways_manager: Arc::new(crate::services::gateways::GatewaysManager::new()),
            scratchpad: Arc::new(crate::services::scratchpad::SessionScratchpadManager::new()),
            scheduler: Arc::new(crate::services::scheduler::BackgroundScheduler::new()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChunkPayload {
    pub content: String,
    pub thinking: Option<String>,
    pub is_done: bool,
    pub elapsed_ms: u64,
    pub tool_calls: Option<Vec<crate::core::mcp::McpToolCall>>,
    pub metrics: Option<crate::core::model::GenerationMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub mlx_online: bool,
    pub ollama_online: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub os: String,
    pub supports_mlx: bool,
    pub supports_whisper_local: bool,
}

#[command]
async fn get_platform_info() -> Result<PlatformInfo, String> {
    let os = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "linux"
    };
    Ok(PlatformInfo {
        os: os.to_string(),
        supports_mlx: cfg!(target_os = "macos"),
        supports_whisper_local: true,
    })
}

#[command]
async fn select_folder(default_path: Option<String>) -> Result<Option<String>, String> {
    let mut dialog = rfd::AsyncFileDialog::new().set_title("Select Models Directory");
    if let Some(ref path) = default_path {
        let resolved = crate::core::config::resolve_path(path);
        if resolved.exists() {
            dialog = dialog.set_directory(&resolved);
        }
    }
    let folder = dialog.pick_folder().await;
    Ok(folder.map(|f| {
        let path_str = f.path().to_string_lossy().to_string();
        crate::core::config::contract_path(&path_str)
    }))
}

#[derive(serde::Deserialize)]
pub struct FileDialogFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

#[command]
async fn save_file_content(
    title: Option<String>,
    default_name: Option<String>,
    default_path: Option<String>,
    filters: Option<Vec<FileDialogFilter>>,
    content: String,
) -> Result<Option<String>, String> {
    let mut dialog = rfd::AsyncFileDialog::new();
    if let Some(ref t) = title {
        if !t.trim().is_empty() {
            dialog = dialog.set_title(t);
        }
    }
    if let Some(ref name) = default_name {
        if !name.trim().is_empty() {
            dialog = dialog.set_file_name(name);
        }
    }
    if let Some(ref path) = default_path {
        if !path.trim().is_empty() && std::path::Path::new(path).exists() {
            dialog = dialog.set_directory(path);
        }
    }
    if let Some(ref flts) = filters {
        for f in flts {
            let ext_slices: Vec<&str> = f.extensions.iter().map(|s| s.trim_start_matches('.')).collect();
            dialog = dialog.add_filter(&f.name, &ext_slices);
        }
    }
    let file_handle = dialog.save_file().await;
    if let Some(handle) = file_handle {
        let path = handle.path().to_path_buf();
        tokio::fs::write(&path, content.as_bytes())
            .await
            .map_err(|e| format!("Falha ao salvar arquivo: {}", e))?;
        Ok(Some(path.to_string_lossy().to_string()))
    } else {
        Ok(None)
    }
}


#[command]
async fn open_url(url: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(&url).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = silent_command("cmd").args(["/C", "start", &url]).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
    }
    Ok(())
}

#[command]
async fn hf_search_models(
    query: String,
    filter_type: Option<String>,
    sort: Option<String>,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<HfModelSummary>, String> {
    state.downloader.search_hf_models(&query, filter_type.as_deref(), sort.as_deref(), limit.unwrap_or(30)).await
}

#[command]
async fn hf_get_model_details(
    repo_id: String,
    state: State<'_, AppState>,
) -> Result<HfModelDetail, String> {
    state.downloader.get_model_details(&repo_id).await
}

#[command]
async fn hf_get_author_avatar(
    author: String,
    state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    Ok(state.downloader.get_author_avatar(&author).await)
}

#[command]
async fn hf_get_author_avatars(
    authors: Vec<String>,
    state: State<'_, AppState>,
) -> Result<std::collections::HashMap<String, Option<String>>, String> {
    Ok(state.downloader.get_author_avatars(authors).await)
}

#[command]
async fn start_hf_download(
    repo_id: String,
    selected_files: Vec<String>,
    target_dir: String,
    is_mlx_bundle: bool,
    model_title: Option<String>,
    author_avatar_url: Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    state.downloader.start_download(repo_id, selected_files, target_dir, is_mlx_bundle, model_title, author_avatar_url).await
}

#[command]
async fn check_hf_model_downloaded(
    repo_id: String,
    files: Vec<String>,
    models_dirs: Vec<String>,
    is_mlx_bundle: bool,
) -> Result<Option<ModelInfo>, String> {
    Ok(ModelScanner::check_model_exists_on_disk(&models_dirs, &repo_id, &files, is_mlx_bundle).await)
}

#[command]
async fn cancel_hf_download(
    task_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.downloader.cancel_download(&task_id).await
}

#[command]
async fn retry_hf_download(
    task_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.downloader.retry_download(&task_id).await
}

#[command]
async fn dismiss_download_task(
    task_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.downloader.dismiss_task(&task_id).await
}

#[command]
async fn clear_finished_downloads(
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.downloader.clear_finished_tasks().await
}

#[command]
async fn get_active_downloads(
    state: State<'_, AppState>,
) -> Result<Vec<DownloadTaskProgress>, String> {
    Ok(state.downloader.get_all_tasks().await)
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct CliStatus {
    pub installed: bool,
    pub path: Option<String>,
}

pub async fn check_cli_installed_internal() -> Result<CliStatus, String> {
    #[cfg(unix)]
    {
        let local_bin = crate::core::config::resolve_path("~/.local/bin/atena");
        let usr_bin = std::path::PathBuf::from("/usr/local/bin/atena");

        if local_bin.exists() || local_bin.is_symlink() {
            return Ok(CliStatus {
                installed: true,
                path: Some(local_bin.to_string_lossy().to_string()),
            });
        }

        if usr_bin.exists() || usr_bin.is_symlink() {
            return Ok(CliStatus {
                installed: true,
                path: Some(usr_bin.to_string_lossy().to_string()),
            });
        }

        if let Ok(output) = std::process::Command::new("which").arg("atena").output() {
            if output.status.success() {
                let found = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !found.is_empty() {
                    return Ok(CliStatus {
                        installed: true,
                        path: Some(found),
                    });
                }
            }
        }

        Ok(CliStatus {
            installed: false,
            path: None,
        })
    }

    #[cfg(windows)]
    {
        if let Ok(output) = silent_command("where").arg("atena").output() {
            if output.status.success() {
                let found = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !found.is_empty() {
                    return Ok(CliStatus {
                        installed: true,
                        path: Some(found),
                    });
                }
            }
        }
        Ok(CliStatus {
            installed: false,
            path: None,
        })
    }
}

#[command]
async fn check_cli_installed() -> Result<CliStatus, String> {
    check_cli_installed_internal().await
}

pub async fn install_cli_command_internal() -> Result<String, String> {
    #[cfg(unix)]
    {
        // 1. Determine target binary path
        let app_bundle_path = std::path::PathBuf::from("/Applications/Atena Studio.app/Contents/MacOS/app");
        let target_bin = if app_bundle_path.exists() {
            app_bundle_path
        } else {
            std::env::current_exe().map_err(|e| format!("Failed to get executable: {}", e))?
        };

        // 2. Try ~/.local/bin/atena first (always writable by user without sudo)
        let local_bin_dir = crate::core::config::resolve_path("~/.local/bin");
        let local_bin_link = local_bin_dir.join("atena");
        let _ = tokio::fs::create_dir_all(&local_bin_dir).await;
        let _ = tokio::fs::remove_file(&local_bin_link).await;

        let mut installed_paths = Vec::new();
        #[cfg(target_family = "unix")]
        {
            if std::os::unix::fs::symlink(&target_bin, &local_bin_link).is_ok() {
                installed_paths.push(local_bin_link.to_string_lossy().to_string());
            }
        }

        // 3. Try /usr/local/bin/atena
        let usr_link = std::path::Path::new("/usr/local/bin/atena");
        let _ = tokio::fs::remove_file(&usr_link).await;
        #[cfg(target_family = "unix")]
        {
            if std::os::unix::fs::symlink(&target_bin, &usr_link).is_ok() {
                installed_paths.push(usr_link.to_string_lossy().to_string());
            }
        }

        // If direct symlink succeeded, return success!
        if !installed_paths.is_empty() {
            return Ok(installed_paths.join(", "));
        }

        // 4. Fallback on macOS with admin privileges if neither could be created directly
        #[cfg(target_os = "macos")]
        {
            let script = format!(
                "do shell script \"mkdir -p /usr/local/bin && ln -sf '{}' /usr/local/bin/atena\" with administrator privileges",
                target_bin.to_string_lossy()
            );
            let out = std::process::Command::new("osascript")
                .arg("-e")
                .arg(&script)
                .output()
                .map_err(|e| format!("Failed to execute osascript: {}", e))?;

            if out.status.success() {
                return Ok("/usr/local/bin/atena".to_string());
            } else {
                let err = String::from_utf8_lossy(&out.stderr);
                return Err(format!("Permission denied: {}", err.trim()));
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err("Could not create symlink in ~/.local/bin or /usr/local/bin.".to_string())
        }
    }

    #[cfg(windows)]
    {
        let target_dir = std::env::current_exe()
            .map_err(|e| e.to_string())?
            .parent()
            .map(|p| p.to_path_buf())
            .ok_or_else(|| "Could not determine executable directory".to_string())?;

        let ps_script = format!(
            "[Environment]::SetEnvironmentVariable('Path', [Environment]::GetEnvironmentVariable('Path', 'User') + ';{}', 'User')",
            target_dir.to_string_lossy()
        );

        let out = silent_command("powershell")
            .args(["-NoProfile", "-Command", &ps_script])
            .output()
            .map_err(|e| e.to_string())?;

        if out.status.success() {
            Ok(target_dir.to_string_lossy().to_string())
        } else {
            Err(String::from_utf8_lossy(&out.stderr).to_string())
        }
    }
}

#[command]
async fn install_cli_command() -> Result<String, String> {
    install_cli_command_internal().await
}

#[command]
async fn delete_model(
    model_id: String,
    local_path: Option<String>,
    format: Option<String>,
    backend: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
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
        return Ok(());
    }

    // 2. If it has a local path on disk, remove directory or file
    if let Some(ref path_str) = local_path {
        let path = std::path::Path::new(path_str);
        if path.exists() {
            if path.is_dir() {
                tokio::fs::remove_dir_all(path)
                    .await
                    .map_err(|e| format!("Falha ao excluir diretório do modelo: {}", e))?;
            } else if path.is_file() {
                let parent = path.parent().map(|p| p.to_path_buf());
                tokio::fs::remove_file(path)
                    .await
                    .map_err(|e| format!("Falha ao excluir arquivo do modelo: {}", e))?;

                // If parent directory is now empty and not the root directory, remove it
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

    Ok(())
}

#[command]
fn get_cached_models() -> Result<Vec<ModelInfo>, String> {
    Ok(ModelScanner::load_cached_models())
}

#[command]
async fn scan_models(
    models_dir: Option<String>,
    models_dirs: Option<Vec<String>>,
    ollama_host: String,
    ollama_port: u16,
) -> Result<Vec<ModelInfo>, String> {
    let mut dirs = Vec::new();
    if let Some(ds) = models_dirs {
        for d in ds {
            let trimmed = d.trim().to_string();
            if !trimmed.is_empty() && !dirs.contains(&trimmed) {
                dirs.push(trimmed);
            }
        }
    }
    if let Some(d) = models_dir {
        let trimmed = d.trim().to_string();
        if !trimmed.is_empty() && !dirs.contains(&trimmed) {
            dirs.push(trimmed);
        }
    }
    if dirs.is_empty() {
        dirs.push(crate::core::config::default_models_directory());
    }
    Ok(ModelScanner::scan_all_directories(&dirs, &ollama_host, ollama_port).await)
}

#[command]
fn detect_model_directories() -> Result<Vec<crate::services::scanner::DetectedModelDirectory>, String> {
    Ok(ModelScanner::detect_common_model_dirs())
}

#[command]
fn get_default_models_directory() -> Result<String, String> {
    Ok(crate::core::config::default_models_directory())
}

#[command]
async fn check_agy_session() -> Result<crate::services::backend::AgySessionStatus, String> {
    Ok(BackendManager::check_agy_session().await)
}

#[command]
fn start_agy_login() -> Result<(), String> {
    BackendManager::launch_agy_login_terminal()
}

#[command]
fn launch_terminal_command(command: String) -> Result<(), String> {
    BackendManager::launch_terminal_command(&command)
}

#[command]
async fn get_agy_usage() -> Result<crate::services::backend::AgyUsageData, String> {
    BackendManager::get_agy_usage().await
}

#[command]
async fn test_cloud_provider_connection(
    provider: String,
    api_key: String,
    base_url: Option<String>,
) -> Result<String, String> {
    BackendManager::test_cloud_provider_connection(&provider, &api_key, base_url.as_deref()).await
}

#[command]
async fn fetch_provider_models(
    provider: String,
    api_key: Option<String>,
    base_url: Option<String>,
) -> Result<Vec<crate::core::model::ModelInfo>, String> {
    crate::services::scanner::ModelScanner::fetch_provider_models_direct(&provider, api_key.as_deref(), base_url.as_deref()).await
}

#[command]
fn get_app_config() -> Result<crate::core::config::AppConfig, String> {
    Ok(crate::core::config::AppConfig::load())
}

#[command]
async fn save_app_config(
    config: serde_json::Value,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut current = crate::core::config::AppConfig::load();
    let saved_cfg = if let Ok(parsed) = serde_json::from_value::<crate::core::config::AppConfig>(config.clone()) {
        parsed.save()?;
        let _ = crate::tray::update_tray_locale(&app, &parsed.language);
        parsed
    } else {
        if let Some(cp) = config.get("cloud_providers") {
            if let Ok(parsed_cp) = serde_json::from_value::<crate::core::config::CloudProvidersConfig>(cp.clone()) {
                current.cloud_providers = parsed_cp;
            }
        }
        if let Some(mem) = config.get("enable_cognitive_memory").and_then(|v| v.as_bool()) {
            current.enable_cognitive_memory = mem;
        }
        if let Some(mem) = config.get("enable_facts_memory").and_then(|v| v.as_bool()) {
            current.enable_facts_memory = mem;
        }
        if let Some(mem) = config.get("enable_skills_memory").and_then(|v| v.as_bool()) {
            current.enable_skills_memory = mem;
        }
        if let Some(mem) = config.get("enable_episodic_memory").and_then(|v| v.as_bool()) {
            current.enable_episodic_memory = mem;
        }
        if let Some(gw) = config.get("gateways") {
            if let Ok(parsed_gw) = serde_json::from_value::<crate::core::config::GatewaysConfig>(gw.clone()) {
                current.gateways = parsed_gw;
            }
        }
        if let Some(bg) = config.get("run_in_background").and_then(|v| v.as_bool()) {
            current.run_in_background = bg;
        }
        if let Some(ct) = config.get("close_to_tray").and_then(|v| v.as_bool()) {
            current.close_to_tray = ct;
        }
        if let Some(tz) = config.get("timezone").and_then(|v| v.as_str()) {
            current.timezone = tz.to_string();
        }
        if let Some(lang) = config.get("language").and_then(|v| v.as_str()) {
            current.language = lang.to_string();
            let _ = crate::tray::update_tray_locale(&app, lang);
        }
        current.save()?;
        current
    };

    // Synchronize background gateway workers with updated configuration
    state.gateways_manager.sync_with_config(app, &saved_cfg).await;
    crate::services::scheduler::BackgroundScheduler::refresh_all_task_schedules(&state);
    Ok(())
}

#[command]
fn update_tray_locale(app: AppHandle, locale: String) -> Result<(), String> {
    crate::tray::update_tray_locale(&app, &locale).map_err(|e| e.to_string())
}

#[command]
fn get_gateways_status(state: State<'_, AppState>) -> Result<Vec<crate::services::gateways::GatewayStatusReport>, String> {
    let cfg = crate::core::config::AppConfig::load();
    Ok(state.gateways_manager.get_status_reports(&cfg))
}

#[command]
async fn test_telegram_connection(token: String) -> Result<String, String> {
    crate::services::gateways::GatewaysManager::test_telegram_token(&token).await
}

#[command]
async fn test_discord_connection(token: String) -> Result<String, String> {
    crate::services::gateways::GatewaysManager::test_discord_token(&token).await
}

#[command]
async fn restart_gateways(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let cfg = crate::core::config::AppConfig::load();
    state.gateways_manager.sync_with_config(app, &cfg).await;
    Ok(())
}

#[command]
fn get_cloud_providers_config() -> Result<crate::core::config::CloudProvidersConfig, String> {
    Ok(crate::core::config::AppConfig::load().cloud_providers)
}

#[command]
fn save_cloud_providers_config(cloud_providers: crate::core::config::CloudProvidersConfig) -> Result<(), String> {
    let mut current = crate::core::config::AppConfig::load();
    current.cloud_providers = cloud_providers;
    current.save()
}


#[command]
async fn get_hardware_info(state: State<'_, AppState>) -> Result<SystemHardwareInfo, String> {
    let mut hw = state.hardware_info.lock().await;
    hw.refresh();
    Ok(hw.clone())
}

#[command]
async fn check_services(
    mlx_host: String,
    mlx_port: u16,
    ollama_host: String,
    ollama_port: u16,
) -> Result<ServiceHealth, String> {
    let t_mlx = std::time::Instant::now();
    let mlx_online = BackendManager::check_mlx_health(&mlx_host, mlx_port).await;
    let mlx_latency = t_mlx.elapsed().as_millis() as u64;
    LocalServerController::record_log(
        "GET",
        &format!("http://{}:{}/v1/models", mlx_host, mlx_port),
        if mlx_online { 200 } else { 503 },
        mlx_latency,
        0,
        0,
    );

    let t_ollama = std::time::Instant::now();
    let ollama_online = BackendManager::check_ollama_health(&ollama_host, ollama_port).await;
    let ollama_latency = t_ollama.elapsed().as_millis() as u64;
    LocalServerController::record_log(
        "GET",
        &format!("http://{}:{}/api/version", ollama_host, ollama_port),
        if ollama_online { 200 } else { 503 },
        ollama_latency,
        0,
        0,
    );

    Ok(ServiceHealth {
        mlx_online,
        ollama_online,
    })
}

#[command]
async fn refresh_model_info(
    model: ModelInfo,
) -> Result<ModelInfo, String> {
    Ok(ModelScanner::refresh_single_model(&model).await)
}

#[command]
async fn set_active_model(
    model: Option<ModelInfo>,
    state: State<'_, AppState>,
) -> Result<Option<ModelInfo>, String> {
    let fresh = if let Some(ref m) = model {
        Some(ModelScanner::refresh_single_model(m).await)
    } else {
        None
    };
    let mut active = state.active_model.lock().await;
    *active = fresh.clone();
    Ok(fresh)
}

#[command]
async fn get_active_model(
    state: State<'_, AppState>,
) -> Result<Option<ModelInfo>, String> {
    let active = state.active_model.lock().await;
    Ok(active.clone())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamChatRequest {
    pub model: Option<ModelInfo>,
    pub system_prompt: String,
    pub messages: Option<Vec<ChatMessage>>,
    pub user_message: Option<String>,
    pub session_id: Option<String>,
    pub session_title: Option<String>,
    pub params: InferenceParams,
    pub mlx_host: String,
    pub mlx_port: u16,
    pub ollama_host: String,
    pub ollama_port: u16,
    pub enable_memory: Option<bool>,
    pub enable_facts_memory: Option<bool>,
    pub enable_skills_memory: Option<bool>,
    pub enable_episodic_memory: Option<bool>,
}

pub async fn execute_stream_chat_internal(
    state: &AppState,
    req: StreamChatRequest,
    on_event: Arc<dyn Fn(ChatChunkPayload) + Send + Sync>,
) -> Result<(), String> {
    let StreamChatRequest {
        model,
        system_prompt,
        messages,
        user_message,
        session_id,
        session_title,
        mut params,
        mlx_host,
        mlx_port,
        ollama_host,
        ollama_port,
        enable_memory,
        enable_facts_memory,
        enable_skills_memory,
        enable_episodic_memory,
    } = req;

    let is_memory_enabled = enable_memory.unwrap_or(false);
    let app_cfg = crate::core::config::AppConfig::load();
    let use_facts = is_memory_enabled && enable_facts_memory.unwrap_or(app_cfg.enable_facts_memory);
    let use_skills = is_memory_enabled && enable_skills_memory.unwrap_or(app_cfg.enable_skills_memory);
    let use_episodic = is_memory_enabled && enable_episodic_memory.unwrap_or(app_cfg.enable_episodic_memory);

    // Backend-side injection: cognitive memory and procedural skill tools are managed
    // exclusively by the memory toggles, not by the frontend's native tools toggle.
    // The frontend never sends these tools — the backend injects them autonomously.
    if is_memory_enabled {
        let cognitive_tools = McpManager::memory_and_skills_tools();
        if let Some(ref mut tools) = params.mcp_tools {
            // Avoid duplicates: only add tools not already present
            for ct in cognitive_tools {
                if !tools.iter().any(|t| t.tool.name == ct.tool.name) {
                    tools.push(ct);
                }
            }
        } else {
            params.mcp_tools = Some(cognitive_tools);
        }

        // Selectively remove tools for disabled sub-features
        if !use_facts {
            if let Some(ref mut tools) = params.mcp_tools {
                tools.retain(|t| t.tool.name != "atena_search_memory");
            }
        }
        if !use_episodic {
            if let Some(ref mut tools) = params.mcp_tools {
                tools.retain(|t| t.tool.name != "atena_search_episodes" && t.tool.name != "atena_read_episode");
            }
        }
        if !use_skills {
            if let Some(ref mut tools) = params.mcp_tools {
                tools.retain(|t| {
                    t.server_id != "skills"
                        && t.tool.name != "run_command"
                        && t.tool.name != "run_skill_command"
                        && t.tool.name != "run_skill_script"
                        && t.tool.name != "create_procedural_skill"
                        && t.tool.name != "update_procedural_skill"
                        && t.tool.name != "edit_procedural_skill"
                });
            }
        }
    } else {
        log::info!("🧠 Cognitive memory disabled or suppressed: lightweight inference without neural associative prompt, facts, episodic tools, or procedural skills.");
        // When memory is fully disabled, ensure no cognitive tools leak through
        if let Some(ref mut tools) = params.mcp_tools {
            tools.retain(|t| {
                t.tool.name != "atena_search_memory"
                    && t.tool.name != "atena_search_episodes"
                    && t.tool.name != "atena_read_episode"
                    && t.server_id != "skills"
                    && t.tool.name != "run_command"
                    && t.tool.name != "run_skill_command"
                    && t.tool.name != "run_skill_script"
                    && t.tool.name != "create_procedural_skill"
                    && t.tool.name != "update_procedural_skill"
                    && t.tool.name != "edit_procedural_skill"
            });
        }
    }
    let raw_user_text = user_message.clone().unwrap_or_default();

    // 0. Registrar no buffer diário de vigília do Hipocampo (~/.atena/brain/hipocampo/diario_vigilia.atena)
    if is_memory_enabled && use_episodic && !raw_user_text.trim().is_empty() {
        let _ = MemoryGraphEngine::record_vigilia_event("user", &raw_user_text, None, None);
    }

    // Contexto textual expandido com as mensagens recentes da conversa (para capturar entidades e temas anteriores)
    let full_conversation_context_text = {
        let mut text = String::new();
        if let Some(ref msgs) = messages {
            for m in msgs.iter().rev().take(6) {
                text.push_str(&m.content);
                text.push(' ');
            }
        }
        if text.trim().is_empty() {
            raw_user_text.clone()
        } else {
            text
        }
    };

    // 1. Consult active associative memory, user profile, recent records, procedural skills, and episodes
    let (memory_context, user_profile_context, recent_records_context) = if is_memory_enabled && use_facts {
        let mut engine = state.memory_engine.lock().await;
        let query_ctx = engine.search_active_context_for_query(&full_conversation_context_text);
        let profile_ctx = engine.get_user_profile_context();
        let recent_ctx = engine.get_recent_memory_records(8);
        (query_ctx, profile_ctx, recent_ctx)
    } else {
        (None, None, None)
    };
    let skills_context = if is_memory_enabled && use_skills {
        MemoryGraphEngine::build_skills_prompt_context(&full_conversation_context_text)
    } else {
        None
    };

    let episodic_context = if is_memory_enabled && use_episodic {
        MemoryGraphEngine::get_recent_episodes_context(2)
    } else {
        None
    };

    let now_local = chrono::Local::now();
    let current_date_str = now_local.format("%d/%m/%Y").to_string();
    let current_time_str = now_local.format("%H:%M:%S").to_string();
    let current_day_name = match now_local.format("%u").to_string().as_str() {
        "1" => "Monday",
        "2" => "Tuesday",
        "3" => "Wednesday",
        "4" => "Thursday",
        "5" => "Friday",
        "6" => "Saturday",
        "7" => "Sunday",
        _ => "Today",
    };

    // 2. Inject associative memory, user profile, skills, and instructions into system prompt
    let autonomous_memory_instruction = format!(r##"
[AUTONOMOUS COGNITIVE MEMORY SYSTEM]:
You maintain an autonomous long-term cognitive memory graph across conversations.

MANDATORY CONVERSATIONAL DIRECTIVE:
- You are first and foremost an intelligent, friendly, and helpful AI assistant.
- You MUST ALWAYS respond to the user in natural language FIRST (converse warmly, acknowledge shared details, answer questions, or provide insights).
- NEVER end your turn with only memory tags or finish without speaking directly to the user!
- Memory tags (<memory>, <forget>) are silent background metadata that MUST ONLY be appended at the very end of your response, AFTER your full conversational reply.

1. REMEMBERING FACTS (<memory ... />):
When the user mentions personal facts, family, friends, pets, preferences, plans, or milestones worth remembering long-term:
Append concise XML tag(s) at the very end of your response:
<memory subject="User|EntityName" property="Concise Fact or Relationship" type="Object" valence="1" />
- subject: "User" for user-specific facts, or the specific person/entity name (e.g. "Maria").
- property: Clear, concise statement in the SAME LANGUAGE as the conversation (e.g. in Portuguese: "Moedor de Café: Comandante C40", "Alergia: Frutos do mar", "Pet: Gato chamado Byte"; in English: "Coffee Grinder: Comandante C40", "Allergy: Seafood").
- type: "Object" (default) or "RuleOrAlert" (strictly for negative rules/constraints/allergies to avoid, valence="-1").
- valence: "1" (positive/reinforce), "0" (neutral), "-1" (inhibitory/avoid).
- Do NOT memorize fleeting pleasantries, simple greetings, or temporary queries.

2. RETRIEVING & SEARCHING MEMORY:
- When querying memory via `atena_search_memory` or `atena_search_episodes`, formulate query terms matching the concepts in the conversational language or keywords (e.g. "moedor", "café", "alergia", "coffee grinder", "allergy").

3. FORGETTING & CORRECTING OBSOLETE DATA (<forget ... />):
If you notice outdated, conflicting, or corrected records in [LATEST RECORDS IN YOUR BRAIN], append:
<forget subject="EntityName" property="Fact or *" />

4. REFERENCE DATE & TIME:
- Today is: {} ({}) at {}.
- Convert relative dates (e.g. "tomorrow", "next week", "yesterday") to absolute DD/MM/YYYY dates in your memory tags.
"##,
        current_date_str,
        current_day_name,
        current_time_str
    );

    let mut effective_system_prompt = String::new();
    let base_prompt_clean = system_prompt.trim();

    if !base_prompt_clean.is_empty() {
        effective_system_prompt.push_str(base_prompt_clean);
    } else {
        effective_system_prompt.push_str("You are Atena Studio, an intelligent, helpful, and friendly AI assistant. You communicate clearly, naturally, and warmly.");
    }

    if is_memory_enabled {
        if use_facts {
            effective_system_prompt.push_str("\n\n");
            effective_system_prompt.push_str(&autonomous_memory_instruction);

            if let Some(profile_str) = user_profile_context {
                if !profile_str.trim().is_empty() {
                    log::info!("Injecting user profile into prompt");
                    effective_system_prompt.push_str(&format!(
                        "\n\n[USER PROFILE & CORE ATTRIBUTES]:\n{}\n(Instruction: Seamlessly incorporate these core user characteristics into your tone and context.)",
                        profile_str.trim()
                    ));
                }
            }

            if let Some(recent_str) = recent_records_context {
                if !recent_str.trim().is_empty() {
                    log::info!("Injecting recent brain records into prompt");
                    effective_system_prompt.push_str(&format!(
                        "\n\n[LATEST RECORDS IN YOUR BRAIN]:\n{}\n(Instruction: These are facts you already know. If updated or corrected by the user, emit the updated <memory> or <forget> tag at the end of your response.)",
                        recent_str.trim()
                    ));
                }
            }

            if let Some(mem_str) = memory_context {
                if !mem_str.trim().is_empty() {
                    log::info!("Injecting active neural associative memory context into prompt");
                    effective_system_prompt.push_str(&format!(
                        "\n\n[ACTIVATED ASSOCIATIVE MEMORY CONTEXT]:\n{}\n(Instruction: Contextually relevant knowledge nodes dynamically activated from your memory graph. Use them naturally to provide coherent responses.)",
                        mem_str.trim()
                    ));
                }
            }
        }

        if use_skills {
            if let Some(sk_str) = skills_context {
                if !sk_str.trim().is_empty() {
                    log::info!("Injecting procedural skills context into prompt");
                    effective_system_prompt.push_str(&sk_str);
                }
            }
        }

        if use_episodic {
            if let Some(ep_str) = episodic_context {
                if !ep_str.trim().is_empty() {
                    log::info!("Injecting recent episodic chain into prompt");
                    effective_system_prompt.push_str(&format!(
                        "\n\n{}\n(Instruction: Recent episode files for conversational continuity.)",
                        ep_str.trim()
                    ));
                }
            }
        }
    }

    let final_messages = if let Some(mut msgs) = messages {
        if let Some(user_msg) = user_message {
            if !user_msg.trim().is_empty() {
                let already_present = msgs
                    .last()
                    .map(|m| m.role == "user" && m.content.trim() == user_msg.trim())
                    .unwrap_or(false);
                if !already_present {
                    msgs.push(ChatMessage {
                        role: "user".to_string(),
                        content: user_msg,
                        images: None,
                        tool_calls: None,
                        tool_call_id: None,
                    });
                }
            }
        }
        msgs
    } else if let Some(user_msg) = user_message {
        if !user_msg.trim().is_empty() {
            vec![ChatMessage {
                role: "user".to_string(),
                content: user_msg,
                images: None,
                tool_calls: None,
                tool_call_id: None,
            }]
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let full_assistant_response = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let full_resp_clone = full_assistant_response.clone();
    let on_event_cb = on_event.clone();

    InferenceEngine::execute_chat_stream(
        model,
        effective_system_prompt,
        final_messages,
        params,
        mlx_host,
        mlx_port,
        ollama_host,
        ollama_port,
        move |content, thinking, tool_calls, is_done, elapsed_ms, metrics| {
            if !content.is_empty() {
                if let Ok(mut lock) = full_resp_clone.lock() {
                    *lock = content.clone();
                }
            }
            on_event_cb(ChatChunkPayload {
                content,
                thinking,
                is_done,
                elapsed_ms,
                tool_calls,
                metrics,
            });
        },
    )
    .await;

    // Ao finalizar a geração da IA: processar tags de memória autônoma e gravar vigília
    let full_resp_str = {
        if let Ok(lock) = full_assistant_response.lock() {
            lock.trim().to_string()
        } else {
            String::new()
        }
    };

    if !full_resp_str.is_empty() && is_memory_enabled {
        let (clean_resp, learned) = if use_facts {
            let mut engine = state.memory_engine.lock().await;
            let (clean_resp, learned) = engine.extract_and_apply_memory_tags(&full_resp_str);
            for item in &learned {
                log::info!("🧠 Memória Autônoma: {}", item);
            }
            (clean_resp, learned)
        } else {
            (full_resp_str.clone(), Vec::new())
        };

        if use_episodic {
            let _ = MemoryGraphEngine::record_vigilia_event(
                "atena",
                &clean_resp,
                None,
                Some("resposta_ia"),
            );

            // Gravar elo da cadeia episódica em Markdown (.md)
            if let Err(e) = MemoryGraphEngine::record_episodic_turn(
                &raw_user_text,
                &clean_resp,
                &learned,
                session_id.as_deref(),
                session_title.as_deref(),
            ) {
                log::warn!("⚠️ Erro ao registrar elo episódico em Markdown: {}", e);
            }
        }
    }

    Ok(())
}

#[command]
async fn stream_chat(
    model: Option<ModelInfo>,
    system_prompt: String,
    messages: Option<Vec<ChatMessage>>,
    user_message: Option<String>,
    session_id: Option<String>,
    session_title: Option<String>,
    params: InferenceParams,
    mlx_host: String,
    mlx_port: u16,
    ollama_host: String,
    ollama_port: u16,
    enable_memory: Option<bool>,
    enable_facts_memory: Option<bool>,
    enable_skills_memory: Option<bool>,
    enable_episodic_memory: Option<bool>,
    state: State<'_, AppState>,
    on_event: Channel<ChatChunkPayload>,
) -> Result<(), String> {
    let req = StreamChatRequest {
        model,
        system_prompt,
        messages,
        user_message,
        session_id,
        session_title,
        params,
        mlx_host,
        mlx_port,
        ollama_host,
        ollama_port,
        enable_memory,
        enable_facts_memory,
        enable_skills_memory,
        enable_episodic_memory,
    };
    let cb = Arc::new(move |chunk| {
        let _ = on_event.send(chunk);
    });
    execute_stream_chat_internal(&state, req, cb).await
}

#[command]
fn stop_chat_generation() {
    crate::services::backend::BackendManager::abort_active_inference();
}

#[command]
async fn start_mlx_server(
    model_path: String,
    host: String,
    port: u16,
    channel: Channel<ModelLoadProgress>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let progress_cb = Arc::new(move |prog: ModelLoadProgress| {
        let _ = channel.send(prog);
    }) as Arc<dyn Fn(ModelLoadProgress) + Send + Sync>;
    state.backend_manager.start_mlx_server(&model_path, &host, port, Some(progress_cb)).await
}

#[command]
async fn start_llama_server(
    model_path: String,
    host: String,
    port: u16,
    context_length: Option<usize>,
    kv_cache_quant: Option<String>,
    flash_attn: Option<bool>,
    prompt_cache: Option<bool>,
    channel: Channel<ModelLoadProgress>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let progress_cb = Arc::new(move |prog: ModelLoadProgress| {
        let _ = channel.send(prog);
    }) as Arc<dyn Fn(ModelLoadProgress) + Send + Sync>;
    state.backend_manager.start_llama_server(
        &model_path,
        &host,
        port,
        context_length.unwrap_or(8192),
        kv_cache_quant,
        flash_attn,
        prompt_cache,
        Some(progress_cb),
    ).await
}

#[command]
async fn start_ollama_server(
    channel: Channel<ModelLoadProgress>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let progress_cb = Arc::new(move |prog: ModelLoadProgress| {
        let _ = channel.send(prog);
    }) as Arc<dyn Fn(ModelLoadProgress) + Send + Sync>;
    state.backend_manager.start_ollama_serve(Some(progress_cb)).await
}

#[command]
async fn stop_all_servers(state: State<'_, AppState>) -> Result<(), String> {
    state.backend_manager.stop_all().await;
    Ok(())
}

#[command]
async fn get_server_logs(state: State<'_, AppState>) -> Result<Vec<ServerRequestLog>, String> {
    let logs = LocalServerController::get_logs();
    if let Ok(mut state_logs) = state.server_logs.try_lock() {
        *state_logs = logs.clone();
    }
    Ok(logs)
}

#[command]
async fn get_developer_logs() -> Result<Vec<DeveloperLogEntry>, String> {
    Ok(LocalServerController::get_developer_logs())
}

#[command]
async fn clear_developer_logs() -> Result<(), String> {
    LocalServerController::clear_developer_logs();
    Ok(())
}

#[command]
async fn clear_server_logs(state: State<'_, AppState>) -> Result<(), String> {
    LocalServerController::clear_logs();
    if let Ok(mut logs) = state.server_logs.try_lock() {
        logs.clear();
    }
    Ok(())
}

#[command]
async fn get_mcp_servers(state: State<'_, AppState>) -> Result<Vec<McpServerConfig>, String> {
    Ok(state.mcp_manager.get_servers().await)
}

#[command]
async fn save_mcp_servers(
    servers: Vec<McpServerConfig>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.mcp_manager.save_servers(servers).await
}

#[command]
async fn inspect_server_tools(
    server: McpServerConfig,
) -> Result<Vec<McpToolDefinition>, String> {
    McpManager::inspect_server_tools(&server).await
}

#[command]
async fn list_all_mcp_tools(state: State<'_, AppState>) -> Result<Vec<McpToolWithServer>, String> {
    Ok(state.mcp_manager.list_all_tools().await)
}

fn heuristic_humanize_tool_name(name: &str, locale: &str) -> String {
    let lower = name.to_lowercase();
    let words: Vec<&str> = lower.split(|c: char| c == '_' || c == '-').filter(|s| !s.is_empty()).collect();
    let mut parts = Vec::new();
    for &w in &words {
        let trans = match locale {
            "en" => match w {
                "id" => "ID",
                "url" => "URL",
                "api" => "API",
                "mcp" => "MCP",
                "csv" => "CSV",
                "pdf" => "PDF",
                "xml" => "XML",
                "json" => "JSON",
                _ => "",
            },
            "es" => match w {
                "get" | "fetch" => "Buscar",
                "list" => "Listar",
                "search" | "find" => "Buscar",
                "query" => "Consultar",
                "read" => "Leer",
                "write" => "Escribir",
                "create" | "add" => "Crear",
                "update" | "modify" => "Actualizar",
                "delete" | "remove" => "Eliminar",
                "send" => "Enviar",
                "check" => "Verificar",
                "overdue" => "Atrasadas",
                "upcoming" => "Próximas",
                "transactions" => "Transacciones",
                "accounts" => "Cuentas",
                "summary" => "Resumen",
                "notes" => "Notas",
                "tasks" => "Tareas",
                "file" => "Archivo",
                "files" => "Archivos",
                "directory" | "dir" => "Directorio",
                "database" | "db" => "Base de Datos",
                "table" => "Tabla",
                "history" => "Historial",
                "all" => "Todos",
                "and" => "y",
                "to" => "a",
                "of" => "de",
                "for" => "para",
                "with" => "con",
                _ => "",
            },
            "zh-CN" | "zh" => match w {
                "get" | "fetch" => "获取",
                "list" => "列出",
                "search" | "find" => "搜索",
                "query" => "查询",
                "read" => "读取",
                "write" => "写入",
                "create" | "add" => "创建",
                "update" | "modify" => "更新",
                "delete" | "remove" => "删除",
                "send" => "发送",
                "check" => "检查",
                "overdue" => "逾期",
                "upcoming" => "即将到来",
                "transactions" => "交易记录",
                "accounts" => "账户",
                "summary" => "摘要",
                "notes" => "笔记",
                "tasks" => "任务",
                "file" => "文件",
                "files" => "文件列表",
                "directory" | "dir" => "目录",
                "database" | "db" => "数据库",
                "table" => "表格",
                "history" => "历史",
                "all" => "全部",
                "and" => "与",
                "to" => "至",
                "of" => "的",
                "for" => "用于",
                "with" => "附带",
                _ => "",
            },
            "ru" => match w {
                "get" | "fetch" => "Получить",
                "list" => "Список",
                "search" | "find" => "Поиск",
                "query" => "Запрос",
                "read" => "Чтение",
                "write" => "Запись",
                "create" | "add" => "Создать",
                "update" | "modify" => "Обновить",
                "delete" | "remove" => "Удалить",
                "send" => "Отправить",
                "check" => "Проверить",
                "overdue" => "Просроченные",
                "upcoming" => "Предстоящие",
                "transactions" => "Транзакции",
                "accounts" => "Счета",
                "summary" => "Сводка",
                "notes" => "Заметки",
                "tasks" => "Задачи",
                "file" => "Файл",
                "files" => "Файлы",
                "directory" | "dir" => "Папка",
                "database" | "db" => "База данных",
                "table" => "Таблица",
                "history" => "История",
                "all" => "Все",
                "and" => "и",
                "to" => "в",
                "of" => "из",
                "for" => "для",
                "with" => "с",
                _ => "",
            },
            _ => match w {
                "get" | "fetch" => "Buscar",
                "list" => "Listar",
                "search" | "find" => "Pesquisar",
                "query" => "Consultar",
                "read" => "Ler",
                "write" => "Gravar",
                "create" | "add" => "Criar",
                "update" | "modify" => "Atualizar",
                "delete" | "remove" => "Excluir",
                "send" => "Enviar",
                "check" => "Verificar",
                "overdue" => "Atrasadas",
                "upcoming" => "Próximas",
                "transactions" => "Transações",
                "accounts" => "Contas",
                "summary" => "Resumo",
                "notes" => "Notas",
                "tasks" => "Tarefas",
                "file" => "Arquivo",
                "files" => "Arquivos",
                "directory" | "dir" => "Diretório",
                "database" | "db" => "Banco",
                "table" => "Tabela",
                "history" => "Histórico",
                "all" => "Todos",
                "and" => "e",
                "to" => "para",
                "of" => "de",
                "for" => "para",
                "with" => "com",
                _ => "",
            },
        };

        if !trans.is_empty() {
            parts.push(trans.to_string());
        } else {
            let mut chars = w.chars();
            let capitalized = match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            };
            parts.push(capitalized);
        }
    }
    if parts.is_empty() {
        name.to_string()
    } else {
        parts.join(" ")
    }
}

fn heuristic_humanize_field_name(name: &str, locale: &str) -> String {
    let lower = name.to_lowercase();
    match locale {
        "en" => match lower.as_str() {
            "upcoming_days" => return "Upcoming Days".to_string(),
            "overdue_days" => return "Overdue Days".to_string(),
            "query" | "q" => return "Search Query".to_string(),
            "limit" => return "Limit".to_string(),
            "offset" => return "Offset".to_string(),
            "page" => return "Page".to_string(),
            "id" | "identifier" => return "ID".to_string(),
            "user_id" => return "User ID".to_string(),
            "start_date" => return "Start Date".to_string(),
            "end_date" => return "End Date".to_string(),
            "date" => return "Date".to_string(),
            "month" => return "Month".to_string(),
            "year" => return "Year".to_string(),
            "amount" => return "Amount".to_string(),
            "currency" => return "Currency".to_string(),
            "description" | "desc" => return "Description".to_string(),
            "title" => return "Title".to_string(),
            "name" => return "Name".to_string(),
            "status" => return "Status".to_string(),
            "category" => return "Category".to_string(),
            "filter" => return "Filter".to_string(),
            "sort" | "order" => return "Sort Order".to_string(),
            "path" => return "Path".to_string(),
            "filepath" | "file_path" => return "File Path".to_string(),
            "filename" | "file_name" => return "File Name".to_string(),
            "content" => return "Content".to_string(),
            "text" => return "Text".to_string(),
            "url" => return "Web URL".to_string(),
            "type" => return "Type".to_string(),
            "enabled" => return "Enabled".to_string(),
            _ => {}
        },
        "es" => match lower.as_str() {
            "upcoming_days" => return "Próximos Días".to_string(),
            "overdue_days" => return "Días en Atraso".to_string(),
            "query" | "q" => return "Término de Búsqueda".to_string(),
            "limit" => return "Límite".to_string(),
            "offset" => return "Desplazamiento".to_string(),
            "page" => return "Página".to_string(),
            "id" | "identifier" => return "Identificador".to_string(),
            "user_id" => return "ID de Usuario".to_string(),
            "start_date" => return "Fecha Inicial".to_string(),
            "end_date" => return "Fecha Final".to_string(),
            "date" => return "Fecha".to_string(),
            "month" => return "Mes".to_string(),
            "year" => return "Año".to_string(),
            "amount" => return "Monto".to_string(),
            "currency" => return "Moneda".to_string(),
            "description" | "desc" => return "Descripción".to_string(),
            "title" => return "Título".to_string(),
            "name" => return "Nombre".to_string(),
            "status" => return "Estado".to_string(),
            "category" => return "Categoría".to_string(),
            "filter" => return "Filtro".to_string(),
            "sort" | "order" => return "Orden".to_string(),
            "path" => return "Ruta".to_string(),
            "filepath" | "file_path" => return "Ruta del Archivo".to_string(),
            "filename" | "file_name" => return "Nombre del Archivo".to_string(),
            "content" => return "Contenido".to_string(),
            "text" => return "Texto".to_string(),
            "url" => return "Dirección Web (URL)".to_string(),
            "type" => return "Tipo".to_string(),
            "enabled" => return "Activado".to_string(),
            _ => {}
        },
        "zh-CN" | "zh" => match lower.as_str() {
            "upcoming_days" => return "未来天数".to_string(),
            "overdue_days" => return "逾期天数".to_string(),
            "query" | "q" => return "搜索词".to_string(),
            "limit" => return "限制数量".to_string(),
            "offset" => return "偏移量".to_string(),
            "page" => return "页码".to_string(),
            "id" | "identifier" => return "标识符 (ID)".to_string(),
            "user_id" => return "用户 ID".to_string(),
            "start_date" => return "起始日期".to_string(),
            "end_date" => return "结束日期".to_string(),
            "date" => return "日期".to_string(),
            "month" => return "月份".to_string(),
            "year" => return "年份".to_string(),
            "amount" => return "金额/数量".to_string(),
            "currency" => return "币种".to_string(),
            "description" | "desc" => return "描述".to_string(),
            "title" => return "标题".to_string(),
            "name" => return "名称".to_string(),
            "status" => return "状态".to_string(),
            "category" => return "分类".to_string(),
            "filter" => return "筛选".to_string(),
            "sort" | "order" => return "排序".to_string(),
            "path" => return "路径".to_string(),
            "filepath" | "file_path" => return "文件路径".to_string(),
            "filename" | "file_name" => return "文件名".to_string(),
            "content" => return "内容".to_string(),
            "text" => return "文本".to_string(),
            "url" => return "网址 (URL)".to_string(),
            "type" => return "类型".to_string(),
            "enabled" => return "是否启用".to_string(),
            _ => {}
        },
        "ru" => match lower.as_str() {
            "upcoming_days" => return "Ближайшие дни".to_string(),
            "overdue_days" => return "Дни просрочки".to_string(),
            "query" | "q" => return "Поисковый запрос".to_string(),
            "limit" => return "Лимит".to_string(),
            "offset" => return "Смещение".to_string(),
            "page" => return "Страница".to_string(),
            "id" | "identifier" => return "Идентификатор".to_string(),
            "user_id" => return "ID пользователя".to_string(),
            "start_date" => return "Начальная дата".to_string(),
            "end_date" => return "Конечная дата".to_string(),
            "date" => return "Дата".to_string(),
            "month" => return "Месяц".to_string(),
            "year" => return "Год".to_string(),
            "amount" => return "Сумма".to_string(),
            "currency" => return "Валюта".to_string(),
            "description" | "desc" => return "Описание".to_string(),
            "title" => return "Заголовок".to_string(),
            "name" => return "Имя".to_string(),
            "status" => return "Статус".to_string(),
            "category" => return "Категория".to_string(),
            "filter" => return "Фильтр".to_string(),
            "sort" | "order" => return "Сортировка".to_string(),
            "path" => return "Путь".to_string(),
            "filepath" | "file_path" => return "Путь к файлу".to_string(),
            "filename" | "file_name" => return "Имя файла".to_string(),
            "content" => return "Содержимое".to_string(),
            "text" => return "Текст".to_string(),
            "url" => return "Веб-ссылка (URL)".to_string(),
            "type" => return "Тип".to_string(),
            "enabled" => return "Включено".to_string(),
            _ => {}
        },
        _ => match lower.as_str() {
            "upcoming_days" => return "Próximos Dias".to_string(),
            "overdue_days" => return "Dias em Atraso".to_string(),
            "query" | "q" => return "Termo de Busca".to_string(),
            "limit" => return "Limite".to_string(),
            "offset" => return "Deslocamento".to_string(),
            "page" => return "Página".to_string(),
            "id" | "identifier" => return "Identificador".to_string(),
            "user_id" => return "ID do Usuário".to_string(),
            "start_date" => return "Data Inicial".to_string(),
            "end_date" => return "Data Final".to_string(),
            "date" => return "Data".to_string(),
            "month" => return "Mês".to_string(),
            "year" => return "Ano".to_string(),
            "amount" => return "Valor".to_string(),
            "currency" => return "Moeda".to_string(),
            "description" | "desc" => return "Descrição".to_string(),
            "title" => return "Título".to_string(),
            "name" => return "Nome".to_string(),
            "status" => return "Status".to_string(),
            "category" => return "Categoria".to_string(),
            "filter" => return "Filtro".to_string(),
            "sort" | "order" => return "Ordenação".to_string(),
            "path" => return "Caminho".to_string(),
            "filepath" | "file_path" => return "Caminho do Arquivo".to_string(),
            "filename" | "file_name" => return "Nome do Arquivo".to_string(),
            "content" => return "Conteúdo".to_string(),
            "text" => return "Texto".to_string(),
            "url" => return "Endereço Web (URL)".to_string(),
            "type" => return "Tipo".to_string(),
            "enabled" => return "Ativado".to_string(),
            _ => {}
        }
    }

    let words: Vec<&str> = lower.split(|c: char| c == '_' || c == '-').filter(|s| !s.is_empty()).collect();
    let mut parts = Vec::new();
    for &w in &words {
        let trans = match locale {
            "en" => match w {
                "id" => "ID",
                "url" => "URL",
                "api" => "API",
                "mcp" => "MCP",
                _ => "",
            },
            "es" => match w {
                "upcoming" => "Próximos",
                "overdue" => "Atraso",
                "days" => "Días",
                "day" => "Día",
                "month" => "Mes",
                "year" => "Año",
                "date" => "Fecha",
                "start" => "Inicio",
                "end" => "Fin",
                "query" => "Búsqueda",
                "limit" => "Límite",
                "offset" => "Desplazamiento",
                "max" => "Máximo",
                "min" => "Mínimo",
                "count" => "Cantidad",
                "file" => "Archivo",
                "path" => "Ruta",
                "type" => "Tipo",
                "id" => "ID",
                "name" => "Nombre",
                "title" => "Título",
                "description" | "desc" => "Descripción",
                "category" => "Categoría",
                "status" => "Estado",
                "account" => "Cuenta",
                "user" => "Usuario",
                "value" => "Valor",
                "amount" => "Monto",
                _ => "",
            },
            _ => match w {
                "upcoming" => "Próximos",
                "overdue" => "Atraso",
                "days" => "Dias",
                "day" => "Dia",
                "month" => "Mês",
                "year" => "Ano",
                "date" => "Data",
                "start" => "Início",
                "end" => "Fim",
                "query" => "Busca",
                "limit" => "Limite",
                "offset" => "Offset",
                "max" => "Máximo",
                "min" => "Mínimo",
                "count" => "Quantidade",
                "file" => "Arquivo",
                "path" => "Caminho",
                "type" => "Tipo",
                "id" => "ID",
                "name" => "Nome",
                "title" => "Título",
                "description" | "desc" => "Descrição",
                "category" => "Categoria",
                "status" => "Status",
                "account" => "Conta",
                "user" => "Usuário",
                "value" => "Valor",
                "amount" => "Quantia",
                _ => "",
            },
        };

        if !trans.is_empty() {
            parts.push(trans.to_string());
        } else {
            let mut chars = w.chars();
            let capitalized = match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            };
            parts.push(capitalized);
        }
    }
    if parts.is_empty() {
        name.to_string()
    } else {
        parts.join(" ")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTranslationResponse {
    pub tool_labels: HashMap<String, String>,
    pub tool_field_labels: HashMap<String, HashMap<String, String>>,
}

static MCP_TRANSLATION_LOCK: Mutex<()> = Mutex::const_new(());

fn fallback_heuristic_response(tools: &[McpToolDefinition], locale: &str) -> McpTranslationResponse {
    let mut tool_labels = HashMap::new();
    let mut tool_field_labels = HashMap::new();
    for t in tools {
        tool_labels.insert(t.name.clone(), heuristic_humanize_tool_name(&t.name, locale));
        let mut fields_map = HashMap::new();
        if let Some(props) = t.input_schema.get("properties").and_then(|p| p.as_object()) {
            for prop_name in props.keys() {
                fields_map.insert(prop_name.clone(), heuristic_humanize_field_name(prop_name, locale));
            }
        }
        tool_field_labels.insert(t.name.clone(), fields_map);
    }
    McpTranslationResponse {
        tool_labels,
        tool_field_labels,
    }
}

#[command]
async fn translate_mcp_tool_labels(
    tools: Vec<McpToolDefinition>,
    mlx_host: String,
    mlx_port: u16,
    ollama_host: String,
    ollama_port: u16,
    target_locale: Option<String>,
    state: State<'_, AppState>,
) -> Result<McpTranslationResponse, String> {
    if tools.is_empty() {
        return Ok(McpTranslationResponse {
            tool_labels: HashMap::new(),
            tool_field_labels: HashMap::new(),
        });
    }

    let locale = target_locale.as_deref().unwrap_or("pt-BR");

    // Trava 1: Previne múltiplas inferências simultâneas com IA que sobrecarreguem a máquina
    let _lock = match MCP_TRANSLATION_LOCK.try_lock() {
        Ok(guard) => guard,
        Err(_) => {
            eprintln!("[MCP Translation] Trava acionada: tradução com IA já em andamento. Aplicando fallback heurístico seguro.");
            return Ok(fallback_heuristic_response(&tools, locale));
        }
    };

    let active_model = {
        let lock = state.active_model.lock().await;
        lock.clone()
    };

    if let Some(model) = active_model {
        // Trava 2: Evitar invocar CLI ou modelo pesado se o daemon não estiver ativo e responsivo
        let is_server_ready = match model.backend {
            BackendType::MlxLm => BackendManager::check_mlx_health(&mlx_host, mlx_port).await,
            BackendType::Ollama => BackendManager::check_ollama_health(&ollama_host, ollama_port).await,
            BackendType::Antigravity | BackendType::CloudOpenAi => true,
            _ => false,
        };

        if !is_server_ready {
            eprintln!("[MCP Translation] Backend ({:?}) não está respondendo via HTTP. Aplicando fallback heurístico para não travar a máquina.", model.backend);
            return Ok(fallback_heuristic_response(&tools, locale));
        }

        // Trava 3: Limitar quantidade de ferramentas enviadas ao LLM para não estourar contexto/RAM
        let max_tools_for_ai = 12;
        let (tools_for_ai, remaining_tools) = if tools.len() > max_tools_for_ai {
            (&tools[..max_tools_for_ai], &tools[max_tools_for_ai..])
        } else {
            (&tools[..], &tools[0..0])
        };

        let mut tool_list_text = String::new();
        for t in tools_for_ai {
            let desc = t.description.as_deref().unwrap_or("Sem descrição");
            tool_list_text.push_str(&format!("- Ferramenta: `{}`\n  Descrição: {}\n", t.name, desc));
            if let Some(props) = t.input_schema.get("properties").and_then(|p| p.as_object()) {
                if !props.is_empty() {
                    tool_list_text.push_str("  Campos/Parâmetros:\n");
                    for (i, (prop_name, prop_val)) in props.iter().enumerate() {
                        if i >= 6 {
                            tool_list_text.push_str("    * ...outros parâmetros omitidos para brevidade\n");
                            break;
                        }
                        let p_desc = prop_val.get("description").and_then(|d| d.as_str()).unwrap_or("");
                        let p_type = prop_val.get("type").and_then(|t| t.as_str()).unwrap_or("any");
                        tool_list_text.push_str(&format!("    * `{}` ({}): {}\n", prop_name, p_type, p_desc));
                    }
                }
            }
        }

        let (system_prompt, user_prompt) = match locale {
            "pt-BR" => (
                "Você é um especialista em interfaces intuitivas e localização em Português do Brasil para o Atena Studio.\n\
                Sua tarefa é criar títulos/rótulos amigáveis, elegantes e concisos em Português do Brasil tanto para as ferramentas técnicas MCP quanto para seus campos/parâmetros (fields).\n\n\
                Regras OBRIGATÓRIAS:\n\
                1. Retorne EXCLUSIVAMENTE um objeto JSON válido.\n\
                2. Para cada ferramenta, forneça:\n\
                   - \"label\": Rótulo amigável da ferramenta (de 2 a 5 palavras, ex: \"Pendências e Próximos Pagamentos\", \"Resumo de Contas\", \"Consultar Transações\").\n\
                   - \"fields\": Objeto mapeando cada nome original de parâmetro para um rótulo amigável em português (ex: \"upcoming_days\": \"Próximos Dias\", \"start_date\": \"Data Inicial\").\n\
                3. Não inclua tags de pensamento (<think>), explicações ou markdown fora do bloco JSON.\n\n\
                Formato esperado:\n\
                {\n\
                  \"nome_da_tool\": {\n\
                    \"label\": \"Rótulo Amigável da Ferramenta\",\n\
                    \"fields\": {\n\
                      \"param1\": \"Rótulo do Campo 1\",\n\
                      \"param2\": \"Rótulo do Campo 2\"\n\
                    }\n\
                  }\n\
                }".to_string(),
                format!(
                    "Traduza as ferramentas técnicas e seus campos para rótulos amigáveis em português:\n\n{}",
                    tool_list_text
                )
            ),
            "es" => (
                "Eres un especialista en interfaces intuitivas y localización en Español para Atena Studio.\n\
                Tu tarea es crear títulos y etiquetas amigables, elegantes y concisos en Español tanto para las herramientas técnicas MCP como para sus campos/parámetros (fields).\n\n\
                Reglas OBLIGATORIAS:\n\
                1. Devuelve EXCLUSIVAMENTE un objeto JSON válido.\n\
                2. Para cada herramienta, proporciona:\n\
                   - \"label\": Etiqueta amigable de la herramienta (de 2 a 5 palabras, ej: \"Pagos Pendientes y Próximos\", \"Resumen de Cuentas\", \"Consultar Transacciones\").\n\
                   - \"fields\": Objeto que mapea cada nombre original de parámetro a una etiqueta amigable en español (ej: \"upcoming_days\": \"Próximos Días\", \"start_date\": \"Fecha Inicial\").\n\
                3. No incluyas etiquetas de pensamiento (<think>), explicaciones ni markdown fuera del bloque JSON.\n\n\
                Formato esperado:\n\
                {\n\
                  \"nombre_tool\": {\n\
                    \"label\": \"Etiqueta Amigable de la Herramienta\",\n\
                    \"fields\": {\n\
                      \"param1\": \"Etiqueta del Campo 1\",\n\
                      \"param2\": \"Etiqueta del Campo 2\"\n\
                    }\n\
                  }\n\
                }".to_string(),
                format!(
                    "Traduce las herramientas técnicas y sus campos a etiquetas amigables en español:\n\n{}",
                    tool_list_text
                )
            ),
            "zh-CN" | "zh" => (
                "您是 Atena Studio 的专业交互界面与中文本地化专家。\n\
                您的任务是为技术性 MCP 工具及其输入参数（fields）生成易读、典雅且简洁的中文友好标题与标签。\n\n\
                强制规则：\n\
                1. 必须且仅返回一个合法的 JSON 对象。\n\
                2. 为每个工具提供：\n\
                   - \"label\": 工具的友好中文名称（2至6个汉字或短语，例如：\"待办与近期账单\"、\"账户概览\"、\"查询交易明细\"）。\n\
                   - \"fields\": 将原参数名映射为友好中文标签的对象（例如：\"upcoming_days\": \"未来天数\", \"start_date\": \"起始日期\"）。\n\
                3. 禁止在 JSON 块外部输出任何思考过程标签 (<think>)、解释或 markdown 文本。\n\n\
                期望输出格式：\n\
                {\n\
                  \"tool_name\": {\n\
                    \"label\": \"工具友好中文名称\",\n\
                    \"fields\": {\n\
                      \"param1\": \"字段标签 1\",\n\
                      \"param2\": \"字段标签 2\"\n\
                    }\n\
                  }\n\
                }".to_string(),
                format!(
                    "请将以下技术工具及其参数字段翻译为友好的中文标签：\n\n{}",
                    tool_list_text
                )
            ),
            "ru" => (
                "Вы — эксперт по пользовательским интерфейсам и локализации на русский язык для Atena Studio.\n\
                Ваша задача — создать понятные, лаконичные и элегантные названия на русском языке для технических инструментов MCP и их параметров (fields).\n\n\
                ОБЯЗАТЕЛЬНЫЕ правила:\n\
                1. Верните ИСКЛЮЧИТЕЛЬНО валидный объект JSON.\n\
                2. Для каждого инструмента укажите:\n\
                   - \"label\": Понятное название инструмента (от 2 до 5 слов, напр. \"Задолженности и предстоящие платежи\", \"Обзор счетов\", \"Запрос транзакций\").\n\
                   - \"fields\": Объект, сопоставляющий имя параметра с понятной меткой на русском языке (напр. \"upcoming_days\": \"Ближайшие дни\", \"start_date\": \"Начальная дата\").\n\
                3. Не включайте теги рассуждений (<think>), пояснений или текста вне блока JSON.\n\n\
                Ожидаемый формат:\n\
                {\n\
                  \"tool_name\": {\n\
                    \"label\": \"Понятное название инструмента\",\n\
                    \"fields\": {\n\
                      \"param1\": \"Метка поля 1\",\n\
                      \"param2\": \"Метка поля 2\"\n\
                    }\n\
                  }\n\
                }".to_string(),
                format!(
                    "Переведите технические инструменты и их параметры в понятные названия на русском языке:\n\n{}",
                    tool_list_text
                )
            ),
            _ => (
                "You are an expert in user interfaces and technical localization for Atena Studio.\n\
                Your task is to create user-friendly, elegant, and concise titles/labels in English for technical MCP tools and their parameter fields.\n\n\
                MANDATORY Rules:\n\
                1. Return EXCLUSIVELY a valid JSON object.\n\
                2. For each tool, provide:\n\
                   - \"label\": Friendly tool title (2 to 5 words, e.g. \"Pending and Upcoming Payments\", \"Account Summary\", \"Query Transactions\").\n\
                   - \"fields\": Object mapping each original parameter name to a user-friendly English label (e.g. \"upcoming_days\": \"Upcoming Days\", \"start_date\": \"Start Date\").\n\
                3. Do not include thinking tags (<think>), explanations, or markdown outside the JSON block.\n\n\
                Expected format:\n\
                {\n\
                  \"tool_name\": {\n\
                    \"label\": \"Friendly Tool Label\",\n\
                    \"fields\": {\n\
                      \"param1\": \"Friendly Field 1\",\n\
                      \"param2\": \"Friendly Field 2\"\n\
                    }\n\
                  }\n\
                }".to_string(),
                format!(
                    "Translate technical tools and their fields into user-friendly English labels:\n\n{}",
                    tool_list_text
                )
            )
        };

        let messages = vec![ChatMessage {
            role: "user".to_string(),
            content: user_prompt,
            images: None,
            tool_calls: None,
            tool_call_id: None,
        }];

        // Trava 4: Configurações ultraleves de inferência para resposta rápida
        let max_tokens = if tools.len() <= 2 { 350 } else { 800 };
        let params = InferenceParams {
            temperature: 0.1,
            top_p: 0.9,
            max_tokens,
            enable_thinking: Some(false),
            thinking_budget: Some(0),
            ..Default::default()
        };

        // Trava 5: Timeout estrito de 10s (para ferramenta única) ou 15s para evitar bloqueio da GPU
        let timeout_secs = if tools.len() <= 2 { 10 } else { 15 };
        let timeout_duration = std::time::Duration::from_secs(timeout_secs);
        let stream_future = BackendManager::execute_stream(
            &model,
            &system_prompt,
            &messages,
            &params,
            &mlx_host,
            mlx_port,
            &ollama_host,
            ollama_port,
        );

        if let Ok(Ok((response, _thinking, _))) = tokio::time::timeout(timeout_duration, stream_future).await {
            let trimmed = response.trim();
            let json_str = if let Some(start) = trimmed.find("```json") {
                let after = &trimmed[start + 7..];
                if let Some(end) = after.find("```") {
                    after[..end].trim()
                } else {
                    after.trim()
                }
            } else if let Some(start) = trimmed.find('{') {
                if let Some(end) = trimmed.rfind('}') {
                    &trimmed[start..=end]
                } else {
                    trimmed
                }
            } else {
                trimmed
            };

            if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
                if let Some(obj) = val.as_object() {
                    let mut tool_labels = HashMap::new();
                    let mut tool_field_labels = HashMap::new();

                    for t in tools_for_ai {
                        if let Some(entry) = obj.get(&t.name) {
                            if let Some(nested_obj) = entry.as_object() {
                                let label = nested_obj.get("label")
                                    .and_then(|l| l.as_str())
                                    .map(|s| s.trim().to_string())
                                    .unwrap_or_else(|| heuristic_humanize_tool_name(&t.name, locale));
                                tool_labels.insert(t.name.clone(), label);

                                let mut fields_map = HashMap::new();
                                if let Some(f_obj) = nested_obj.get("fields").and_then(|f| f.as_object()) {
                                    for (k, v) in f_obj {
                                        if let Some(v_str) = v.as_str() {
                                            fields_map.insert(k.clone(), v_str.trim().to_string());
                                        }
                                    }
                                }
                                if let Some(props) = t.input_schema.get("properties").and_then(|p| p.as_object()) {
                                    for prop_name in props.keys() {
                                        if !fields_map.contains_key(prop_name) {
                                            fields_map.insert(prop_name.clone(), heuristic_humanize_field_name(prop_name, locale));
                                        }
                                    }
                                }
                                tool_field_labels.insert(t.name.clone(), fields_map);
                            } else if let Some(lbl_str) = entry.as_str() {
                                tool_labels.insert(t.name.clone(), lbl_str.trim().to_string());
                                let mut fields_map = HashMap::new();
                                if let Some(props) = t.input_schema.get("properties").and_then(|p| p.as_object()) {
                                    for prop_name in props.keys() {
                                        fields_map.insert(prop_name.clone(), heuristic_humanize_field_name(prop_name, locale));
                                    }
                                }
                                tool_field_labels.insert(t.name.clone(), fields_map);
                            }
                        } else {
                            tool_labels.insert(t.name.clone(), heuristic_humanize_tool_name(&t.name, locale));
                            let mut fields_map = HashMap::new();
                            if let Some(props) = t.input_schema.get("properties").and_then(|p| p.as_object()) {
                                for prop_name in props.keys() {
                                    fields_map.insert(prop_name.clone(), heuristic_humanize_field_name(prop_name, locale));
                                }
                            }
                            tool_field_labels.insert(t.name.clone(), fields_map);
                        }
                    }

                    // Preenche o restante com fallback heurístico amigável
                    for t in remaining_tools {
                        tool_labels.insert(t.name.clone(), heuristic_humanize_tool_name(&t.name, locale));
                        let mut fields_map = HashMap::new();
                        if let Some(props) = t.input_schema.get("properties").and_then(|p| p.as_object()) {
                            for prop_name in props.keys() {
                                fields_map.insert(prop_name.clone(), heuristic_humanize_field_name(prop_name, locale));
                            }
                        }
                        tool_field_labels.insert(t.name.clone(), fields_map);
                    }

                    return Ok(McpTranslationResponse {
                        tool_labels,
                        tool_field_labels,
                    });
                }
            }
        } else {
            eprintln!("[MCP Translation] Timeout ou falha na inferência da LLM. Aplicando fallback heurístico seguro.");
        }
    }

    // Fallback heurístico seguro completo
    Ok(fallback_heuristic_response(&tools, locale))
}

#[command]
async fn update_mcp_tool_label(
    server_id: String,
    tool_name: String,
    label: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut servers = state.mcp_manager.get_servers().await;
    if let Some(server) = servers.iter_mut().find(|s| s.id == server_id) {
        if label.trim().is_empty() {
            server.tool_labels.remove(&tool_name);
        } else {
            server.tool_labels.insert(tool_name, label.trim().to_string());
        }
        state.mcp_manager.save_servers(servers).await?;
        Ok(())
    } else {
        Err(format!("Servidor MCP '{}' não encontrado", server_id))
    }
}

#[command]
async fn update_mcp_tool_details(
    server_id: String,
    tool_name: String,
    label: Option<String>,
    field_labels: Option<HashMap<String, String>>,
    enabled: Option<bool>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut servers = state.mcp_manager.get_servers().await;
    if let Some(server) = servers.iter_mut().find(|s| s.id == server_id) {
        if let Some(lbl) = label {
            if lbl.trim().is_empty() {
                server.tool_labels.remove(&tool_name);
            } else {
                server.tool_labels.insert(tool_name.clone(), lbl.trim().to_string());
            }
        }
        if let Some(fields) = field_labels {
            if fields.is_empty() {
                server.tool_field_labels.remove(&tool_name);
            } else {
                server.tool_field_labels.insert(tool_name.clone(), fields);
            }
        }
        if let Some(is_active) = enabled {
            if is_active {
                server.disabled_tools.retain(|x| x != &tool_name);
            } else if !server.disabled_tools.contains(&tool_name) {
                server.disabled_tools.push(tool_name);
            }
        }
        state.mcp_manager.save_servers(servers).await?;
        Ok(())
    } else {
        Err(format!("Servidor MCP '{}' não encontrado", server_id))
    }
}

#[command]
async fn call_mcp_tool(
    server_id: String,
    tool_name: String,
    arguments: serde_json::Value,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    execute_tool_call_internal(state.inner(), &server_id, &tool_name, arguments).await
}

pub async fn execute_tool_call_internal(
    state: &AppState,
    server_id: &str,
    tool_name: &str,
    arguments: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let effective_server = if server_id.is_empty() {
        if tool_name.starts_with("atena_") {
            "atena_native"
        } else {
            "skills"
        }
    } else {
        server_id
    };

    let servers = state.mcp_manager.get_servers().await;
    if let Some(server) = servers.iter().find(|s| s.id == effective_server) {
        if !server.enabled {
            return Err(format!("Tool server '{}' is disabled.", effective_server));
        }
        if server.disabled_tools.contains(&tool_name.to_string()) {
            return Err(format!("Tool '{}' is disabled in server settings.", tool_name));
        }
    }

    if effective_server == "atena_native" || effective_server == "atena" {
        match tool_name {
            "atena_search_episodes" => {
                let query = arguments.get("query").and_then(|v| v.as_str()).unwrap_or_default();
                let limit = arguments.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
                let results = MemoryGraphEngine::search_episodes(query, None, limit);
                return Ok(serde_json::to_value(&results).unwrap_or(serde_json::json!([])));
            }
            "atena_read_episode" => {
                let identifier = arguments.get("identifier").and_then(|v| v.as_str())
                    .or_else(|| arguments.get("episode_id").and_then(|v| v.as_str()))
                    .or_else(|| arguments.get("id").and_then(|v| v.as_str()))
                    .unwrap_or_default();
                if let Some(detail) = MemoryGraphEngine::get_episode_detail(identifier) {
                    return Ok(serde_json::to_value(&detail).unwrap_or(serde_json::json!({})));
                } else {
                    return Ok(serde_json::json!({ "error": format!("Episódio '{}' não encontrado.", identifier) }));
                }
            }
            "atena_search_memory" => {
                let query = arguments.get("query").and_then(|v| v.as_str()).unwrap_or_default();
                let mut engine = state.memory_engine.lock().await;
                let res = engine.search_active_context_for_query(query);
                return Ok(serde_json::json!({ "result": res.unwrap_or_else(|| "Nenhum registro encontrado na memória para esta consulta.".to_string()) }));
            }
            "atena_web_search" => {
                let query = arguments.get("query").and_then(|v| v.as_str()).unwrap_or_default();
                let max_results = arguments.get("max_results").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
                let results = crate::services::web_tools::WebTools::search(query, max_results).await?;
                return Ok(serde_json::to_value(&results).unwrap_or(serde_json::json!([])));
            }
            "atena_fetch_webpage" => {
                let url = arguments.get("url").and_then(|v| v.as_str()).unwrap_or_default();
                let max_chars = arguments.get("max_characters").and_then(|v| v.as_u64()).map(|n| n as usize);
                let content = crate::services::web_tools::WebTools::fetch_webpage(url, max_chars).await?;
                return Ok(serde_json::json!({ "url": url, "content": content }));
            }
            "atena_scratchpad_write" => {
                let key = arguments.get("key").and_then(|v| v.as_str()).unwrap_or("default");
                let content = arguments.get("content").and_then(|v| v.as_str()).unwrap_or_default();
                let session_id = arguments.get("session_id").and_then(|v| v.as_str()).unwrap_or("active_session");
                state.scratchpad.write(session_id, key, content).await;
                return Ok(serde_json::json!({ "success": true, "key": key, "message": "Note recorded in task scratchpad." }));
            }
            "atena_scratchpad_read" => {
                let session_id = arguments.get("session_id").and_then(|v| v.as_str()).unwrap_or("active_session");
                if let Some(key) = arguments.get("key").and_then(|v| v.as_str()) {
                    let val = state.scratchpad.read_key(session_id, key).await;
                    return Ok(serde_json::json!({ "key": key, "content": val }));
                } else {
                    let all = state.scratchpad.read_all(session_id).await;
                    return Ok(serde_json::to_value(&all).unwrap_or(serde_json::json!({})));
                }
            }
            "atena_scratchpad_clear" => {
                let session_id = arguments.get("session_id").and_then(|v| v.as_str()).unwrap_or("active_session");
                state.scratchpad.clear(session_id).await;
                return Ok(serde_json::json!({ "success": true, "message": "Task scratchpad cleared." }));
            }
            "atena_schedule_task" => {
                let name = arguments.get("name").and_then(|v| v.as_str()).unwrap_or_default();
                let cron_expr = arguments.get("cron_expr").and_then(|v| v.as_str()).unwrap_or("@daily");
                let action_type = arguments.get("action_type").and_then(|v| v.as_str()).unwrap_or("autonomous_prompt");
                let description = arguments.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
                let prompt = arguments.get("prompt").and_then(|v| v.as_str()).unwrap_or_default();

                let skill_slug = arguments.get("skill_slug")
                    .or_else(|| arguments.get("slug"))
                    .or_else(|| arguments.get("skill_id"))
                    .or_else(|| arguments.get("skillId"))
                    .and_then(|v| v.as_str());
                let script_file = arguments.get("script_file")
                    .or_else(|| arguments.get("script"))
                    .and_then(|v| v.as_str());
                let tool_name = arguments.get("tool_name").and_then(|v| v.as_str());
                let raw_args = arguments.get("arguments").or_else(|| arguments.get("args")).cloned();

                let payload = if action_type == "autonomous_prompt" {
                    serde_json::json!({ "prompt": prompt }).to_string()
                } else if action_type == "skill" {
                    let mut p_obj = serde_json::json!({
                        "prompt": prompt,
                    });
                    if let Some(slug) = skill_slug {
                        p_obj["skill_slug"] = serde_json::Value::String(slug.to_string());
                    }
                    if let Some(script) = script_file {
                        p_obj["script_file"] = serde_json::Value::String(script.to_string());
                    }
                    if let Some(t_name) = tool_name {
                        p_obj["tool_name"] = serde_json::Value::String(t_name.to_string());
                    }
                    if let Some(a) = raw_args {
                        p_obj["arguments"] = a;
                    }
                    p_obj.to_string()
                } else {
                    serde_json::json!({}).to_string()
                };

                let delivery_channel = arguments.get("delivery_channel").and_then(|v| v.as_str()).unwrap_or("chat");
                let delivery_target = arguments.get("delivery_target").and_then(|v| v.as_str()).map(|s| s.to_string());

                let task_id = format!("task-{}", chrono::Utc::now().timestamp_millis());
                let now = chrono::Utc::now().to_rfc3339();
                let task = crate::services::db::DbScheduledTask {
                    id: task_id.clone(),
                    name: name.to_string(),
                    description,
                    cron_expr: cron_expr.to_string(),
                    action_type: action_type.to_string(),
                    payload,
                    enabled: true,
                    last_run: None,
                    next_run: crate::services::scheduler::BackgroundScheduler::compute_next_run(cron_expr, chrono::Utc::now()).map(|dt| dt.to_rfc3339()),
                    delivery_channel: delivery_channel.to_string(),
                    delivery_target,
                    last_result: None,
                    created_at: now.clone(),
                    updated_at: now,
                };
                state.db.save_scheduled_task(&task)?;

                let run_immediately = arguments.get("run_immediately").and_then(|v| v.as_bool()).unwrap_or(false);
                let run_result = if run_immediately {
                    let (success, out) = Box::pin(crate::services::scheduler::BackgroundScheduler::execute_task_direct(state, &task)).await;
                    Some(serde_json::json!({
                        "executed": true,
                        "success": success,
                        "output": out
                    }))
                } else {
                    None
                };

                return Ok(serde_json::json!({
                    "success": true,
                    "task_id": task_id,
                    "message": format!("Routine '{}' successfully scheduled with schedule '{}'.", name, cron_expr),
                    "immediate_execution": run_result
                }));
            }
            "atena_list_scheduled_tasks" => {
                let tasks = state.db.get_scheduled_tasks()?;
                let summary: Vec<serde_json::Value> = tasks.into_iter().map(|t| {
                    serde_json::json!({
                        "id": t.id,
                        "name": t.name,
                        "cron_expr": t.cron_expr,
                        "action_type": t.action_type,
                        "enabled": t.enabled,
                        "last_run": t.last_run,
                        "next_run": t.next_run
                    })
                }).collect();
                return Ok(serde_json::json!(summary));
            }
            "atena_cancel_scheduled_task" => {
                let identifier = arguments.get("identifier").and_then(|v| v.as_str()).unwrap_or_default();
                let tasks = state.db.get_scheduled_tasks()?;
                if let Some(target) = tasks.into_iter().find(|t| t.id == identifier || t.name.eq_ignore_ascii_case(identifier) || t.name.to_lowercase().contains(&identifier.to_lowercase())) {
                    state.db.delete_scheduled_task(&target.id)?;
                    return Ok(serde_json::json!({
                        "success": true,
                        "message": format!("Routine '{}' ({}) successfully cancelled.", target.name, target.id)
                    }));
                } else {
                    return Err(format!("Routine with identifier '{}' not found.", identifier));
                }
            }
            "atena_run_scheduled_task" => {
                let identifier = arguments.get("identifier").and_then(|v| v.as_str()).unwrap_or_default();
                let tasks = state.db.get_scheduled_tasks()?;
                if let Some(target) = tasks.into_iter().find(|t| t.id == identifier || t.name.eq_ignore_ascii_case(identifier) || t.name.to_lowercase().contains(&identifier.to_lowercase())) {
                    let (success, output) = Box::pin(crate::services::scheduler::BackgroundScheduler::execute_task_direct(state, &target)).await;
                    return Ok(serde_json::json!({
                        "success": success,
                        "task_id": target.id,
                        "task_name": target.name,
                        "output": output,
                        "message": if success {
                            format!("Routine '{}' executed successfully.", target.name)
                        } else {
                            format!("Routine '{}' finished with error: {}", target.name, output)
                        }
                    }));
                } else {
                    return Err(format!("Routine with identifier '{}' not found.", identifier));
                }
            }
            _ => return Err(format!("Ferramenta nativa '{}' desconhecida.", tool_name)),
        }
    }
    if effective_server == "skills" || tool_name == "run_command" || tool_name == "run_skill_command" || tool_name == "run_skill_script" || tool_name == "create_procedural_skill" || tool_name == "update_procedural_skill" || tool_name == "edit_procedural_skill" {
        let app_cfg = crate::core::config::AppConfig::load();
        if !app_cfg.enable_cognitive_memory || !app_cfg.enable_skills_memory {
            return Err("Procedural skills and terminal automation are currently disabled in settings.".to_string());
        }
        match tool_name {
            "create_procedural_skill" => {
                let name = arguments.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                let description = arguments.get("description").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                let triggers: Vec<String> = serde_json::from_value(arguments.get("triggers").cloned().unwrap_or(serde_json::json!([]))).unwrap_or_default();
                let steps = serde_json::from_value(arguments.get("steps").cloned().unwrap_or(serde_json::json!([]))).unwrap_or_default();
                let scripts = serde_json::from_value(arguments.get("scripts").cloned().unwrap_or(serde_json::json!([]))).unwrap_or_default();
                let env_vars = serde_json::from_value(arguments.get("env_vars").or_else(|| arguments.get("envVars")).cloned().unwrap_or(serde_json::json!(null))).ok();
                let skill = MemoryGraphEngine::create_skill_with_scripts(&name, &description, triggers, steps, scripts, env_vars)?;
                return Ok(serde_json::to_value(&skill).unwrap_or(serde_json::json!({})));
            }
            "update_procedural_skill" | "edit_procedural_skill" => {
                let id = arguments.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
                let name = arguments.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
                let description = arguments.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
                let triggers: Option<Vec<String>> = serde_json::from_value(arguments.get("triggers").cloned().unwrap_or(serde_json::json!(null))).ok();
                let steps: Option<Vec<SkillStep>> = serde_json::from_value(arguments.get("steps").cloned().unwrap_or(serde_json::json!(null))).ok();
                let scripts: Option<Vec<SkillScriptFilePayload>> = serde_json::from_value(arguments.get("scripts").cloned().unwrap_or(serde_json::json!(null))).ok();
                let refinement_note = arguments.get("refinement_note").or_else(|| arguments.get("refinementNote")).and_then(|v| v.as_str()).map(|s| s.to_string());
                let env_vars = serde_json::from_value(arguments.get("env_vars").or_else(|| arguments.get("envVars")).cloned().unwrap_or(serde_json::json!(null))).ok();
                let permission_mode = arguments.get("permission_mode").or_else(|| arguments.get("permissionMode")).and_then(|v| v.as_str()).map(|s| s.to_string());
                let enabled = arguments.get("enabled").and_then(|v| v.as_bool());
                let skill = MemoryGraphEngine::update_skill_with_scripts(id, name, description, triggers, steps, scripts, refinement_note, env_vars, permission_mode, enabled)?;
                return Ok(serde_json::to_value(&skill).unwrap_or(serde_json::json!({})));
            }
            "run_command" | "run_skill_command" => {
                let command = arguments.get("command").or_else(|| arguments.get("cmd")).and_then(|v| v.as_str()).unwrap_or_default().to_string();
                let timeout_ms = arguments.get("timeout_ms").or_else(|| arguments.get("timeoutMs")).and_then(|v| v.as_u64());
                let res = crate::services::skill_runner::SkillScriptRunner::run_command(&command, None, None, None, timeout_ms).await?;
                return Ok(serde_json::to_value(&res).unwrap_or(serde_json::json!({})));
            }
            "run_skill_script" => {
                let slug = arguments.get("slug")
                    .or_else(|| arguments.get("skill_id"))
                    .or_else(|| arguments.get("skillId"))
                    .or_else(|| arguments.get("id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let script_file = arguments.get("script_file")
                    .or_else(|| arguments.get("script"))
                    .or_else(|| arguments.get("script_name"))
                    .or_else(|| arguments.get("scriptName"))
                    .or_else(|| arguments.get("file_name"))
                    .or_else(|| arguments.get("fileName"))
                    .or_else(|| arguments.get("filename"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let args: Vec<String> = serde_json::from_value(arguments.get("args").cloned().unwrap_or(serde_json::json!([]))).unwrap_or_default();
                let timeout_ms = arguments.get("timeout_ms").or_else(|| arguments.get("timeoutMs")).and_then(|v| v.as_u64());
                let skills = MemoryGraphEngine::load_skills();
                let skill = skills.into_iter().find(|s| {
                    s.id == slug
                        || s.id == format!("skill-{}", slug)
                        || slug == format!("skill-{}", s.id)
                        || s.name.eq_ignore_ascii_case(slug)
                        || s.id.trim_start_matches("skill-") == slug.trim_start_matches("skill-")
                }).ok_or_else(|| format!("Skill '{}' não encontrada", slug))?;
                if !skill.enabled {
                    return Err(format!("A habilidade procedural '{}' está desabilitada.", skill.name));
                }
                let folder = skill.folder_path.as_ref().ok_or_else(|| "Pasta da skill não configurada".to_string())?;
                let skill_dir = std::path::PathBuf::from(folder);
                let clean_script = script_file.trim_start_matches('/');
                let script_rel_path = if clean_script.starts_with("scripts/") {
                    clean_script.to_string()
                } else {
                    format!("scripts/{}", clean_script)
                };
                let res = crate::services::skill_runner::SkillScriptRunner::run_script(&skill_dir, &script_rel_path, &args, None, skill.env_vars.as_ref(), timeout_ms).await?;
                return Ok(serde_json::to_value(&res).unwrap_or(serde_json::json!({})));
            }
            _ => return Err(format!("Ferramenta de procedural skill '{}' desconhecida.", tool_name)),
        }
    }
    state.mcp_manager.call_tool(effective_server, tool_name, arguments).await
}

#[command]
async fn read_file_attachment(file_path: String) -> Result<serde_json::Value, String> {
    let path = std::path::Path::new(&file_path);
    if !path.exists() {
        return Err("Arquivo não encontrado".to_string());
    }

    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("arquivo").to_string();
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let is_image = matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp" | "gif" | "svg");

    let bytes = std::fs::read(path).map_err(|e| format!("Erro ao ler arquivo: {}", e))?;
    let size_bytes = bytes.len();

    if is_image {
        let b64 = BackendManager::encode_base64(&bytes);
        let mime = match ext.as_str() {
            "png" => "image/png",
            "webp" => "image/webp",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            _ => "image/jpeg",
        };
        let data_url = format!("data:{};base64,{}", mime, b64);
        Ok(serde_json::json!({
            "name": file_name,
            "is_image": true,
            "data_url": data_url,
            "size_bytes": size_bytes
        }))
    } else {
        // Read text / code
        let text = String::from_utf8_lossy(&bytes).to_string();
        Ok(serde_json::json!({
            "name": file_name,
            "is_image": false,
            "text": text,
            "size_bytes": size_bytes
        }))
    }
}

/// Decode base64 string to bytes
fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
    const TABLE: [u8; 128] = {
        let mut t = [255u8; 128];
        let mut i = 0u8;
        while i < 26 {
            t[(b'A' + i) as usize] = i;
            t[(b'a' + i) as usize] = i + 26;
            i += 1;
        }
        let mut d = 0u8;
        while d < 10 {
            t[(b'0' + d) as usize] = d + 52;
            d += 1;
        }
        t[b'+' as usize] = 62;
        t[b'/' as usize] = 63;
        t
    };

    let input = input.trim();
    let bytes: Vec<u8> = input.bytes().filter(|&b| b != b'\n' && b != b'\r' && b != b' ').collect();
    let mut out = Vec::with_capacity(bytes.len() * 3 / 4);
    let mut i = 0;
    while i + 3 < bytes.len() {
        let (a, b, c, d) = (bytes[i], bytes[i+1], bytes[i+2], bytes[i+3]);
        let va = *TABLE.get(a as usize).unwrap_or(&255);
        let vb = *TABLE.get(b as usize).unwrap_or(&255);
        let vc = if c == b'=' { 0 } else { *TABLE.get(c as usize).unwrap_or(&255) };
        let vd = if d == b'=' { 0 } else { *TABLE.get(d as usize).unwrap_or(&255) };
        if va == 255 || vb == 255 || (c != b'=' && vc == 255) || (d != b'=' && vd == 255) {
            return Err("Base64 inválido".to_string());
        }
        out.push((va << 2) | (vb >> 4));
        if c != b'=' { out.push((vb << 4) | (vc >> 2)); }
        if d != b'=' { out.push((vc << 6) | vd); }
        i += 4;
    }
    Ok(out)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioPcmResult {
    pub samples: Vec<f32>,
    pub duration_secs: f64,
}

#[command]
async fn convert_audio_to_pcm(audio_base64: String) -> Result<AudioPcmResult, String> {
    let audio_bytes = decode_base64(&audio_base64)?;

    let tmp_dir = std::env::temp_dir().join("atena_audio");
    std::fs::create_dir_all(&tmp_dir).map_err(|e| format!("Erro ao criar diretório temporário: {}", e))?;

    let input_path = tmp_dir.join(format!("input_{}.audio", std::process::id()));
    let output_path = tmp_dir.join(format!("output_{}.raw", std::process::id()));

    // Write input audio bytes to temp file
    {
        let mut f = std::fs::File::create(&input_path)
            .map_err(|e| format!("Erro ao criar arquivo temporário: {}", e))?;
        f.write_all(&audio_bytes)
            .map_err(|e| format!("Erro ao escrever arquivo de áudio: {}", e))?;
    }

    let aug_path = BackendManager::augmented_path();
    let ffmpeg_bin = RuntimeManager::resolve_binary("ffmpeg")
        .unwrap_or_else(|| std::path::PathBuf::from("ffmpeg"));

    // Run FFmpeg to convert to raw PCM float32, 16kHz, mono
    let ffmpeg_result = silent_tokio_command(&ffmpeg_bin)
        .env("PATH", &aug_path)
        .arg("-y")
        .arg("-i")
        .arg(input_path.to_str().unwrap_or_default())
        .arg("-ar")
        .arg("16000")
        .arg("-ac")
        .arg("1")
        .arg("-f")
        .arg("f32le")
        .arg(output_path.to_str().unwrap_or_default())
        .output()
        .await;

    // Clean up input file
    let _ = std::fs::remove_file(&input_path);

    match ffmpeg_result {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let _ = std::fs::remove_file(&output_path);
                return Err(format!(
                    "FFmpeg falhou ao converter áudio. Verifique os binários embutidos. Erro: {}",
                    stderr.chars().take(300).collect::<String>()
                ));
            }

            // Read raw PCM float32 output
            let raw_bytes = std::fs::read(&output_path)
                .map_err(|e| format!("Erro ao ler áudio convertido: {}", e))?;
            let _ = std::fs::remove_file(&output_path);

            if raw_bytes.len() < 4 {
                return Err("Áudio convertido está vazio ou muito curto.".to_string());
            }

            // Convert bytes to f32 samples (little-endian)
            let sample_count = raw_bytes.len() / 4;
            let mut samples = Vec::with_capacity(sample_count);
            for chunk in raw_bytes.chunks_exact(4) {
                let val = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                samples.push(val);
            }

            let duration_secs = sample_count as f64 / 16000.0;

            Ok(AudioPcmResult {
                samples,
                duration_secs,
            })
        }
        Err(e) => {
            let _ = std::fs::remove_file(&output_path);
            Err(format!(
                "FFmpeg não encontrado. Erro: {}",
                e
            ))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTranscriptionResult {
    pub text: String,
    pub duration_secs: Option<f64>,
}

#[command]
async fn transcribe_audio(
    audio_base64: String,
    file_name: Option<String>,
    language: Option<String>,
    model: Option<String>,
) -> Result<AudioTranscriptionResult, String> {
    let audio_bytes = decode_base64(&audio_base64)?;

    let tmp_dir = std::env::temp_dir().join("atena_audio");
    std::fs::create_dir_all(&tmp_dir).map_err(|e| format!("Erro ao criar diretório temporário: {}", e))?;

    let ext = file_name
        .as_deref()
        .and_then(|n| std::path::Path::new(n).extension())
        .and_then(|e| e.to_str())
        .unwrap_or("audio");

    let unique_id = format!(
        "{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );
    let input_path = tmp_dir.join(format!("audio_{}.{}", unique_id, ext));

    // Write input audio bytes to temp file
    {
        let mut f = std::fs::File::create(&input_path)
            .map_err(|e| format!("Erro ao criar arquivo temporário: {}", e))?;
        f.write_all(&audio_bytes)
            .map_err(|e| format!("Erro ao escrever arquivo de áudio: {}", e))?;
    }

    let aug_path = BackendManager::augmented_path();
    let input_str = input_path.to_string_lossy().to_string();
    let lang_str = language.as_deref().unwrap_or("auto");
    let model_str = model.as_deref().unwrap_or("mlx-community/whisper-small-mlx");

    // Cross-platform Python script to run Whisper:
    // 1. Tries mlx_whisper (macOS Apple Silicon Metal GPU)
    // 2. Tries faster_whisper (Windows CUDA / CPU, Linux)
    // 3. Tries standard openai-whisper (Windows / Linux)
    let py_script = r#"
import sys, json

try:
    audio_path = sys.argv[1]
    lang = sys.argv[2] if len(sys.argv) > 2 and sys.argv[2] not in ("", "null", "auto", "None") else None
    model_name = sys.argv[3] if len(sys.argv) > 3 and sys.argv[3] not in ("", "null", "auto", "None") else "mlx-community/whisper-small-mlx"

    decode_kwargs = {
        "condition_on_previous_text": False,
        "temperature": (0.0, 0.2, 0.4),
        "no_speech_threshold": 0.8,
        "logprob_threshold": -1.5,
    }

    if lang:
        decode_kwargs["language"] = lang
    else:
        decode_kwargs["initial_prompt"] = "Transcrição de áudio em português com pontuação e acentuação adequadas."

    # 1. Try Apple Silicon MLX Whisper
    try:
        import mlx_whisper
        mlx_model = model_name
        if mlx_model in ("small", "small-mlx"):
            mlx_model = "mlx-community/whisper-small-mlx"
        elif mlx_model in ("base", "base-mlx"):
            mlx_model = "mlx-community/whisper-base-mlx"
        elif mlx_model in ("turbo", "large-turbo", "large-v3-turbo"):
            mlx_model = "mlx-community/whisper-large-v3-turbo"
        elif mlx_model in ("tiny", "tiny-mlx"):
            mlx_model = "mlx-community/whisper-tiny"

        result = mlx_whisper.transcribe(audio_path, path_or_hf_repo=mlx_model, **decode_kwargs)
        text = (result.get("text") or "").strip()
        segments = result.get("segments") or []
        duration = segments[-1].get("end") if segments else None
        print(json.dumps({"text": text, "duration": duration, "language": result.get("language")}))
        sys.exit(0)
    except ImportError:
        pass

    # 2. Try faster-whisper (Cross-platform: Windows CUDA/CPU, Linux)
    try:
        from faster_whisper import WhisperModel
        fw_size = "small" if "small" in model_name else ("base" if "base" in model_name else ("large-v3" if "turbo" in model_name or "large" in model_name else "tiny"))
        model = WhisperModel(fw_size, device="auto", compute_type="default")
        prompt = decode_kwargs.get("initial_prompt") if not lang else None
        segments, info = model.transcribe(audio_path, language=lang, initial_prompt=prompt, beam_size=5)
        text = " ".join([s.text for s in segments]).strip()
        print(json.dumps({"text": text, "duration": info.duration, "language": info.language}))
        sys.exit(0)
    except ImportError:
        pass

    # 3. Try standard openai whisper (Windows / Linux)
    try:
        import whisper
        w_size = "small" if "small" in model_name else ("base" if "base" in model_name else ("large" if "turbo" in model_name or "large" in model_name else "tiny"))
        w_model = whisper.load_model(w_size)
        res = w_model.transcribe(audio_path, **decode_kwargs)
        print(json.dumps({"text": res.get("text", "").strip(), "duration": None, "language": res.get("language")}))
        sys.exit(0)
    except ImportError:
        print(json.dumps({"error": "Nenhum mecanismo Whisper (mlx-whisper, faster-whisper ou whisper) está instalado no ambiente Python local."}))
except Exception as e:
    print(json.dumps({"error": str(e)}))
"#;

    let output_res = if let Some(py_path) = RuntimeManager::isolated_python() {
        silent_tokio_command(&py_path)
            .env("PATH", &aug_path)
            .arg("-c")
            .arg(py_script)
            .arg(&input_str)
            .arg(lang_str)
            .arg(model_str)
            .output()
            .await
    } else if let Some(uv_path) = RuntimeManager::resolve_binary("uv") {
        silent_tokio_command(&uv_path)
            .env("PATH", &aug_path)
            .arg("run")
            .arg("--with")
            .arg("mlx-whisper")
            .arg("python3")
            .arg("-c")
            .arg(py_script)
            .arg(&input_str)
            .arg(lang_str)
            .arg(model_str)
            .output()
            .await
    } else if let Some(py_bin) = RuntimeManager::get_python() {
        silent_tokio_command(&py_bin)
            .env("PATH", &aug_path)
            .arg("-c")
            .arg(py_script)
            .arg(&input_str)
            .arg(lang_str)
            .arg(model_str)
            .output()
            .await
    } else {
        silent_tokio_command(if cfg!(target_os = "windows") { "python" } else { "python3" })
            .env("PATH", &aug_path)
            .arg("-c")
            .arg(py_script)
            .arg(&input_str)
            .arg(lang_str)
            .arg(model_str)
            .output()
            .await
    };

    let _ = std::fs::remove_file(&input_path);

    match output_res {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            // Find JSON line in stdout (in case uv prints extra info)
            let json_line = stdout.lines().rev().find(|l| l.starts_with('{') && l.ends_with('}')).unwrap_or(&stdout);
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_line) {
                if let Some(err) = val.get("error").and_then(|e| e.as_str()) {
                    return Err(format!("Erro MLX Whisper: {}", err));
                }
                let text = val.get("text").and_then(|t| t.as_str()).unwrap_or("").trim().to_string();
                let duration_secs = val.get("duration").and_then(|d| d.as_f64());
                return Ok(AudioTranscriptionResult {
                    text: if text.is_empty() { "[Nenhuma fala detectada no arquivo de áudio]".to_string() } else { text },
                    duration_secs,
                });
            }
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Falha na transcrição: {}", stderr.chars().take(200).collect::<String>()))
        }
        Err(e) => Err(format!("Erro ao executar transcritor Whisper: {}", e)),
    }
}

#[command]
async fn get_runtime_status(state: State<'_, AppState>) -> Result<RuntimeStatus, String> {
    Ok(state.runtime_manager.get_status().await)
}

#[command]
async fn bootstrap_mlx_runtime(
    channel: Channel<BootstrapProgress>,
    state: State<'_, AppState>,
    from_github: Option<bool>,
) -> Result<(), String> {
    let mgr = state.runtime_manager.clone();
    let github = from_github.unwrap_or(false);
    mgr.bootstrap_mlx_runtime(github, move |prog| {
        let _ = channel.send(prog);
    })
    .await
}

#[command]
async fn update_mlx_packages(
    channel: Channel<BootstrapProgress>,
    state: State<'_, AppState>,
    from_github: Option<bool>,
) -> Result<(), String> {
    let mgr = state.runtime_manager.clone();
    let github = from_github.unwrap_or(false);
    mgr.update_mlx_packages(github, move |prog| {
        let _ = channel.send(prog);
    })
    .await
}

#[command]
async fn bootstrap_llama_runtime(
    channel: Channel<BootstrapProgress>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mgr = state.runtime_manager.clone();
    mgr.bootstrap_llama_runtime(move |prog| {
        let _ = channel.send(prog);
    })
    .await
}

#[command]
async fn bootstrap_ffmpeg_runtime(
    channel: Channel<BootstrapProgress>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mgr = state.runtime_manager.clone();
    mgr.bootstrap_ffmpeg_runtime(move |prog| {
        let _ = channel.send(prog);
    })
    .await
}

#[command]
async fn bootstrap_all_runtimes(
    channel: Channel<BootstrapProgress>,
    state: State<'_, AppState>,
    from_github: Option<bool>,
) -> Result<(), String> {
    let mgr = state.runtime_manager.clone();
    let github = from_github.unwrap_or(false);
    mgr.bootstrap_all_runtimes(github, move |prog| {
        let _ = channel.send(prog);
    })
    .await
}

// =============================================================================
// Comandos de Memória Associativa
// =============================================================================

#[command]
async fn memory_add_node(
    type_flag: u8,
    label: String,
    state: State<'_, AppState>,
) -> Result<u32, String> {
    let node_type = NodeType::from_byte(type_flag)
        .ok_or_else(|| format!("Tipo de nó inválido: 0x{:02X}", type_flag))?;
    let mut engine = state.memory_engine.lock().await;
    Ok(engine.add_node(node_type, &label))
}

#[command]
async fn memory_add_edge(
    source_id: u32,
    target_id: u32,
    relation_type: u8,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let rel = RelationType::from_byte(relation_type)
        .ok_or_else(|| format!("Tipo de relação inválido: 0x{:02X}", relation_type))?;
    let mut engine = state.memory_engine.lock().await;
    engine.add_edge(source_id, target_id, rel)
}

#[command]
async fn memory_query(
    start_label: String,
    max_depth: Option<usize>,
    state: State<'_, AppState>,
) -> Result<MemoryQueryResult, String> {
    let mut engine = state.memory_engine.lock().await;
    let depth = max_depth.unwrap_or(3);
    let paths = engine.traverse_associations(&start_label, depth)?;
    let llm_context = MemoryGraphEngine::build_llm_context(&paths);
    let stats = engine.stats();
    Ok(MemoryQueryResult {
        llm_context,
        paths,
        stats,
    })
}

#[command]
async fn memory_reinforce(
    source_id: u32,
    target_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut engine = state.memory_engine.lock().await;
    engine.reinforce_edge(source_id, target_id)
}

#[command]
async fn memory_apply_decay(
    decay_factor: Option<f32>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut engine = state.memory_engine.lock().await;
    engine.apply_decay(decay_factor.unwrap_or(0.05));
    Ok(())
}

#[command]
async fn memory_save(
    filepath: String,
    compression: Option<u8>,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let engine = state.memory_engine.lock().await;
    let comp = compression
        .and_then(CompressionType::from_byte)
        .unwrap_or(CompressionType::Zstd);
    engine.save_to_compressed_binary(std::path::Path::new(&filepath), comp)
}

#[command]
async fn memory_load(
    filepath: String,
    state: State<'_, AppState>,
) -> Result<EngineStats, String> {
    let new_engine =
        MemoryGraphEngine::load_from_compressed_binary(std::path::Path::new(&filepath), 128)?;
    let stats = new_engine.stats();
    let mut engine = state.memory_engine.lock().await;
    *engine = new_engine;
    Ok(stats)
}

#[command]
async fn memory_stats(
    state: State<'_, AppState>,
) -> Result<EngineStats, String> {
    let engine = state.memory_engine.lock().await;
    Ok(engine.stats())
}

#[command]
async fn memory_get_full_graph(
    state: State<'_, AppState>,
) -> Result<FullGraphData, String> {
    let engine = state.memory_engine.lock().await;
    Ok(engine.get_full_graph())
}

#[command]
async fn memory_learn_text(
    text: String,
    session_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let mut engine = state.memory_engine.lock().await;
    let facts = engine.learn_from_text_with_session(&text, session_id.as_deref());
    Ok(facts)
}

#[command]
async fn memory_update_node_scope(
    id: u32,
    session_id: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut engine = state.memory_engine.lock().await;
    engine.update_node_scope(id, session_id)
}

#[command]
async fn memory_export_graph(
    format: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let engine = state.memory_engine.lock().await;
    match format.to_lowercase().as_str() {
        "md" | "markdown" => Ok(engine.export_graph_markdown()),
        "json" => engine.export_graph_json(),
        other => Err(format!("Formato não suportado: '{}'. Use 'json' ou 'markdown'.", other)),
    }
}

#[command]
async fn memory_delete_node(
    id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut engine = state.memory_engine.lock().await;
    engine.delete_node(id)
}

#[command]
async fn memory_clear_all(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut engine = state.memory_engine.lock().await;
    engine.clear_all();
    Ok(())
}

#[command]
async fn memory_record_vigilia_event(
    speaker: String,
    text: String,
    valence: Option<i8>,
    category: Option<String>,
) -> Result<VigiliaEvent, String> {
    MemoryGraphEngine::record_vigilia_event(&speaker, &text, valence, category.as_deref())
}

#[command]
async fn memory_get_vigilia_buffer() -> Result<Vec<VigiliaEvent>, String> {
    Ok(MemoryGraphEngine::load_vigilia_buffer())
}

/// Structural optimization, connection analysis, and memory graph pruning
#[command]
async fn memory_optimize_and_prune(
    state: State<'_, AppState>,
) -> Result<GraphOptimizationReport, String> {
    let mut engine = state.memory_engine.lock().await;
    Ok(engine.optimize_and_prune_graph())
}

#[command]
async fn memory_trigger_sleep_cycle(
    state: State<'_, AppState>,
) -> Result<SleepConsolidationReport, String> {
    let mut engine = state.memory_engine.lock().await;
    Ok(engine.run_sleep_cycle())
}

/// Consolidação profunda via IA (opcional, consome recursos do modelo ativo)
#[command]
async fn memory_trigger_deep_sleep_cycle(
    state: State<'_, AppState>,
) -> Result<SleepConsolidationReport, String> {
    let active_model = {
        let lock = state.active_model.lock().await;
        lock.clone()
    };

    let events = MemoryGraphEngine::load_vigilia_buffer();
    if events.is_empty() {
        let mut engine = state.memory_engine.lock().await;
        return Ok(engine.run_sleep_cycle());
    }

    let Some(model) = active_model else {
        // Sem modelo ativo, usar motor procedural
        let mut engine = state.memory_engine.lock().await;
        return Ok(engine.run_sleep_cycle());
    };

    let max_events = 30;
    let slice_events = if events.len() > max_events {
        &events[events.len() - max_events..]
    } else {
        &events[..]
    };

    let mut daily_log = String::new();
    for evt in slice_events {
        let clean_text = MemoryGraphEngine::sanitize_markdown_text(&evt.text);
        let truncated_text = if clean_text.len() > 250 {
            format!("{}...", &clean_text[..250])
        } else {
            clean_text
        };
        daily_log.push_str(&format!(
            "- [{}] {}: {}\n",
            evt.speaker.to_uppercase(),
            if evt.valence == 1 { "(Reward)" } else if evt.valence == -1 { "(Error)" } else { "(Dialogue)" },
            truncated_text
        ));
    }

    let system_prompt = r#"You are the Cognitive Memory Consolidation Agent for Atena.
Analyze the waking memory log and return STRICTLY a JSON object.

Rules:
1. Discard greetings and trivial filler dialogues into "noise_discarded".
2. Extract relevant facts into "facts".
3. Create protective rules / safeguards for mistakes or inhibitions into "rules_and_safeguards".

JSON format:
```json
{
  "noise_discarded": ["discarded text..."],
  "facts": [{"subject": "Entity", "property": "Fact", "valence": 0}],
  "rules_and_safeguards": [{"rule": "Avoid X", "reason": "Reason"}]
}
```"#.to_string();

    let user_prompt = format!(
        "Analyze and process the memory consolidation:\n\n{}",
        daily_log
    );

    let messages = vec![ChatMessage {
        role: "user".to_string(),
        content: user_prompt,
        images: None,
        tool_calls: None,
        tool_call_id: None,
    }];

    let params = InferenceParams {
        temperature: 0.1,
        top_p: 0.9,
        max_tokens: 2048,
        ..Default::default()
    };

    let (ai_resp, _thinking, _elapsed) = InferenceEngine::execute_chat(
        Some(model),
        system_prompt,
        messages,
        params,
        "127.0.0.1".to_string(),
        8080,
        "127.0.0.1".to_string(),
        11434,
    ).await;

    let mut engine = state.memory_engine.lock().await;
    Ok(engine.apply_ai_sleep_consolidation(&events, &ai_resp))
}

#[command]
async fn memory_clear_vigilia_buffer() -> Result<(), String> {
    MemoryGraphEngine::clear_vigilia_buffer()
}

#[command]
async fn skills_get_all() -> Result<Vec<ProceduralSkill>, String> {
    Ok(MemoryGraphEngine::load_skills())
}

#[command]
async fn skills_get_folder() -> Result<String, String> {
    let dir = MemoryGraphEngine::skills_root_dir();
    Ok(dir.to_string_lossy().to_string())
}

#[command]
async fn skills_open_folder(skill_id: Option<String>) -> Result<(), String> {
    let root = MemoryGraphEngine::skills_root_dir();
    let target_dir = if let Some(id) = skill_id {
        let skills = MemoryGraphEngine::load_skills();
        if let Some(s) = skills.iter().find(|item| item.id == id) {
            if let Some(folder) = &s.folder_path {
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
    {
        std::process::Command::new("open")
            .arg(&target_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder on macOS: {}", e))?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&target_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder on Windows: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&target_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder on Linux: {}", e))?;
    }
    Ok(())
}

#[command]
async fn skills_list_scripts(skill_id: String) -> Result<Vec<String>, String> {
    let skills = MemoryGraphEngine::load_skills();
    if let Some(s) = skills.iter().find(|item| item.id == skill_id) {
        Ok(s.scripts.clone())
    } else {
        Ok(Vec::new())
    }
}

#[command]
async fn skills_run_command(
    skill_id: Option<String>,
    command: String,
    cwd: Option<String>,
    timeout_ms: Option<u64>,
) -> Result<SkillCommandResult, String> {
    let skill_dir = if let Some(id) = &skill_id {
        let skills = MemoryGraphEngine::load_skills();
        skills
            .into_iter()
            .find(|s| s.id == *id)
            .and_then(|s| s.folder_path.map(std::path::PathBuf::from))
    } else {
        None
    };

    let working_dir = cwd.map(std::path::PathBuf::from);

    SkillScriptRunner::run_command(
        &command,
        skill_dir.as_deref(),
        working_dir.as_deref(),
        None,
        timeout_ms,
    )
    .await
}

#[command]
async fn skills_run_script(
    skill_id: Option<String>,
    slug: Option<String>,
    script_name: Option<String>,
    script_file: Option<String>,
    args: Option<Vec<String>>,
    cwd: Option<String>,
    timeout_ms: Option<u64>,
) -> Result<SkillCommandResult, String> {
    let target_id = skill_id
        .or(slug)
        .ok_or_else(|| "Missing required parameter 'skillId' or 'slug'".to_string())?;

    let target_script = script_name
        .or(script_file)
        .ok_or_else(|| "Missing required parameter 'scriptName' or 'scriptFile'".to_string())?;

    let script_args = args.unwrap_or_default();

    let skills = MemoryGraphEngine::load_skills();
    let skill = skills
        .into_iter()
        .find(|s| {
            s.id == target_id
                || s.id == format!("skill-{}", target_id)
                || target_id == format!("skill-{}", s.id)
                || s.name.eq_ignore_ascii_case(&target_id)
                || s.id.trim_start_matches("skill-") == target_id.trim_start_matches("skill-")
        })
        .ok_or_else(|| format!("Skill '{}' not found", target_id))?;

    let folder = skill
        .folder_path
        .ok_or_else(|| format!("Skill '{}' has no directory configured", target_id))?;

    let skill_dir = std::path::PathBuf::from(&folder);
    let clean_script = target_script.trim_start_matches('/');
    let script_rel_path = if clean_script.starts_with("scripts/") {
        clean_script.to_string()
    } else {
        format!("scripts/{}", clean_script)
    };
    let working_dir = cwd.map(std::path::PathBuf::from);

    SkillScriptRunner::run_script(
        &skill_dir,
        &script_rel_path,
        &script_args,
        working_dir.as_deref(),
        skill.env_vars.as_ref(),
        timeout_ms,
    )
    .await
}

#[command]
async fn skills_create_with_scripts(
    name: String,
    description: String,
    triggers: Vec<String>,
    steps: Vec<SkillStep>,
    scripts: Option<Vec<SkillScriptFilePayload>>,
    env_vars: Option<HashMap<String, String>>,
) -> Result<ProceduralSkill, String> {
    MemoryGraphEngine::create_skill_with_scripts(
        &name,
        &description,
        triggers,
        steps,
        scripts.unwrap_or_default(),
        env_vars,
    )
}

#[command]
async fn skills_update_with_scripts(
    id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    triggers: Option<Vec<String>>,
    steps: Option<Vec<SkillStep>>,
    scripts: Option<Vec<SkillScriptFilePayload>>,
    refinement_note: Option<String>,
    env_vars: Option<HashMap<String, String>>,
    permission_mode: Option<String>,
    enabled: Option<bool>,
) -> Result<ProceduralSkill, String> {
    MemoryGraphEngine::update_skill_with_scripts(
        id,
        name,
        description,
        triggers,
        steps,
        scripts,
        refinement_note,
        env_vars,
        permission_mode,
        enabled,
    )
}

#[command]
async fn skills_save_manual(
    id: Option<String>,
    name: String,
    description: String,
    triggers: Vec<String>,
    steps: Vec<String>,
    steps_detailed: Option<Vec<SkillStep>>,
    refinement_note: Option<String>,
    env_vars: Option<HashMap<String, String>>,
    permission_mode: Option<String>,
    enabled: Option<bool>,
) -> Result<ProceduralSkill, String> {
    if let Some(detailed) = steps_detailed {
        if !detailed.is_empty() {
            return Ok(MemoryGraphEngine::learn_or_refine_skill_advanced(
                id,
                &name,
                &description,
                triggers,
                detailed,
                refinement_note,
                env_vars,
                permission_mode,
                enabled,
            ));
        }
    }

    let mut step_structs = Vec::new();
    for (i, s) in steps.into_iter().enumerate() {
        let clean = s.trim().to_string();
        if !clean.is_empty() {
            step_structs.push(SkillStep {
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

    Ok(MemoryGraphEngine::learn_or_refine_skill_advanced(
        id,
        &name,
        &description,
        triggers,
        step_structs,
        refinement_note,
        env_vars,
        permission_mode,
        enabled,
    ))
}

#[command]
async fn skills_set_permission_mode(
    skill_id: String,
    permission_mode: String,
) -> Result<(), String> {
    let mut skills = MemoryGraphEngine::load_skills();
    if let Some(skill) = skills.iter_mut().find(|s| s.id == skill_id || s.id == format!("skill-{}", skill_id)) {
        skill.permission_mode = permission_mode;
        MemoryGraphEngine::save_single_skill(skill)?;
        Ok(())
    } else {
        Err(format!("Skill '{}' not found", skill_id))
    }
}

#[command]
async fn skills_set_enabled(
    skill_id: String,
    enabled: bool,
) -> Result<(), String> {
    MemoryGraphEngine::set_skill_enabled(&skill_id, enabled).map(|_| ())
}

#[command]
async fn skills_delete(id: String) -> Result<(), String> {
    MemoryGraphEngine::delete_skill(&id)
}

#[command]
async fn episodes_get_all() -> Result<Vec<EpisodeItem>, String> {
    Ok(MemoryGraphEngine::get_all_episodes())
}

#[command]
async fn episodes_open_folder() -> Result<(), String> {
    let dir = MemoryGraphEngine::episodios_dir();
    let _ = std::fs::create_dir_all(&dir);
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&dir)
            .spawn()
            .map_err(|e| format!("Falha ao abrir pasta no macOS: {}", e))?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&dir)
            .spawn()
            .map_err(|e| format!("Falha ao abrir pasta no Windows: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&dir)
            .spawn()
            .map_err(|e| format!("Falha ao abrir pasta no Linux: {}", e))?;
    }
    Ok(())
}

#[command]
async fn episodes_search(
    query: String,
    session_id: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<EpisodeItem>, String> {
    Ok(MemoryGraphEngine::search_episodes(&query, session_id.as_deref(), limit.unwrap_or(10)))
}

#[command]
async fn episodes_get_detail(identifier: String) -> Result<Option<EpisodeItem>, String> {
    Ok(MemoryGraphEngine::get_episode_detail(&identifier))
}

#[command]
async fn session_memory_stats(session_id: String) -> Result<serde_json::Value, String> {
    let (ep_count, mem_count) = MemoryGraphEngine::count_session_episodes_and_memories(&session_id);
    Ok(serde_json::json!({
        "session_id": session_id,
        "episodes_count": ep_count,
        "memories_count": mem_count,
    }))
}

#[command]
async fn session_delete_memories(
    session_id: String,
    delete_episodes: bool,
    delete_memories: bool,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let mut engine = state.memory_engine.lock().await;
    let (deleted_eps, deleted_mems) = engine.delete_session_episodes_and_memories(
        &session_id,
        delete_episodes,
        delete_memories,
    )?;
    Ok(serde_json::json!({
        "session_id": session_id,
        "deleted_episodes": deleted_eps,
        "deleted_memories": deleted_mems,
    }))
}

#[command]
async fn list_plugins() -> Result<Vec<PluginInfo>, String> {
    Ok(PluginManager::list_all_plugins())
}

#[command]
async fn toggle_plugin(id: String, enabled: bool) -> Result<bool, String> {
    PluginManager::toggle_plugin(&id, enabled)
}

#[command]
async fn get_plugins_folder() -> Result<String, String> {
    Ok(PluginManager::get_plugins_directory().to_string_lossy().to_string())
}

#[command]
async fn open_plugins_folder() -> Result<(), String> {
    let folder = PluginManager::get_plugins_directory();
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(&folder).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = silent_command("cmd").args(["/C", "start", &folder.to_string_lossy().to_string()]).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(&folder).spawn();
    }
    Ok(())
}

#[command]
async fn save_plugin_settings(
    plugin_id: String,
    settings: std::collections::HashMap<String, serde_json::Value>,
) -> Result<(), String> {
    PluginManager::save_plugin_settings(&plugin_id, settings)
}

#[command]
async fn get_plugin_settings(
    plugin_id: String,
) -> Result<std::collections::HashMap<String, serde_json::Value>, String> {
    Ok(PluginManager::get_plugin_settings(&plugin_id))
}

#[command]
async fn run_plugin_native(
    plugin_id: String,
    command: String,
    payload: serde_json::Value,
) -> Result<serde_json::Value, String> {
    PluginManager::run_plugin_native(&plugin_id, &command, payload)
}

#[command]
async fn read_plugin_script(
    plugin_id: String,
) -> Result<String, String> {
    PluginManager::read_plugin_script(&plugin_id)
}

#[command]
async fn db_get_sessions(
    state: State<'_, AppState>,
) -> Result<Vec<crate::services::db::DbChatSession>, String> {
    state.db.get_sessions()
}

#[command]
async fn db_get_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Option<crate::services::db::DbChatSession>, String> {
    state.db.get_session(&session_id)
}

#[command]
async fn db_save_session(
    state: State<'_, AppState>,
    session: crate::services::db::DbChatSession,
) -> Result<(), String> {
    state.db.save_session(&session)
}

#[command]
async fn db_save_sessions_batch(
    state: State<'_, AppState>,
    sessions: Vec<crate::services::db::DbChatSession>,
) -> Result<(), String> {
    state.db.save_sessions_batch(&sessions)
}

#[command]
async fn db_delete_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    state.db.delete_session(&session_id)
}

#[command]
async fn db_delete_sessions_batch(
    state: State<'_, AppState>,
    session_ids: Vec<String>,
) -> Result<(), String> {
    state.db.delete_sessions_batch(&session_ids)
}

#[command]
async fn db_get_personas_data(
    state: State<'_, AppState>,
) -> Result<crate::services::db::DbPersonasData, String> {
    state.db.get_personas_data()
}

#[command]
async fn db_save_persona(
    state: State<'_, AppState>,
    persona: crate::services::db::DbPersona,
) -> Result<(), String> {
    state.db.save_persona(&persona)
}

#[command]
async fn db_delete_persona(
    state: State<'_, AppState>,
    persona_id: String,
) -> Result<(), String> {
    state.db.delete_persona(&persona_id)
}

#[command]
async fn db_save_persona_override(
    state: State<'_, AppState>,
    persona_id: String,
    system_prompt: String,
) -> Result<(), String> {
    state.db.save_persona_override(&persona_id, &system_prompt)
}

#[command]
async fn db_reset_persona_override(
    state: State<'_, AppState>,
    persona_id: String,
) -> Result<(), String> {
    state.db.reset_persona_override(&persona_id)
}

#[command]
async fn db_get_setting(
    state: State<'_, AppState>,
    key: String,
) -> Result<Option<String>, String> {
    state.db.get_setting(&key)
}

#[command]
async fn db_set_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    state.db.set_setting(&key, &value)
}

#[command]
async fn db_get_all_settings(
    state: State<'_, AppState>,
) -> Result<HashMap<String, String>, String> {
    state.db.get_all_settings()
}

#[command]
async fn scheduler_get_tasks(
    state: State<'_, AppState>,
) -> Result<Vec<crate::services::db::DbScheduledTask>, String> {
    state.db.get_scheduled_tasks()
}

#[command]
async fn scheduler_save_task(
    mut task: crate::services::db::DbScheduledTask,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if task.next_run.is_none() {
        task.next_run = crate::services::scheduler::BackgroundScheduler::compute_next_run(&task.cron_expr, chrono::Utc::now()).map(|dt| dt.to_rfc3339());
    }
    state.db.save_scheduled_task(&task)
}

#[command]
async fn scheduler_delete_task(
    task_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.db.delete_scheduled_task(&task_id)
}

#[command]
async fn scheduler_toggle_task(
    task_id: String,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let tasks = state.db.get_scheduled_tasks()?;
    if let Some(mut task) = tasks.into_iter().find(|t| t.id == task_id) {
        task.enabled = enabled;
        state.db.save_scheduled_task(&task)?;
    }
    Ok(())
}

#[command]
async fn scheduler_run_now(
    task_id: String,
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let tasks = state.db.get_scheduled_tasks()?;
    if let Some(task) = tasks.into_iter().find(|t| t.id == task_id) {
        tauri::async_runtime::spawn(async move {
            crate::services::scheduler::BackgroundScheduler::execute_task(app_handle, task).await;
        });
        Ok(())
    } else {
        Err(format!("Task '{}' not found", task_id))
    }
}

#[command]
async fn start_autonomous_agent_task(
    prompt: String,
    max_steps: Option<u32>,
    session_id: Option<String>,
    on_event: Channel<crate::services::agent_loop::AgentStepEvent>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let app_cfg = crate::core::config::AppConfig::load();
    let active_model = state.active_model.lock().await.clone();
    let sys_prompt = "You are Atena, an economic and autonomous assistant. Solve the user's task using available tools step by step.".to_string();

        let mut params = crate::core::model::InferenceParams::default();
        params.mcp_tools = Some(state.mcp_manager.list_all_tools().await);

        let req = crate::StreamChatRequest {
            model: active_model,
            system_prompt: sys_prompt,
            messages: Some(vec![]),
            user_message: Some(prompt),
            session_id,
            session_title: Some("Autonomous Task".to_string()),
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

    let cancel_tok = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let cb = Arc::new(move |evt: crate::services::agent_loop::AgentStepEvent| {
        let _ = on_event.send(evt);
    });

    crate::services::agent_loop::AutonomousAgentRunner::run_loop(
        &state,
        req,
        max_steps.unwrap_or(6),
        cancel_tok,
        cb,
    ).await.map(|res| res.final_answer)
}

#[command]
async fn scheduler_get_task_runs(
    task_id: Option<String>,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<Vec<crate::services::db::DbScheduledTaskRun>, String> {
    state.db.get_task_runs(task_id.as_deref(), limit.unwrap_or(50))
}

#[command]
fn get_app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new();

    let app = tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .filter(|metadata| {
                    !metadata.target().starts_with("tao")
                        && !metadata.target().starts_with("wry")
                        && !metadata.target().starts_with("reqwest")
                        && !metadata.target().starts_with("hyper")
                })
                .build(),
        )
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            db_get_sessions,
            db_get_session,
            db_save_session,
            db_save_sessions_batch,
            db_delete_session,
            db_delete_sessions_batch,
            db_get_personas_data,
            db_save_persona,
            db_delete_persona,
            db_save_persona_override,
            db_reset_persona_override,
            db_get_setting,
            db_set_setting,
            db_get_all_settings,
            get_cached_models,
            refresh_model_info,
            scan_models,
            get_hardware_info,
            check_services,
            set_active_model,
            get_active_model,
            stream_chat,
            stop_chat_generation,
            start_mlx_server,
            start_llama_server,
            start_ollama_server,
            stop_all_servers,
            get_server_logs,
            get_developer_logs,
            clear_developer_logs,
            clear_server_logs,
            get_mcp_servers,
            save_mcp_servers,
            inspect_server_tools,
            list_all_mcp_tools,
            call_mcp_tool,
            translate_mcp_tool_labels,
            update_mcp_tool_label,
            update_mcp_tool_details,
            read_file_attachment,
            convert_audio_to_pcm,
            transcribe_audio,
            select_folder,
            save_file_content,
            open_url,
            hf_search_models,
            hf_get_model_details,
            hf_get_author_avatar,
            hf_get_author_avatars,
            start_hf_download,
            check_hf_model_downloaded,
            cancel_hf_download,
            retry_hf_download,
            dismiss_download_task,
            clear_finished_downloads,
            get_active_downloads,
            delete_model,
            check_cli_installed,
            install_cli_command,
            get_runtime_status,
            bootstrap_mlx_runtime,
            update_mlx_packages,
            bootstrap_llama_runtime,
            bootstrap_ffmpeg_runtime,
            bootstrap_all_runtimes,
            get_platform_info,
            memory_add_node,
            memory_add_edge,
            memory_query,
            memory_reinforce,
            memory_apply_decay,
            memory_save,
            memory_load,
            memory_stats,
            memory_get_full_graph,
            memory_learn_text,
            memory_update_node_scope,
            memory_export_graph,
            memory_delete_node,
            memory_clear_all,
            memory_record_vigilia_event,
            memory_get_vigilia_buffer,
            memory_trigger_sleep_cycle,
            memory_trigger_deep_sleep_cycle,
            memory_optimize_and_prune,
            memory_clear_vigilia_buffer,
            skills_get_all,
            skills_get_folder,
            skills_open_folder,
            skills_list_scripts,
            skills_run_command,
            skills_run_script,
            skills_save_manual,
            skills_create_with_scripts,
            skills_update_with_scripts,
            skills_delete,
            skills_set_permission_mode,
            skills_set_enabled,
            episodes_get_all,
            episodes_search,
            episodes_get_detail,
            session_memory_stats,
            session_delete_memories,
            episodes_open_folder,
            detect_model_directories,
            get_default_models_directory,
            check_agy_session,
            start_agy_login,
            launch_terminal_command,
            get_agy_usage,
            test_cloud_provider_connection,
            fetch_provider_models,
            get_app_config,
            save_app_config,
            get_cloud_providers_config,
            save_cloud_providers_config,
            list_plugins,
            toggle_plugin,
            get_plugins_folder,
            open_plugins_folder,
            save_plugin_settings,
            get_plugin_settings,
            run_plugin_native,
            read_plugin_script,
            get_gateways_status,
            test_telegram_connection,
            test_discord_connection,
            restart_gateways,
            scheduler_get_tasks,
            scheduler_save_task,
            scheduler_delete_task,
            scheduler_toggle_task,
            scheduler_run_now,
            scheduler_get_task_runs,
            start_autonomous_agent_task,
            update_tray_locale,
            get_app_version
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            LocalServerController::init_handle(handle.clone());
            crate::services::browser_engine::BrowserEngine::init_handle(handle.clone());
            if let Err(e) = tray::setup_tray(&handle) {
                log::warn!("Failed to initialize system tray: {}", e);
            }
            let state = app.state::<AppState>();
            let gateways = state.gateways_manager.clone();
            let cfg = crate::core::config::AppConfig::load();
            let gw_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                gateways.sync_with_config(gw_handle, &cfg).await;
            });
            let scheduler = state.scheduler.clone();
            scheduler.start(handle);
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        match event {
            tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit => {
                let _ = app_handle.save_window_state(StateFlags::all());
                let state = app_handle.state::<AppState>();
                state.scheduler.stop();
                state.gateways_manager.stop_all();
                state.backend_manager.stop_all_sync();
                if let Ok(engine) = state.memory_engine.try_lock() {
                    if engine.node_count() > 0 {
                        let _ = engine.auto_persist_default();
                    }
                };
            }
            tauri::RunEvent::WindowEvent {
                label,
                event: tauri::WindowEvent::CloseRequested { api, .. },
                ..
            } => {
                let cfg = crate::core::config::AppConfig::load();
                if cfg.close_to_tray {
                    api.prevent_close();
                    if let Some(window) = app_handle.get_webview_window(&label) {
                        let _ = window.hide();
                    }
                } else {
                    let _ = app_handle.save_window_state(StateFlags::all());
                    let state = app_handle.state::<AppState>();
                    if let Ok(engine) = state.memory_engine.try_lock() {
                        if engine.node_count() > 0 {
                            let _ = engine.auto_persist_default();
                        }
                    };
                }
            }
            tauri::RunEvent::WindowEvent {
                event: tauri::WindowEvent::Destroyed,
                ..
            } => {
                let _ = app_handle.save_window_state(StateFlags::all());
                let state = app_handle.state::<AppState>();
                state.backend_manager.stop_all_sync();
            }
            _ => {}
        }
    });
}


