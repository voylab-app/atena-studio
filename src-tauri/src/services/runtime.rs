use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;
use serde::{Deserialize, Serialize};
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStatus {
    pub llama_server_available: bool,
    pub llama_server_path: Option<String>,
    pub llama_version: Option<String>,
    pub ffmpeg_available: bool,
    pub ffmpeg_path: Option<String>,
    pub uv_available: bool,
    pub uv_path: Option<String>,
    pub mlx_ready: bool,
    pub mlx_version: Option<String>,
    pub python_path: Option<String>,
    pub is_bootstrapping: bool,
    pub runtime_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapProgress {
    pub stage: String, // "init", "downloading_python", "creating_env", "installing_packages", "done", "error"
    pub message: String,
    pub progress_percent: u8,
}

#[derive(Clone)]
pub struct RuntimeManager {
    is_bootstrapping: Arc<AtomicBool>,
}

impl RuntimeManager {
    pub fn new() -> Self {
        Self {
            is_bootstrapping: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Detects target architecture triple
    pub fn target_triple() -> &'static str {
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            "aarch64-apple-darwin"
        }
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        {
            "x86_64-apple-darwin"
        }
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            "x86_64-unknown-linux-gnu"
        }
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        {
            "x86_64-pc-windows-msvc"
        }
        #[cfg(not(any(
            all(target_os = "macos", target_arch = "aarch64"),
            all(target_os = "macos", target_arch = "x86_64"),
            all(target_os = "linux", target_arch = "x86_64"),
            all(target_os = "windows", target_arch = "x86_64")
        )))]
        {
            "unknown"
        }
    }

    /// Returns true if the current platform is macOS (where MLX is supported)
    pub fn is_macos() -> bool {
        cfg!(target_os = "macos")
    }

    /// Returns true if the current platform is Windows
    pub fn is_windows() -> bool {
        cfg!(target_os = "windows")
    }

    /// Resolves the absolute path of a bundled sidecar or system binary that is functional
    pub fn resolve_binary(name: &str) -> Option<PathBuf> {
        let triple = Self::target_triple();
        let name_with_triple = format!("{}-{}", name, triple);

        // Platform-specific binary name (add .exe on Windows)
        let bin_name = if cfg!(target_os = "windows") {
            if name.ends_with(".exe") {
                name.to_string()
            } else {
                format!("{}.exe", name)
            }
        } else {
            name.to_string()
        };

        // 1. Check dedicated Atena isolated runtime directory first
        let rdir = Self::runtime_dir();
        let mut runtime_candidates = Vec::new();
        if name.contains("llama") {
            runtime_candidates.push(rdir.join("llama").join(&bin_name));
            runtime_candidates.push(rdir.join("llama").join("llama-server"));
            runtime_candidates.push(rdir.join("llama-server").join(&bin_name));
        }
        runtime_candidates.push(rdir.join(name).join(&bin_name));
        runtime_candidates.push(rdir.join(name).join(name));
        runtime_candidates.push(rdir.join("bin").join(&bin_name));
        runtime_candidates.push(rdir.join("bin").join(name));
        runtime_candidates.push(rdir.join(&bin_name));
        runtime_candidates.push(rdir.join(name));

        for c in &runtime_candidates {
            if c.is_file() && is_binary_runnable(c) {
                return Some(c.canonicalize().unwrap_or_else(|_| c.clone()));
            }
        }

        // 2. Check standard system paths
        #[cfg(not(target_os = "windows"))]
        let system_paths = {
            let home = std::env::var("HOME").unwrap_or_default();
            vec![
                format!("/opt/homebrew/bin/{}", name),
                format!("/usr/local/bin/{}", name),
                format!("{}/.cargo/bin/{}", home, name),
                format!("{}/.local/bin/{}", home, name),
                format!("/usr/bin/{}", name),
            ]
        };

        #[cfg(target_os = "windows")]
        let system_paths = {
            let userprofile = std::env::var("USERPROFILE").unwrap_or_default();
            let localappdata = std::env::var("LOCALAPPDATA").unwrap_or_default();
            let programfiles = std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
            let programfiles_x86 = std::env::var("ProgramFiles(x86)").unwrap_or_else(|_| "C:\\Program Files (x86)".to_string());
            vec![
                format!("{}\\.cargo\\bin\\{}", userprofile, bin_name),
                format!("{}\\{}\\{}", localappdata, name, bin_name),
                format!("{}\\Programs\\{}\\{}", localappdata, name, bin_name),
                format!("{}\\Ollama\\{}", localappdata, bin_name),
                format!("{}\\{}\\{}", programfiles, name, bin_name),
                format!("{}\\{}\\{}", programfiles_x86, name, bin_name),
            ]
        };

        for sp in &system_paths {
            let p = PathBuf::from(sp);
            if p.is_file() && is_binary_runnable(&p) {
                return Some(p);
            }
        }

        // 3. Check next to running executable (App bundle MacOS folder or target/debug)
        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(exe_dir) = current_exe.parent() {
                let mut candidates = Vec::new();
                if name.contains("llama") {
                    candidates.push(exe_dir.join("llama").join(&bin_name));
                    candidates.push(exe_dir.join("llama").join("llama-server"));
                    candidates.push(exe_dir.join("../Resources/binaries/llama").join(&bin_name));
                    candidates.push(exe_dir.join("../Resources/binaries/llama").join("llama-server"));
                    candidates.push(exe_dir.join("../Resources/llama").join(&bin_name));
                    candidates.push(exe_dir.join("../Resources/llama").join("llama-server"));
                }
                candidates.push(exe_dir.join(name).join(&bin_name));
                candidates.push(exe_dir.join(name).join(name));
                candidates.push(exe_dir.join(&name_with_triple));
                candidates.push(exe_dir.join(&bin_name));
                candidates.push(exe_dir.join(name));
                candidates.push(exe_dir.join("../Resources/binaries").join(name).join(&bin_name));
                candidates.push(exe_dir.join("../Resources/binaries").join(&name_with_triple));
                candidates.push(exe_dir.join("../Resources/binaries").join(&bin_name));
                candidates.push(exe_dir.join("../Resources/binaries").join(name));
                candidates.push(exe_dir.join("../Resources").join(&name_with_triple));
                candidates.push(exe_dir.join("../Resources").join(&bin_name));
                candidates.push(exe_dir.join("../Resources").join(name));

                for c in &candidates {
                    if c.is_file() && is_binary_runnable(c) {
                        return Some(c.canonicalize().unwrap_or_else(|_| c.clone()));
                    }
                }
            }
        }

        // 4. Check workspace / project folders (during development)
        if let Ok(cwd) = std::env::current_dir() {
            let mut candidates = Vec::new();
            if name.contains("llama") {
                candidates.push(cwd.join("src-tauri/binaries/llama").join(&bin_name));
                candidates.push(cwd.join("src-tauri/binaries/llama").join("llama-server"));
                candidates.push(cwd.join("binaries/llama").join(&bin_name));
                candidates.push(cwd.join("binaries/llama").join("llama-server"));
            }
            candidates.push(cwd.join("src-tauri/binaries").join(name).join(&bin_name));
            candidates.push(cwd.join("src-tauri/binaries").join(name).join(name));
            candidates.push(cwd.join("src-tauri/binaries").join(&name_with_triple));
            candidates.push(cwd.join("src-tauri/binaries").join(&bin_name));
            candidates.push(cwd.join("src-tauri/binaries").join(name));
            candidates.push(cwd.join("binaries").join(name).join(&bin_name));
            candidates.push(cwd.join("binaries").join(&name_with_triple));
            candidates.push(cwd.join("binaries").join(&bin_name));
            candidates.push(cwd.join("binaries").join(name));
            candidates.push(cwd.join("../src-tauri/binaries").join(&name_with_triple));
            candidates.push(cwd.join("../src-tauri/binaries").join(&bin_name));
            candidates.push(cwd.join("../src-tauri/binaries").join(name));

            for c in &candidates {
                if c.is_file() && is_binary_runnable(c) {
                    return Some(c.canonicalize().unwrap_or_else(|_| c.clone()));
                }
            }
        }

        // 5. Fallback: which <name> (Unix) or where.exe <name> (Windows)
        #[cfg(not(target_os = "windows"))]
        {
            if let Ok(out) = std::process::Command::new("which").arg(name).output() {
                if out.status.success() {
                    let path_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if !path_str.is_empty() {
                        let p = PathBuf::from(path_str);
                        if p.is_file() && is_binary_runnable(&p) {
                            return Some(p);
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Ok(out) = std::process::Command::new("where.exe").arg(&bin_name).output() {
                if out.status.success() {
                    let path_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    // where.exe may return multiple lines; check each
                    for line in path_str.lines() {
                        let p = PathBuf::from(line.trim());
                        if p.is_file() && is_binary_runnable(&p) {
                            return Some(p);
                        }
                    }
                }
            }
        }

        None
    }

    /// Returns the dedicated isolated runtime directory for Atena
    pub fn runtime_dir() -> PathBuf {
        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home)
                .join("Library/Application Support/com.atena.studio/runtime")
        }

        #[cfg(target_os = "windows")]
        {
            let appdata = std::env::var("APPDATA")
                .unwrap_or_else(|_| std::env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string()));
            PathBuf::from(appdata)
                .join("com.atena.studio")
                .join("runtime")
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home)
                .join(".local/share/com.atena.studio/runtime")
        }
    }

    /// Returns path to the isolated Python executable ONLY if MLX is actually installed in it
    /// On non-macOS platforms, MLX is never available so this always returns None
    pub fn isolated_python() -> Option<PathBuf> {
        let rdir = Self::runtime_dir();

        #[cfg(target_os = "windows")]
        let py_candidates = vec![
            rdir.join("venv/Scripts/python.exe"),
            rdir.join("venv/Scripts/python3.exe"),
        ];

        #[cfg(not(target_os = "windows"))]
        let py_candidates = vec![
            rdir.join("venv/bin/python3"),
            rdir.join("venv/bin/python"),
        ];

        for py in py_candidates {
            if py.is_file() && is_executable(&py) {
                // On macOS, verify MLX is importable in this venv
                #[cfg(target_os = "macos")]
                {
                    if let Ok(out) = std::process::Command::new(&py).arg("-c").arg("import mlx_lm").output() {
                        if out.status.success() {
                            return Some(py);
                        }
                    }
                }
                // On non-macOS, return the python if it exists (no MLX check needed)
                #[cfg(not(target_os = "macos"))]
                {
                    return Some(py);
                }
            }
        }

        None
    }

    /// Returns the best available Python executable (isolated first, then system)
    pub fn get_python() -> Option<PathBuf> {
        if let Some(py) = Self::isolated_python() {
            return Some(py);
        }
        #[cfg(target_os = "windows")]
        {
            Self::resolve_binary("python")
                .or_else(|| Self::resolve_binary("python3"))
        }
        #[cfg(not(target_os = "windows"))]
        {
            Self::resolve_binary("python3").or_else(|| Self::resolve_binary("python"))
        }
    }

    /// Proactively extracts bundled llama-server and companions from application resources into runtime/llama if available
    pub fn auto_provision_bundled_llama() {
        let rdir = Self::runtime_dir();
        let llama_dir = rdir.join("llama");
        let bin_name = if cfg!(target_os = "windows") { "llama-server.exe" } else { "llama-server" };
        let target_bin = llama_dir.join(bin_name);

        if target_bin.is_file() && is_binary_runnable(&target_bin) {
            return;
        }

        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(exe_dir) = current_exe.parent() {
                let candidate_dirs = [
                    exe_dir.join("../Resources/binaries/llama"),
                    exe_dir.join("../Resources/llama"),
                    exe_dir.join("../Resources/binaries"),
                    exe_dir.join("../Resources"),
                    exe_dir.join("binaries/llama"),
                    exe_dir.join("binaries"),
                    exe_dir.to_path_buf(),
                ];

                for src_dir in &candidate_dirs {
                    let src_bin = src_dir.join(bin_name);
                    let src_bin_triple = src_dir.join(format!("{}-{}", bin_name, Self::target_triple()));
                    let active_src = if src_bin.is_file() {
                        Some(src_bin)
                    } else if src_bin_triple.is_file() {
                        Some(src_bin_triple)
                    } else {
                        None
                    };

                    if let Some(bin) = active_src {
                        let _ = std::fs::create_dir_all(&llama_dir);
                        let _ = std::fs::copy(&bin, &target_bin);
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            let _ = std::fs::set_permissions(&target_bin, std::fs::Permissions::from_mode(0o755));
                        }

                        // Copy companion dynamic libraries if present
                        if let Ok(entries) = std::fs::read_dir(src_dir) {
                            for entry in entries.flatten() {
                                let p = entry.path();
                                if let Some(ext) = p.extension() {
                                    if ext == "dylib" || ext == "so" || ext == "dll" {
                                        if let Some(fname) = p.file_name() {
                                            let dest = llama_dir.join(fname);
                                            let _ = std::fs::copy(&p, &dest);
                                        }
                                    }
                                }
                            }
                        }

                        if is_binary_runnable(&target_bin) {
                            return;
                        }
                    }
                }
            }
        }
    }

    /// Proactively extracts bundled llama-server and uv tools into runtime directory if available
    pub fn auto_provision_bundled_tools() {
        Self::auto_provision_bundled_llama();

        let rdir = Self::runtime_dir();
        let bin_dir = rdir.join("bin");
        let _ = std::fs::create_dir_all(&bin_dir);

        let uv_bin_name = if cfg!(target_os = "windows") { "uv.exe" } else { "uv" };
        let target_uv = bin_dir.join(uv_bin_name);

        if !target_uv.is_file() || !is_binary_runnable(&target_uv) {
            if let Some(src_uv) = Self::resolve_binary("uv") {
                if src_uv != target_uv {
                    let _ = std::fs::copy(&src_uv, &target_uv);
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let _ = std::fs::set_permissions(&target_uv, std::fs::Permissions::from_mode(0o755));
                    }
                }
            }
        }

        let ffmpeg_bin_name = if cfg!(target_os = "windows") { "ffmpeg.exe" } else { "ffmpeg" };
        let target_ffmpeg = bin_dir.join(ffmpeg_bin_name);

        if !target_ffmpeg.is_file() || !is_binary_runnable(&target_ffmpeg) {
            if let Some(src_ffmpeg) = Self::resolve_binary("ffmpeg") {
                if src_ffmpeg != target_ffmpeg {
                    let _ = std::fs::copy(&src_ffmpeg, &target_ffmpeg);
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let _ = std::fs::set_permissions(&target_ffmpeg, std::fs::Permissions::from_mode(0o755));
                    }
                }
            }
        }

        let uvx_bin_name = if cfg!(target_os = "windows") { "uvx.exe" } else { "uvx" };
        let target_uvx = bin_dir.join(uvx_bin_name);

        if target_uv.is_file() {
            #[cfg(unix)]
            {
                if target_uvx.symlink_metadata().is_ok() && !target_uvx.is_file() {
                    let _ = std::fs::remove_file(&target_uvx);
                }
                if target_uvx.symlink_metadata().is_err() {
                    let _ = std::os::unix::fs::symlink("uv", &target_uvx);
                }
            }
            #[cfg(windows)]
            {
                if !target_uvx.is_file() {
                    let _ = std::fs::copy(&target_uv, &target_uvx);
                }
            }
        }
    }

    /// Returns full status of all sidecars and runtime engines
    pub async fn get_status(&self) -> RuntimeStatus {
        Self::auto_provision_bundled_tools();

        let llama = Self::resolve_binary("llama-server");
        let llama_version = Self::get_llama_version().await;
        let ffmpeg = Self::resolve_binary("ffmpeg");
        let uv = Self::resolve_binary("uv");
        let python = Self::get_python();
        let rdir = Self::runtime_dir();

        let mlx_version = Self::get_mlx_version().await;
        let mlx_ready = mlx_version.is_some() || Self::check_mlx_ready().await;

        RuntimeStatus {
            llama_server_available: llama.is_some(),
            llama_server_path: llama.map(|p| p.to_string_lossy().to_string()),
            llama_version,
            ffmpeg_available: ffmpeg.is_some(),
            ffmpeg_path: ffmpeg.map(|p| p.to_string_lossy().to_string()),
            uv_available: uv.is_some(),
            uv_path: uv.map(|p| p.to_string_lossy().to_string()),
            mlx_ready,
            mlx_version,
            python_path: python.map(|p| p.to_string_lossy().to_string()),
            is_bootstrapping: self.is_bootstrapping.load(Ordering::SeqCst),
            runtime_dir: rdir.to_string_lossy().to_string(),
        }
    }

    /// Returns installed llama-server version if available and runnable
    pub async fn get_llama_version() -> Option<String> {
        if let Some(llama_bin) = Self::resolve_binary("llama-server") {
            let mut cmd = Command::new(&llama_bin);
            #[cfg(target_os = "macos")]
            {
                let home = std::env::var("HOME").unwrap_or_default();
                let rdir = Self::runtime_dir();
                let rdir_llama = rdir.join("llama");
                let parent_str = llama_bin.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
                let dyld_paths = format!("{}:{}:{}/lib:/opt/homebrew/lib:/usr/local/lib", parent_str, rdir_llama.to_string_lossy(), home);
                cmd.env("DYLD_LIBRARY_PATH", &dyld_paths);
                cmd.env("DYLD_FALLBACK_LIBRARY_PATH", &dyld_paths);
            }
            cmd.arg("--version");
            if let Ok(out) = cmd.output().await {
                if out.status.success() {
                    let s = if !out.stdout.is_empty() {
                        String::from_utf8_lossy(&out.stdout)
                    } else {
                        String::from_utf8_lossy(&out.stderr)
                    };
                    if let Some(line) = s.lines().next() {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            return Some(trimmed.to_string());
                        }
                    }
                }
            }
        }
        None
    }

    /// Returns installed mlx-lm version if available in isolated venv or system
    pub async fn get_mlx_version() -> Option<String> {
        #[cfg(not(target_os = "macos"))]
        {
            return None;
        }

        #[cfg(target_os = "macos")]
        {
            if let Some(py) = Self::isolated_python() {
                if let Ok(out) = Command::new(&py)
                    .arg("-c")
                    .arg("import mlx_lm; print(mlx_lm.__version__)")
                    .output()
                    .await
                {
                    if out.status.success() {
                        let ver = String::from_utf8_lossy(&out.stdout).trim().to_string();
                        if !ver.is_empty() {
                            return Some(ver);
                        }
                    }
                }
            }

            let aug_path = crate::services::backend::BackendManager::augmented_path();
            if let Some(uv_path) = Self::resolve_binary("uv") {
                if let Ok(out) = Command::new(&uv_path)
                    .env("PATH", &aug_path)
                    .arg("run")
                    .arg("--with")
                    .arg("mlx-lm")
                    .arg("python3")
                    .arg("-c")
                    .arg("import mlx_lm; print(mlx_lm.__version__)")
                    .output()
                    .await
                {
                    if out.status.success() {
                        let ver = String::from_utf8_lossy(&out.stdout).trim().to_string();
                        if !ver.is_empty() {
                            return Some(ver);
                        }
                    }
                }
            }

            None
        }
    }

    /// Checks if MLX is callable (via isolated venv or uv or python)
    /// On non-macOS platforms, MLX is never available
    pub async fn check_mlx_ready() -> bool {
        #[cfg(not(target_os = "macos"))]
        {
            return false;
        }

        #[cfg(target_os = "macos")]
        {
            let aug_path = crate::services::backend::BackendManager::augmented_path();

            // 1. Check isolated venv first
            if let Some(py) = Self::isolated_python() {
                if let Ok(out) = Command::new(&py)
                    .arg("-c")
                    .arg("import mlx_lm, mlx_vlm, jinja2")
                    .output()
                    .await
                {
                    if out.status.success() {
                        return true;
                    }
                }
            }

            // 2. Check if uv is available
            if let Some(uv_path) = Self::resolve_binary("uv") {
                if let Ok(out) = Command::new(&uv_path)
                    .env("PATH", &aug_path)
                    .arg("run")
                    .arg("--with")
                    .arg("mlx-lm")
                    .arg("python3")
                    .arg("-c")
                    .arg("import mlx_lm")
                    .output()
                    .await
                {
                    if out.status.success() {
                        return true;
                    }
                }
            }

            // 3. Fallback: check global python3
            if let Ok(out) = Command::new("python3")
                .env("PATH", &aug_path)
                .arg("-c")
                .arg("import mlx_lm")
                .output()
                .await
            {
                if out.status.success() {
                    return true;
                }
            }

            false
        }
    }

    /// Provisions the isolated Python runtime and installs mlx-lm, mlx-vlm, mlx-whisper
    /// Only available on macOS Apple Silicon
    pub async fn bootstrap_mlx_runtime<F>(&self, from_github: bool, on_progress: F) -> Result<(), String>
    where
        F: Fn(BootstrapProgress) + Send + Sync + 'static,
    {
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (from_github, on_progress);
            return Err("MLX requer macOS com Apple Silicon. Esta funcionalidade não está disponível nesta plataforma.".to_string());
        }

        #[cfg(target_os = "macos")]
        {
            if self.is_bootstrapping.swap(true, Ordering::SeqCst) {
                return Err("A inicialização do runtime já está em andamento.".to_string());
            }

            let is_running = self.is_bootstrapping.clone();
            let result = self.do_bootstrap(from_github, on_progress).await;
            is_running.store(false, Ordering::SeqCst);
            result
        }
    }

    /// Updates or repairs MLX packages in the isolated runtime (allows fetching latest from GitHub or PyPI)
    pub async fn update_mlx_packages<F>(&self, from_github: bool, on_progress: F) -> Result<(), String>
    where
        F: Fn(BootstrapProgress) + Send + Sync + 'static,
    {
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (from_github, on_progress);
            return Err("MLX requer macOS com Apple Silicon. Esta funcionalidade não está disponível nesta plataforma.".to_string());
        }

        #[cfg(target_os = "macos")]
        {
            if self.is_bootstrapping.swap(true, Ordering::SeqCst) {
                return Err("Uma atualização de runtime já está em andamento.".to_string());
            }

            let is_running = self.is_bootstrapping.clone();
            let result = self.do_update_packages(from_github, on_progress).await;
            is_running.store(false, Ordering::SeqCst);
            result
        }
    }

    /// Downloads and configures the self-contained official llama.cpp release with Metal acceleration into runtime/llama
    pub async fn bootstrap_llama_runtime<F>(&self, on_progress: F) -> Result<(), String>
    where
        F: Fn(BootstrapProgress) + Send + Sync + 'static,
    {
        if self.is_bootstrapping.swap(true, Ordering::SeqCst) {
            return Err("A configuração de um runtime já está em andamento.".to_string());
        }

        let is_running = self.is_bootstrapping.clone();
        let result = self.do_bootstrap_llama(on_progress).await;
        is_running.store(false, Ordering::SeqCst);
        result
    }

    /// Downloads and configures the self-contained static FFmpeg release into runtime/bin
    pub async fn bootstrap_ffmpeg_runtime<F>(&self, on_progress: F) -> Result<(), String>
    where
        F: Fn(BootstrapProgress) + Send + Sync + 'static,
    {
        if self.is_bootstrapping.swap(true, Ordering::SeqCst) {
            return Err("A configuração de um runtime já está em andamento.".to_string());
        }

        let is_running = self.is_bootstrapping.clone();
        let result = self.do_bootstrap_ffmpeg(on_progress).await;
        is_running.store(false, Ordering::SeqCst);
        result
    }

    /// Performs full automated bootstrap of all engines (GGUF, FFmpeg, and MLX) sequentially
    pub async fn bootstrap_all_runtimes<F>(&self, from_github: bool, on_progress: F) -> Result<(), String>
    where
        F: Fn(BootstrapProgress) + Send + Sync + 'static,
    {
        if self.is_bootstrapping.swap(true, Ordering::SeqCst) {
            return Err("A configuração de um runtime já está em andamento.".to_string());
        }

        let is_running = self.is_bootstrapping.clone();
        let progress_cb = Arc::new(on_progress);

        Self::auto_provision_bundled_tools();

        let llama_available = Self::resolve_binary("llama-server").is_some();
        let ffmpeg_available = Self::resolve_binary("ffmpeg").is_some();
        #[cfg(target_os = "macos")]
        let mlx_available = Self::check_mlx_ready().await;
        #[cfg(not(target_os = "macos"))]
        let mlx_available = true;

        if !ffmpeg_available {
            let cb_ffmpeg = progress_cb.clone();
            let ffmpeg_res = self.do_bootstrap_ffmpeg(move |p| {
                cb_ffmpeg(p);
            }).await;

            if let Err(e) = ffmpeg_res {
                log::warn!("Warning: Failed to auto-bootstrap FFmpeg: {}", e);
            }
        }

        if !llama_available {
            let cb_llama = progress_cb.clone();
            let need_both = !mlx_available;
            let llama_res = self.do_bootstrap_llama(move |mut p| {
                if need_both {
                    p.progress_percent = (p.progress_percent as f32 * 0.5) as u8;
                }
                cb_llama(p);
            }).await;

            if let Err(e) = llama_res {
                is_running.store(false, Ordering::SeqCst);
                return Err(format!("Erro ao configurar motor GGUF: {}", e));
            }
        }

        #[cfg(target_os = "macos")]
        {
            if !mlx_available {
                let cb_mlx = progress_cb.clone();
                let had_both = !llama_available;
                let mlx_res = self.do_bootstrap(from_github, move |mut p| {
                    if had_both {
                        p.progress_percent = 50 + ((p.progress_percent as f32 * 0.5) as u8).min(50);
                    }
                    cb_mlx(p);
                }).await;

                if let Err(e) = mlx_res {
                    is_running.store(false, Ordering::SeqCst);
                    return Err(format!("Erro ao configurar motor MLX: {}", e));
                }
            }
        }

        is_running.store(false, Ordering::SeqCst);

        progress_cb(BootstrapProgress {
            stage: "done".to_string(),
            message: "Todos os motores de IA foram validados e estão prontos para uso!".to_string(),
            progress_percent: 100,
        });

        Ok(())
    }

    #[cfg(target_os = "macos")]
    async fn do_bootstrap<F>(&self, from_github: bool, on_progress: F) -> Result<(), String>
    where
        F: Fn(BootstrapProgress) + Send + Sync + 'static,
    {
        let rdir = Self::runtime_dir();
        std::fs::create_dir_all(&rdir)
            .map_err(|e| format!("Erro ao criar diretório de runtime: {}", e))?;

        let uv_bin = Self::resolve_binary("uv").ok_or_else(|| {
            "Binário 'uv' não foi encontrado nos recursos do app nem no sistema.".to_string()
        })?;

        on_progress(BootstrapProgress {
            stage: "init".to_string(),
            message: "Iniciando configuração do ambiente de IA isolado...".to_string(),
            progress_percent: 10,
        });

        let venv_dir = rdir.join("venv");
        let aug_path = crate::services::backend::BackendManager::augmented_path();

        // Step 1: Create venv with uv (automatically downloads standalone Python 3.11 if needed)
        on_progress(BootstrapProgress {
            stage: "creating_env".to_string(),
            message: "Criando ambiente virtual Python 3.11 isolado...".to_string(),
            progress_percent: 30,
        });

        let venv_out = Command::new(&uv_bin)
            .env("PATH", &aug_path)
            .arg("venv")
            .arg(&venv_dir)
            .arg("--allow-existing")
            .arg("--python")
            .arg("3.11")
            .output()
            .await
            .map_err(|e| format!("Falha ao invocar uv venv: {}", e))?;

        if !venv_out.status.success() {
            let err_str = String::from_utf8_lossy(&venv_out.stderr);
            // Try without specifying exact python version as fallback
            let fallback_out = Command::new(&uv_bin)
                .env("PATH", &aug_path)
                .arg("venv")
                .arg(&venv_dir)
                .arg("--allow-existing")
                .output()
                .await
                .map_err(|e| format!("Falha ao criar venv de fallback: {}", e))?;

            if !fallback_out.status.success() {
                return Err(format!(
                    "Erro ao criar ambiente Python isolado: {}",
                    if !err_str.is_empty() { err_str.into_owned() } else { String::from_utf8_lossy(&fallback_out.stderr).into_owned() }
                ));
            }
        }

        let py_bin = venv_dir.join("bin/python3");
        if !py_bin.exists() {
            return Err("Executável Python não foi encontrado no venv criado.".to_string());
        }

        // Step 2: Install MLX & Whisper dependencies
        let mlx_source = if from_github {
            "git+https://github.com/ml-explore/mlx-lm.git"
        } else {
            "mlx-lm"
        };

        let msg = if from_github {
            "Instalando aceleração Apple Silicon com mlx-lm direto do repositório oficial (GitHub)...".to_string()
        } else {
            "Instalando aceleração Apple Silicon (mlx-lm, mlx-vlm, mlx-whisper)...".to_string()
        };

        on_progress(BootstrapProgress {
            stage: "installing_packages".to_string(),
            message: msg,
            progress_percent: 60,
        });

        let pip_out = Command::new(&uv_bin)
            .env("PATH", &aug_path)
            .arg("pip")
            .arg("install")
            .arg("--python")
            .arg(&py_bin)
            .arg("--upgrade")
            .arg(mlx_source)
            .arg("mlx-vlm")
            .arg("mlx-whisper")
            .arg("jinja2")
            .output()
            .await
            .map_err(|e| format!("Falha ao instalar pacotes via uv pip: {}", e))?;

        if !pip_out.status.success() {
            let stderr = String::from_utf8_lossy(&pip_out.stderr);
            return Err(format!("Falha na instalação dos pacotes MLX: {}", stderr));
        }

        on_progress(BootstrapProgress {
            stage: "done".to_string(),
            message: "Ambiente MLX configurado com sucesso e pronto para uso!".to_string(),
            progress_percent: 100,
        });

        Ok(())
    }

    #[cfg(target_os = "macos")]
    async fn do_update_packages<F>(&self, from_github: bool, on_progress: F) -> Result<(), String>
    where
        F: Fn(BootstrapProgress) + Send + Sync + 'static,
    {
        let rdir = Self::runtime_dir();
        let py_bin = rdir.join("venv/bin/python3");

        if !py_bin.exists() {
            // Se o venv não existir, faz o bootstrap completo
            return self.do_bootstrap(from_github, on_progress).await;
        }

        let uv_bin = Self::resolve_binary("uv").ok_or_else(|| {
            "Binário 'uv' não foi encontrado nos recursos do app nem no sistema.".to_string()
        })?;

        let aug_path = crate::services::backend::BackendManager::augmented_path();

        let mlx_source = if from_github {
            "git+https://github.com/ml-explore/mlx-lm.git"
        } else {
            "mlx-lm"
        };

        let msg = if from_github {
            "Atualizando mlx-lm diretamente do GitHub oficial (ml-explore/mlx-lm)...".to_string()
        } else {
            "Atualizando pacotes MLX (mlx-lm, mlx-vlm, mlx-whisper) para a versão mais recente...".to_string()
        };

        on_progress(BootstrapProgress {
            stage: "installing_packages".to_string(),
            message: msg,
            progress_percent: 40,
        });

        let pip_out = Command::new(&uv_bin)
            .env("PATH", &aug_path)
            .arg("pip")
            .arg("install")
            .arg("--python")
            .arg(&py_bin)
            .arg("--upgrade")
            .arg(mlx_source)
            .arg("mlx-vlm")
            .arg("mlx-whisper")
            .arg("jinja2")
            .output()
            .await
            .map_err(|e| format!("Falha ao atualizar pacotes via uv pip: {}", e))?;

        if !pip_out.status.success() {
            let stderr = String::from_utf8_lossy(&pip_out.stderr);
            return Err(format!("Falha na atualização dos pacotes MLX: {}", stderr));
        }

        on_progress(BootstrapProgress {
            stage: "done".to_string(),
            message: "Pacotes MLX atualizados com sucesso!".to_string(),
            progress_percent: 100,
        });

        Ok(())
    }

    async fn do_bootstrap_llama<F>(&self, on_progress: F) -> Result<(), String>
    where
        F: Fn(BootstrapProgress) + Send + Sync + 'static,
    {
        let rdir = Self::runtime_dir();
        std::fs::create_dir_all(&rdir)
            .map_err(|e| format!("Erro ao criar diretório de runtime: {}", e))?;

        let llama_dir = rdir.join("llama");
        std::fs::create_dir_all(&llama_dir)
            .map_err(|e| format!("Erro ao criar diretório do llama: {}", e))?;

        on_progress(BootstrapProgress {
            stage: "init".to_string(),
            message: "Iniciando download do motor GGUF (llama.cpp Metal)...".to_string(),
            progress_percent: 5,
        });

        let triple = Self::target_triple();
        let release_tag = "b10909";
        let is_tar = !triple.contains("windows");
        let (filename, download_url) = if triple == "aarch64-apple-darwin" {
            (
                format!("llama-{}-bin-macos-arm64.tar.gz", release_tag),
                format!("https://github.com/ggml-org/llama.cpp/releases/download/{}/llama-{}-bin-macos-arm64.tar.gz", release_tag, release_tag),
            )
        } else if triple == "x86_64-apple-darwin" {
            (
                format!("llama-{}-bin-macos-x64.tar.gz", release_tag),
                format!("https://github.com/ggml-org/llama.cpp/releases/download/{}/llama-{}-bin-macos-x64.tar.gz", release_tag, release_tag),
            )
        } else if triple == "x86_64-pc-windows-msvc" {
            (
                format!("llama-{}-bin-win-cpu-x64.zip", release_tag),
                format!("https://github.com/ggml-org/llama.cpp/releases/download/{}/llama-{}-bin-win-cpu-x64.zip", release_tag, release_tag),
            )
        } else {
            (
                format!("llama-{}-bin-ubuntu-x64.tar.gz", release_tag),
                format!("https://github.com/ggml-org/llama.cpp/releases/download/{}/llama-{}-bin-ubuntu-x64.tar.gz", release_tag, release_tag),
            )
        };

        let temp_archive = rdir.join(&filename);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .map_err(|e| format!("Falha ao inicializar cliente HTTP: {}", e))?;

        let resp = client
            .get(&download_url)
            .header("User-Agent", "Atena-Studio-App")
            .send()
            .await
            .map_err(|e| format!("Falha ao conectar com repositório de releases do llama.cpp: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Falha no download do llama.cpp (HTTP {}): {}", resp.status(), download_url));
        }

        let total_size = resp.content_length().unwrap_or(30_000_000);
        let mut downloaded: u64 = 0;
        let mut stream = resp.bytes_stream();

        let mut out_file = tokio::fs::File::create(&temp_archive)
            .await
            .map_err(|e| format!("Falha ao criar arquivo temporário de download: {}", e))?;

        let mut last_reported = 5u8;

        while let Some(chunk_res) = stream.next().await {
            let chunk = chunk_res.map_err(|e| format!("Erro na transferência do pacote: {}", e))?;
            out_file.write_all(&chunk).await.map_err(|e| format!("Erro ao gravar arquivo temporário: {}", e))?;
            downloaded += chunk.len() as u64;

            if total_size > 0 {
                let pct = (5 + (downloaded as f64 / total_size as f64 * 65.0) as u8).min(70);
                if pct > last_reported + 2 {
                    last_reported = pct;
                    on_progress(BootstrapProgress {
                        stage: "downloading_llama".to_string(),
                        message: format!("Baixando motor GGUF ({:.1} MB / {:.1} MB)...", downloaded as f64 / 1_048_576.0, total_size as f64 / 1_048_576.0),
                        progress_percent: pct,
                    });
                }
            }
        }
        out_file.flush().await.map_err(|e| format!("Erro ao finalizar arquivo temporário: {}", e))?;
        drop(out_file);

        on_progress(BootstrapProgress {
            stage: "extracting".to_string(),
            message: "Descompactando e configurando bibliotecas do motor GGUF...".to_string(),
            progress_percent: 75,
        });

        if is_tar {
            let tar_status = Command::new("tar")
                .arg("-xzf")
                .arg(&temp_archive)
                .arg("-C")
                .arg(&llama_dir)
                .arg("--strip-components=1")
                .output()
                .await;

            if let Ok(ref out) = tar_status {
                if !out.status.success() {
                    let _ = Command::new("tar")
                        .arg("-xzf")
                        .arg(&temp_archive)
                        .arg("-C")
                        .arg(&llama_dir)
                        .output()
                        .await;
                }
            }
        } else {
            let _ = Command::new("tar.exe")
                .arg("-xf")
                .arg(&temp_archive)
                .arg("-C")
                .arg(&llama_dir)
                .output()
                .await;
        }

        let _ = tokio::fs::remove_file(&temp_archive).await;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(entries) = std::fs::read_dir(&llama_dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file() {
                        let _ = std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755));
                    }
                }
            }
        }

        on_progress(BootstrapProgress {
            stage: "validating".to_string(),
            message: "Validando motor GGUF e aceleração de hardware...".to_string(),
            progress_percent: 90,
        });

        let bin_name = if cfg!(target_os = "windows") { "llama-server.exe" } else { "llama-server" };
        let mut llama_bin = llama_dir.join(bin_name);

        if !llama_bin.exists() {
            if let Ok(entries) = std::fs::read_dir(&llama_dir) {
                for entry in entries.flatten() {
                    let sub = entry.path().join(bin_name);
                    if sub.exists() {
                        llama_bin = sub;
                        break;
                    }
                }
            }
        }

        if !is_binary_runnable(&llama_bin) {
            return Err(format!("O executável llama-server foi extraído em {:?}, mas falhou no teste de execução.", llama_bin));
        }

        on_progress(BootstrapProgress {
            stage: "done".to_string(),
            message: "Motor GGUF (llama.cpp Metal) configurado com sucesso e pronto para uso!".to_string(),
            progress_percent: 100,
        });

        Ok(())
    }

    /// Downloads and installs standalone static FFmpeg binary into runtime/bin
    async fn do_bootstrap_ffmpeg<F>(&self, on_progress: F) -> Result<(), String>
    where
        F: Fn(BootstrapProgress) + Send + Sync + 'static,
    {
        let rdir = Self::runtime_dir();
        let bin_dir = rdir.join("bin");
        std::fs::create_dir_all(&bin_dir)
            .map_err(|e| format!("Falha ao criar diretório bin: {}", e))?;

        let bin_name = if cfg!(target_os = "windows") { "ffmpeg.exe" } else { "ffmpeg" };
        let target_ffmpeg = bin_dir.join(bin_name);

        if target_ffmpeg.is_file() && is_binary_runnable(&target_ffmpeg) {
            on_progress(BootstrapProgress {
                stage: "done".to_string(),
                message: "Motor FFmpeg já está instalado e funcional.".to_string(),
                progress_percent: 100,
            });
            return Ok(());
        }

        // Check if resolve_binary finds an existing functional binary to copy into runtime/bin
        if let Some(src_ffmpeg) = Self::resolve_binary("ffmpeg") {
            if src_ffmpeg != target_ffmpeg {
                if std::fs::copy(&src_ffmpeg, &target_ffmpeg).is_ok() {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let _ = std::fs::set_permissions(&target_ffmpeg, std::fs::Permissions::from_mode(0o755));
                    }
                    if is_binary_runnable(&target_ffmpeg) {
                        on_progress(BootstrapProgress {
                            stage: "done".to_string(),
                            message: "Motor FFmpeg provisionado a partir dos binários integrados.".to_string(),
                            progress_percent: 100,
                        });
                        return Ok(());
                    }
                }
            }
        }

        on_progress(BootstrapProgress {
            stage: "downloading_ffmpeg".to_string(),
            message: "Iniciando download do motor de áudio FFmpeg estático...".to_string(),
            progress_percent: 10,
        });

        let triple = Self::target_triple();
        let download_url = match triple {
            "aarch64-apple-darwin" => "https://github.com/eugeneware/ffmpeg-static/releases/download/b6.1.1/ffmpeg-darwin-arm64.gz",
            "x86_64-apple-darwin" => "https://github.com/eugeneware/ffmpeg-static/releases/download/b6.1.1/ffmpeg-darwin-x64.gz",
            "x86_64-unknown-linux-gnu" => "https://github.com/eugeneware/ffmpeg-static/releases/download/b6.1.1/ffmpeg-linux-x64.gz",
            "aarch64-unknown-linux-gnu" => "https://github.com/eugeneware/ffmpeg-static/releases/download/b6.1.1/ffmpeg-linux-arm64.gz",
            "x86_64-pc-windows-msvc" => "https://github.com/eugeneware/ffmpeg-static/releases/download/b6.1.1/ffmpeg-win32-x64.gz",
            _ => return Err(format!("Arquitetura não suportada para download automático de FFmpeg: {}", triple)),
        };

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| format!("Erro ao criar cliente HTTP: {}", e))?;

        let resp = client.get(download_url).send().await
            .map_err(|e| format!("Falha na conexão ao baixar FFmpeg: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Falha no download do FFmpeg (HTTP {}): {}", resp.status(), download_url));
        }

        let total_size = resp.content_length().unwrap_or(30_000_000);
        let mut stream = resp.bytes_stream();
        let mut downloaded: u64 = 0;
        let mut compressed_bytes = Vec::new();

        use futures_util::StreamExt;
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| format!("Erro durante download do FFmpeg: {}", e))?;
            downloaded += chunk.len() as u64;
            compressed_bytes.extend_from_slice(&chunk);

            if total_size > 0 {
                let pct = (10 + (downloaded as f64 / total_size as f64 * 65.0) as u8).min(75);
                on_progress(BootstrapProgress {
                    stage: "downloading_ffmpeg".to_string(),
                    message: format!("Baixando FFmpeg ({:.1} MB / {:.1} MB)...", downloaded as f64 / 1_048_576.0, total_size as f64 / 1_048_576.0),
                    progress_percent: pct,
                });
            }
        }

        on_progress(BootstrapProgress {
            stage: "extracting_ffmpeg".to_string(),
            message: "Descompactando binário FFmpeg estático...".to_string(),
            progress_percent: 80,
        });

        use std::io::Read;
        let mut decoder = flate2::read::GzDecoder::new(&compressed_bytes[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)
            .map_err(|e| format!("Erro ao descompactar FFmpeg gzip: {}", e))?;

        let temp_bin = bin_dir.join(format!("{}.tmp", bin_name));
        std::fs::write(&temp_bin, &decompressed)
            .map_err(|e| format!("Erro ao salvar FFmpeg: {}", e))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&temp_bin, std::fs::Permissions::from_mode(0o755));
        }

        let _ = std::fs::rename(&temp_bin, &target_ffmpeg);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&target_ffmpeg, std::fs::Permissions::from_mode(0o755));
        }

        on_progress(BootstrapProgress {
            stage: "validating_ffmpeg".to_string(),
            message: "Validando execução do FFmpeg...".to_string(),
            progress_percent: 95,
        });

        if !is_binary_runnable(&target_ffmpeg) {
            return Err("Binário FFmpeg baixado não pôde ser executado no sistema.".to_string());
        }

        on_progress(BootstrapProgress {
            stage: "done".to_string(),
            message: "Motor FFmpeg instalado com sucesso e pronto para transcrição.".to_string(),
            progress_percent: 100,
        });

        Ok(())
    }
}

fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = path.metadata() {
            return meta.permissions().mode() & 0o111 != 0;
        }
        false
    }
    #[cfg(not(unix))]
    {
        path.exists()
    }
}

fn is_binary_runnable(path: &Path) -> bool {
    if !path.is_file() || !is_executable(path) {
        return false;
    }
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_lowercase();
    let arg = if file_name.contains("ffmpeg") { "-version" } else { "--version" };

    let mut cmd = std::process::Command::new(path);
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        let rdir = crate::services::runtime::RuntimeManager::runtime_dir();
        let rdir_llama = rdir.join("llama");

        let mut paths = Vec::new();
        if let Some(parent) = path.parent() {
            paths.push(parent.to_string_lossy().to_string());
            paths.push(parent.join("../Resources/binaries/llama").to_string_lossy().to_string());
            paths.push(parent.join("../Resources/binaries").to_string_lossy().to_string());
            paths.push(parent.join("../Resources/llama").to_string_lossy().to_string());
            paths.push(parent.join("../Resources").to_string_lossy().to_string());
        }
        paths.push(rdir_llama.to_string_lossy().to_string());
        paths.push(format!("{}/lib", home));
        paths.push("/opt/homebrew/lib".to_string());
        paths.push("/opt/homebrew/opt/ggml/lib".to_string());
        paths.push("/usr/local/lib".to_string());

        let dyld_paths = paths.join(":");
        cmd.env("DYLD_LIBRARY_PATH", &dyld_paths);
        cmd.env("DYLD_FALLBACK_LIBRARY_PATH", &dyld_paths);
    }
    cmd.arg(arg);
    match cmd.output() {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_llama_runtime_resolution() {
        let mgr = RuntimeManager::new();
        let st = mgr.get_status().await;
        println!("Runtime Status: {:?}", st);
        assert!(st.llama_server_available, "llama-server must be resolved and runnable");
        assert!(st.llama_server_path.is_some(), "llama_server_path must be populated");
        assert!(st.ffmpeg_available, "ffmpeg must be resolved and runnable");
        assert!(st.ffmpeg_path.is_some(), "ffmpeg_path must be populated");
    }
}

