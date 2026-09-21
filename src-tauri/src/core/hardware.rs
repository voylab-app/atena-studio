use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use crate::core::process::silent_command;

#[cfg(target_os = "macos")]
extern "C" {
    fn proc_pid_rusage(pid: i32, flavor: i32, buffer: *mut u8) -> i32;
}

#[cfg(target_os = "windows")]
#[repr(C)]
#[allow(non_snake_case)]
struct MEMORYSTATUSEX {
    dwLength: u32,
    dwMemoryLoad: u32,
    ullTotalPhys: u64,
    ullAvailPhys: u64,
    ullTotalPageFile: u64,
    ullAvailPageFile: u64,
    ullTotalVirtual: u64,
    ullAvailVirtual: u64,
    ullAvailExtendedVirtual: u64,
}

#[cfg(target_os = "windows")]
#[repr(C)]
#[allow(non_snake_case)]
struct FILETIME {
    dwLowDateTime: u32,
    dwHighDateTime: u32,
}

#[cfg(target_os = "windows")]
extern "system" {
    fn GlobalMemoryStatusEx(lpBuffer: *mut MEMORYSTATUSEX) -> i32;
    fn GetSystemTimes(
        lpIdleTime: *mut FILETIME,
        lpKernelTime: *mut FILETIME,
        lpUserTime: *mut FILETIME,
    ) -> i32;
}




#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemHardwareInfo {
    pub device_name: String,
    pub chip_name: String,
    pub total_ram_gb: f32,
    pub used_ram_gb: f32,
    pub free_ram_gb: f32,
    pub ai_ram_gb: f32,
    pub system_other_ram_gb: f32,
    pub total_vram_gb: f32,
    pub used_vram_gb: f32,
    pub cpu_usage_percent: f32,
    pub gpu_usage_percent: f32,
    pub is_unified_memory: bool,
    pub acceleration_backend: String,
}

impl Default for SystemHardwareInfo {
    fn default() -> Self {
        Self::detect_system()
    }
}

impl SystemHardwareInfo {
    /// Detects static host hardware info once at startup.
    /// On macOS uses sysctl; on Windows uses wmic/systeminfo; on Linux uses /proc.
    pub fn detect_system() -> Self {
        #[cfg(target_os = "macos")]
        {
            Self::detect_system_macos()
        }

        #[cfg(target_os = "windows")]
        {
            Self::detect_system_windows()
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            Self::detect_system_fallback()
        }
    }

    /// Checks whether NVIDIA CUDA hardware acceleration is genuinely available on this host.
    pub fn has_cuda_acceleration() -> bool {
        #[cfg(target_os = "macos")]
        {
            false
        }
        #[cfg(not(target_os = "macos"))]
        {
            silent_command("nvidia-smi")
                .arg("-L")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
    }

    /// Checks whether an AMD GPU with acceleration capability (ROCm / HIP) is present on this host.
    pub fn has_amd_acceleration() -> bool {
        #[cfg(target_os = "macos")]
        {
            false
        }
        #[cfg(target_os = "windows")]
        {
            let out = silent_command("wmic")
                .args(["path", "win32_VideoController", "get", "Name"])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .unwrap_or_default()
                .to_lowercase();
            if out.contains("amd") || out.contains("radeon") {
                return true;
            }
            let ps_out = silent_command("powershell")
                .args(["-NoProfile", "-Command", "Get-CimInstance Win32_VideoController | Select-Object -ExpandProperty Name"])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .unwrap_or_default()
                .to_lowercase();
            ps_out.contains("amd") || ps_out.contains("radeon")
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            if std::path::Path::new("/dev/kfd").exists() {
                return true;
            }
            if std::process::Command::new("rocm-smi").output().map(|o| o.status.success()).unwrap_or(false) {
                return true;
            }
            let lspci_out = std::process::Command::new("lspci").output().ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .unwrap_or_default()
                .to_lowercase();
            lspci_out.contains("amd") || lspci_out.contains("radeon")
        }
    }

    /// Checks whether any supported GPU acceleration (Apple Metal, NVIDIA CUDA, AMD ROCm) is available.
    pub fn has_gpu_acceleration() -> bool {
        #[cfg(target_os = "macos")]
        {
            true
        }
        #[cfg(not(target_os = "macos"))]
        {
            Self::has_cuda_acceleration() || Self::has_amd_acceleration()
        }
    }

    /// Refreshes ONLY dynamic fields (used RAM, AI RAM, VRAM, CPU%, GPU%).
    /// Call this periodically (e.g. every 2.5 seconds).
    pub fn refresh(&mut self) {
        #[cfg(target_os = "macos")]
        {
            self.used_ram_gb = Self::read_used_ram_gb_macos();
            self.ai_ram_gb = Self::read_ai_process_ram_gb_macos();
            self.free_ram_gb = (self.total_ram_gb - self.used_ram_gb).max(0.0);
            self.system_other_ram_gb = (self.used_ram_gb - self.ai_ram_gb).max(0.0);
            self.used_vram_gb = self.ai_ram_gb;
            self.cpu_usage_percent = Self::read_cpu_usage_macos();
            self.gpu_usage_percent = Self::read_gpu_usage_macos();
        }

        #[cfg(target_os = "windows")]
        {
            self.used_ram_gb = Self::read_used_ram_gb_windows();
            self.ai_ram_gb = Self::read_ai_process_ram_gb_windows();
            self.free_ram_gb = (self.total_ram_gb - self.used_ram_gb).max(0.0);
            self.system_other_ram_gb = (self.used_ram_gb - self.ai_ram_gb).max(0.0);
            self.used_vram_gb = self.ai_ram_gb;
            self.cpu_usage_percent = Self::read_cpu_usage_windows();
            self.gpu_usage_percent = Self::read_gpu_usage_windows();
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
                let mut total_kb = 0u64;
                let mut avail_kb = 0u64;
                for line in content.lines() {
                    if line.starts_with("MemTotal:") {
                        total_kb = line.split_whitespace().nth(1).and_then(|v| v.parse().ok()).unwrap_or(0);
                    } else if line.starts_with("MemAvailable:") {
                        avail_kb = line.split_whitespace().nth(1).and_then(|v| v.parse().ok()).unwrap_or(0);
                    }
                }
                if total_kb > 0 {
                    self.total_ram_gb = (total_kb as f32) / (1024.0 * 1024.0);
                    let free_gb = (avail_kb as f32) / (1024.0 * 1024.0);
                    self.used_ram_gb = (self.total_ram_gb - free_gb).max(0.0);
                    self.free_ram_gb = free_gb;
                    self.ai_ram_gb = Self::read_ai_process_ram_gb_linux();
                    self.system_other_ram_gb = (self.used_ram_gb - self.ai_ram_gb).max(0.0);
                    self.used_vram_gb = self.ai_ram_gb;
                }
            }

            if let Some((vram_tot, vram_used, gpu_util)) = Self::read_nvidia_vram_linux() {
                self.total_vram_gb = vram_tot;
                self.used_vram_gb = vram_used;
                self.gpu_usage_percent = gpu_util;
                if self.ai_ram_gb == 0.0 {
                    self.ai_ram_gb = vram_used;
                }
            } else {
                self.gpu_usage_percent = Self::read_gpu_usage_linux();
            }

            self.cpu_usage_percent = Self::read_cpu_usage_linux();
        }
    }

    // =====================================================================
    // macOS Implementation
    // =====================================================================
    #[cfg(target_os = "macos")]
    fn detect_system_macos() -> Self {
        let mem_bytes = std::process::Command::new("sysctl")
            .arg("-n")
            .arg("hw.memsize")
            .output()
            .ok()
            .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<u64>().ok())
            .unwrap_or(17179869184);

        let total_ram_gb = (mem_bytes as f32) / (1024.0 * 1024.0 * 1024.0);

        let chip = std::process::Command::new("sysctl")
            .arg("-n")
            .arg("machdep.cpu.brand_string")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Apple Silicon".to_string());

        let model_id = std::process::Command::new("sysctl")
            .arg("-n")
            .arg("hw.model")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();

        let device_name = if model_id.contains("Mac16") || model_id.contains("Macmini") {
            "Mac mini (Apple Silicon)".to_string()
        } else if model_id.contains("MacBookAir") {
            "MacBook Air (Apple Silicon)".to_string()
        } else if model_id.contains("MacBook") {
            "MacBook Pro (Apple Silicon)".to_string()
        } else if model_id.contains("MacStudio") {
            "Mac Studio (Apple Silicon)".to_string()
        } else if model_id.contains("MacPro") {
            "Mac Pro (Apple Silicon)".to_string()
        } else if model_id.contains("iMac") {
            "iMac (Apple Silicon)".to_string()
        } else if !model_id.is_empty() {
            format!("Mac ({})", model_id)
        } else {
            "Mac (Apple Silicon)".to_string()
        };

        // Read initial dynamic values from real system
        let used_ram_gb = Self::read_used_ram_gb_macos();
        let ai_ram_gb = Self::read_ai_process_ram_gb_macos();
        let free_ram_gb = (total_ram_gb - used_ram_gb).max(0.0);
        let system_other_ram_gb = (used_ram_gb - ai_ram_gb).max(0.0);
        let cpu_usage_percent = Self::read_cpu_usage_macos();
        let gpu_usage_percent = Self::read_gpu_usage_macos();

        Self {
            device_name,
            chip_name: format!("{} (Metal 3 / Neural Engine)", chip),
            total_ram_gb,
            used_ram_gb,
            free_ram_gb,
            ai_ram_gb,
            system_other_ram_gb,
            total_vram_gb: total_ram_gb, // Unified memory
            used_vram_gb: ai_ram_gb,     // Dedicated AI Model + KV Cache VRAM
            cpu_usage_percent,
            gpu_usage_percent,
            is_unified_memory: true,
            acceleration_backend: "Metal 3".into(),
        }
    }

    /// Reads real memory consumption strictly used by local AI processes (MLX, llama-server, Ollama).
    /// On macOS Apple Silicon, MLX allocates model weights and KV cache directly into Metal GPU buffers,
    /// which standard Unix `ps rss` omits. Using `proc_pid_rusage` retrieves the true `phys_footprint`.
    #[cfg(target_os = "macos")]
    fn read_ai_process_ram_gb_macos() -> f32 {
        let output = match std::process::Command::new("ps")
            .args(["-A", "-ww", "-o", "pid,rss,command"])
            .output()
        {
            Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
            Err(_) => return 0.0,
        };

        let mut total_bytes: u64 = 0;
        for line in output.lines() {
            let line_lower = line.to_lowercase();
            if line_lower.contains("mlx_lm.server")
                || line_lower.contains("mlx_vlm.server")
                || line_lower.contains("mlx_whisper")
                || line_lower.contains("mlx_audio")
                || line_lower.contains("llama-server")
                || (line_lower.contains("ollama") && (line_lower.contains("serve") || line_lower.contains("runner") || line_lower.contains("app")))
            {
                let mut parts = line.split_whitespace();
                if let (Some(pid_str), Some(rss_str)) = (parts.next(), parts.next()) {
                    let mut got_footprint = false;
                    if let Ok(pid) = pid_str.parse::<i32>() {
                        // In macOS sys/resource.h, RUSAGE_INFO_V4 (flavor 4) is 352 bytes.
                        // In all Darwin rusage_info versions (v0..v6), ri_phys_footprint is uint64_t at offset 72..80.
                        // Using a 1024-byte zeroed buffer guarantees zero possibility of stack buffer overflow across all macOS versions.
                        let mut buf = [0u8; 1024];
                        let res = unsafe {
                            proc_pid_rusage(pid, 4, buf.as_mut_ptr())
                        };
                        if res == 0 {
                            let footprint = u64::from_ne_bytes(buf[72..80].try_into().unwrap_or([0; 8]));
                            if footprint > 0 {
                                total_bytes += footprint;
                                got_footprint = true;
                            }
                        }
                    }

                    if !got_footprint {
                        if let Ok(kb) = rss_str.parse::<u64>() {
                            total_bytes += kb * 1024;
                        }
                    }
                }
            }
        }

        let total_gb = (total_bytes as f32) / (1024.0 * 1024.0 * 1024.0);
        (total_gb * 10.0).round() / 10.0
    }

    /// Reads real used RAM from macOS `vm_stat`.
    #[cfg(target_os = "macos")]
    fn read_used_ram_gb_macos() -> f32 {
        let output = match std::process::Command::new("vm_stat").output() {
            Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
            Err(_) => return 0.0,
        };

        // Detect page size from the first line: "Mach Virtual Memory Statistics: (page size of XXXXX bytes)"
        let page_size: u64 = output
            .lines()
            .next()
            .and_then(|line| {
                // Extract number before "bytes)"
                line.split("page size of ")
                    .nth(1)
                    .and_then(|s| s.split(' ').next())
                    .and_then(|n| n.parse::<u64>().ok())
            })
            .unwrap_or(16384); // Apple Silicon default

        let parse_pages = |key: &str| -> u64 {
            output
                .lines()
                .find(|line| line.starts_with(key))
                .and_then(|line| {
                    line.split(':')
                        .nth(1)
                        .map(|v| v.trim().trim_end_matches('.').replace(',', ""))
                        .and_then(|v| v.parse::<u64>().ok())
                })
                .unwrap_or(0)
        };

        let active = parse_pages("Pages active");
        let wired = parse_pages("Pages wired down");
        let speculative = parse_pages("Pages speculative");
        let compressed = parse_pages("Pages occupied by compressor");

        let used_bytes = (active + wired + speculative + compressed) * page_size;
        (used_bytes as f32) / (1024.0 * 1024.0 * 1024.0)
    }

    /// Reads real CPU usage percentage from macOS `top`.
    #[cfg(target_os = "macos")]
    fn read_cpu_usage_macos() -> f32 {
        let output = match std::process::Command::new("top")
            .args(["-l", "1", "-n", "0"])
            .output()
        {
            Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
            Err(_) => return 0.0,
        };

        // Find all "CPU usage:" lines, take the last one (second sample)
        let cpu_lines: Vec<&str> = output
            .lines()
            .filter(|line| line.starts_with("CPU usage:"))
            .collect();

        let line = match cpu_lines.last() {
            Some(l) => *l,
            None => return 0.0,
        };

        // Format: "CPU usage: 4.76% user, 5.95% sys, 89.28% idle"
        let mut user = 0.0f32;
        let mut sys = 0.0f32;

        for part in line.split(',') {
            let part = part.trim();
            if part.contains("user") {
                user = part
                    .split('%')
                    .next()
                    .and_then(|s| s.split_whitespace().last())
                    .and_then(|n| n.parse::<f32>().ok())
                    .unwrap_or(0.0);
            } else if part.contains("sys") {
                sys = part
                    .split('%')
                    .next()
                    .and_then(|s| s.split_whitespace().last())
                    .and_then(|n| n.parse::<f32>().ok())
                    .unwrap_or(0.0);
            }
        }

        user + sys
    }

    /// Reads GPU utilization on Apple Silicon via `ioreg`.
    #[cfg(target_os = "macos")]
    fn read_gpu_usage_macos() -> f32 {
        let output = match std::process::Command::new("ioreg")
            .args(["-r", "-d", "1", "-w", "0", "-c", "IOAccelerator"])
            .output()
        {
            Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
            Err(_) => return 0.0,
        };

        // Look for "Device Utilization %" = XX or "GPU Activity(%)" = XX
        for line in output.lines() {
            let trimmed = line.trim();

            // Pattern: "Device Utilization %" = 12
            if trimmed.contains("Device Utilization %") || trimmed.contains("GPU Activity(%)") {
                if let Some(val_str) = trimmed.split('=').nth(1) {
                    let cleaned = val_str.trim().trim_end_matches('}').trim();
                    if let Ok(val) = cleaned.parse::<f32>() {
                        return val;
                    }
                    // Try parsing as integer
                    if let Ok(val) = cleaned.parse::<u32>() {
                        return val as f32;
                    }
                }
            }

            // Pattern: "gpu-utilization" = XX (some driver versions)
            if trimmed.contains("gpu-utilization") {
                if let Some(val_str) = trimmed.split('=').nth(1) {
                    let cleaned = val_str.trim().trim_end_matches('}').trim();
                    if let Ok(val) = cleaned.parse::<f32>() {
                        return val;
                    }
                }
            }
        }

        0.0
    }

    // =====================================================================
    // Windows Implementation
    // =====================================================================
    #[cfg(target_os = "windows")]
    fn get_windows_memory_status() -> (u64, u64) {
        let mut mem = MEMORYSTATUSEX {
            dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
            dwMemoryLoad: 0,
            ullTotalPhys: 0,
            ullAvailPhys: 0,
            ullTotalPageFile: 0,
            ullAvailPageFile: 0,
            ullTotalVirtual: 0,
            ullAvailVirtual: 0,
            ullAvailExtendedVirtual: 0,
        };
        let ok = unsafe { GlobalMemoryStatusEx(&mut mem) };
        if ok != 0 && mem.ullTotalPhys > 0 {
            (mem.ullTotalPhys, mem.ullAvailPhys)
        } else {
            (0, 0)
        }
    }

    #[cfg(target_os = "windows")]
    fn detect_system_windows() -> Self {
        // Total physical memory via native Win32 API
        let (total_phys, avail_phys) = Self::get_windows_memory_status();
        let total_ram_gb = if total_phys > 0 {
            let gb = (total_phys as f32) / (1024.0 * 1024.0 * 1024.0);
            (gb * 10.0).round() / 10.0
        } else {
            // Fallback via wmic or 16.0 GB
            let mem_bytes = silent_command("wmic")
                .args(["ComputerSystem", "get", "TotalPhysicalMemory", "/value"])
                .output()
                .ok()
                .and_then(|o| {
                    let out = String::from_utf8_lossy(&o.stdout).to_string();
                    out.lines()
                        .find(|l| l.starts_with("TotalPhysicalMemory="))
                        .and_then(|l| l.split('=').nth(1))
                        .and_then(|v| v.trim().parse::<u64>().ok())
                })
                .unwrap_or(17179869184);
            let gb = (mem_bytes as f32) / (1024.0 * 1024.0 * 1024.0);
            (gb * 10.0).round() / 10.0
        };

        // CPU name (wmic -> registry -> environment variable)
        let chip = silent_command("wmic")
            .args(["cpu", "get", "Name", "/value"])
            .output()
            .ok()
            .and_then(|o| {
                let out = String::from_utf8_lossy(&o.stdout).to_string();
                out.lines()
                    .find(|l| l.starts_with("Name="))
                    .map(|l| l.split('=').nth(1).unwrap_or("").trim().to_string())
            })
            .filter(|s| !s.is_empty())
            .or_else(|| {
                silent_command("reg")
                    .args(["query", "HKLM\\HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0", "/v", "ProcessorNameString"])
                    .output()
                    .ok()
                    .and_then(|o| {
                        let out = String::from_utf8_lossy(&o.stdout).to_string();
                        out.lines()
                            .find(|l| l.contains("ProcessorNameString"))
                            .and_then(|l| l.split("REG_SZ").nth(1))
                            .map(|s| s.trim().to_string())
                    })
                    .filter(|s| !s.is_empty())
            })
            .or_else(|| std::env::var("PROCESSOR_IDENTIFIER").ok().filter(|s| !s.is_empty()))
            .unwrap_or_else(|| "CPU".to_string());

        // GPU name for display (wmic -> powershell CIM)
        let gpu_name = silent_command("wmic")
            .args(["path", "win32_VideoController", "get", "Name", "/value"])
            .output()
            .ok()
            .and_then(|o| {
                let out = String::from_utf8_lossy(&o.stdout).to_string();
                out.lines()
                    .find(|l| l.starts_with("Name="))
                    .map(|l| l.split('=').nth(1).unwrap_or("").trim().to_string())
            })
            .filter(|s| !s.is_empty())
            .or_else(|| {
                silent_command("powershell")
                    .args(["-NoProfile", "-Command", "Get-CimInstance Win32_VideoController | Select-Object -ExpandProperty Name | Select-Object -First 1"])
                    .output()
                    .ok()
                    .and_then(|o| {
                        let out = String::from_utf8_lossy(&o.stdout).trim().to_string();
                        if out.is_empty() { None } else { Some(out) }
                    })
            })
            .unwrap_or_else(|| "GPU".to_string());

        // Computer model (wmic -> registry -> default)
        let device_name = silent_command("wmic")
            .args(["ComputerSystem", "get", "Model", "/value"])
            .output()
            .ok()
            .and_then(|o| {
                let out = String::from_utf8_lossy(&o.stdout).to_string();
                out.lines()
                    .find(|l| l.starts_with("Model="))
                    .map(|l| l.split('=').nth(1).unwrap_or("").trim().to_string())
            })
            .filter(|s| !s.is_empty())
            .or_else(|| {
                silent_command("reg")
                    .args(["query", "HKLM\\HARDWARE\\DESCRIPTION\\System\\BIOS", "/v", "SystemProductName"])
                    .output()
                    .ok()
                    .and_then(|o| {
                        let out = String::from_utf8_lossy(&o.stdout).to_string();
                        out.lines()
                            .find(|l| l.contains("SystemProductName"))
                            .and_then(|l| l.split("REG_SZ").nth(1))
                            .map(|s| s.trim().to_string())
                    })
                    .filter(|s| !s.is_empty())
            })
            .unwrap_or_else(|| "PC Windows".to_string());

        let used_ram_gb = if total_phys > 0 {
            let used_bytes = total_phys.saturating_sub(avail_phys);
            let gb = (used_bytes as f32) / (1024.0 * 1024.0 * 1024.0);
            (gb * 10.0).round() / 10.0
        } else {
            Self::read_used_ram_gb_windows()
        };

        let ai_ram_gb = Self::read_ai_process_ram_gb_windows();
        let free_ram_gb = (total_ram_gb - used_ram_gb).max(0.0);
        let system_other_ram_gb = (used_ram_gb - ai_ram_gb).max(0.0);
        let cpu_usage_percent = Self::read_cpu_usage_windows();
        let gpu_usage_percent = Self::read_gpu_usage_windows();

        // Determine acceleration backend
        let gpu_lower = gpu_name.to_lowercase();
        let has_cuda = Self::has_cuda_acceleration();
        let accel = if has_cuda && (gpu_lower.contains("nvidia") || gpu_lower.contains("geforce") || gpu_lower.contains("rtx") || gpu_lower.contains("gtx")) {
            "NVIDIA CUDA".to_string()
        } else if gpu_lower.contains("amd") || gpu_lower.contains("radeon") {
            "AMD ROCm / DirectML".to_string()
        } else if gpu_lower.contains("intel") {
            "Intel oneAPI / DirectML".to_string()
        } else {
            "CPU".to_string()
        };

        // Try to detect dedicated VRAM (wmic -> powershell)
        let vram_bytes = silent_command("wmic")
            .args(["path", "win32_VideoController", "get", "AdapterRAM", "/value"])
            .output()
            .ok()
            .and_then(|o| {
                let out = String::from_utf8_lossy(&o.stdout).to_string();
                out.lines()
                    .find(|l| l.starts_with("AdapterRAM="))
                    .and_then(|l| l.split('=').nth(1))
                    .and_then(|v| v.trim().parse::<u64>().ok())
            })
            .or_else(|| {
                silent_command("powershell")
                    .args(["-NoProfile", "-Command", "Get-CimInstance Win32_VideoController | Select-Object -ExpandProperty AdapterRAM | Select-Object -First 1"])
                    .output()
                    .ok()
                    .and_then(|o| {
                        let out = String::from_utf8_lossy(&o.stdout).trim().to_string();
                        out.parse::<u64>().ok()
                    })
            })
            .unwrap_or(0);

        let total_vram_gb = if vram_bytes > 0 {
            (vram_bytes as f32) / (1024.0 * 1024.0 * 1024.0)
        } else {
            0.0
        };

        Self {
            device_name,
            chip_name: format!("{} + {}", chip, gpu_name),
            total_ram_gb,
            used_ram_gb,
            free_ram_gb,
            ai_ram_gb,
            system_other_ram_gb,
            total_vram_gb,
            used_vram_gb: ai_ram_gb,
            cpu_usage_percent,
            gpu_usage_percent,
            is_unified_memory: false,
            acceleration_backend: accel,
        }
    }

    /// Reads used RAM on Windows via native Win32 GlobalMemoryStatusEx with wmic fallback
    #[cfg(target_os = "windows")]
    fn read_used_ram_gb_windows() -> f32 {
        let (total_phys, avail_phys) = Self::get_windows_memory_status();
        if total_phys > 0 {
            let used_bytes = total_phys.saturating_sub(avail_phys);
            let gb = (used_bytes as f32) / (1024.0 * 1024.0 * 1024.0);
            return (gb * 10.0).round() / 10.0;
        }

        // Fallback: Get free physical memory (in KB) then subtract from total via wmic
        let free_kb = silent_command("wmic")
            .args(["OS", "get", "FreePhysicalMemory", "/value"])
            .output()
            .ok()
            .and_then(|o| {
                let out = String::from_utf8_lossy(&o.stdout).to_string();
                out.lines()
                    .find(|l| l.starts_with("FreePhysicalMemory="))
                    .and_then(|l| l.split('=').nth(1))
                    .and_then(|v| v.trim().parse::<u64>().ok())
            })
            .unwrap_or(0);

        let total_kb = silent_command("wmic")
            .args(["ComputerSystem", "get", "TotalPhysicalMemory", "/value"])
            .output()
            .ok()
            .and_then(|o| {
                let out = String::from_utf8_lossy(&o.stdout).to_string();
                out.lines()
                    .find(|l| l.starts_with("TotalPhysicalMemory="))
                    .and_then(|l| l.split('=').nth(1))
                    .and_then(|v| v.trim().parse::<u64>().ok())
            })
            .unwrap_or(0) / 1024; // bytes -> KB

        let used_kb = total_kb.saturating_sub(free_kb);
        (used_kb as f32) / (1024.0 * 1024.0)
    }

    /// Reads AI process memory on Windows via tasklist
    #[cfg(target_os = "windows")]
    fn read_ai_process_ram_gb_windows() -> f32 {
        let output = match silent_command("tasklist")
            .args(["/FO", "CSV", "/NH"])
            .output()
        {
            Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
            Err(_) => return 0.0,
        };

        let mut total_kb: u64 = 0;
        for line in output.lines() {
            let line_lower = line.to_lowercase();
            if line_lower.contains("llama-server")
                || line_lower.contains("ollama")
                || line_lower.contains("python")
            {
                // CSV format: "name","PID","session","#","mem_usage"
                // mem_usage is like "123,456 K" or "123.456 K"
                let parts: Vec<&str> = line.split('"').collect();
                if parts.len() >= 10 {
                    let mem_str = parts[9].replace(",", "").replace(".", "").replace(" K", "");
                    if let Ok(kb) = mem_str.trim().parse::<u64>() {
                        total_kb += kb;
                    }
                }
            }
        }

        let total_gb = (total_kb as f32) / (1024.0 * 1024.0);
        (total_gb * 10.0).round() / 10.0
    }

    /// Reads CPU usage on Windows via native Win32 GetSystemTimes with wmic fallback
    #[cfg(target_os = "windows")]
    fn read_cpu_usage_windows() -> f32 {
        static PREV_CPU_TIMES: std::sync::Mutex<Option<(u64, u64)>> = std::sync::Mutex::new(None);

        let mut idle_ft = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
        let mut kernel_ft = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
        let mut user_ft = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };

        let success = unsafe {
            GetSystemTimes(&mut idle_ft, &mut kernel_ft, &mut user_ft)
        };

        if success != 0 {
            let idle = ((idle_ft.dwHighDateTime as u64) << 32) | (idle_ft.dwLowDateTime as u64);
            let kernel = ((kernel_ft.dwHighDateTime as u64) << 32) | (kernel_ft.dwLowDateTime as u64);
            let user = ((user_ft.dwHighDateTime as u64) << 32) | (user_ft.dwLowDateTime as u64);
            let total = kernel + user;

            let mut guard = PREV_CPU_TIMES.lock().unwrap_or_else(|e| e.into_inner());
            match *guard {
                Some((prev_idle, prev_total)) => {
                    let idle_diff = idle.saturating_sub(prev_idle);
                    let total_diff = total.saturating_sub(prev_total);
                    *guard = Some((idle, total));

                    if total_diff > 0 {
                        let usage = (1.0 - (idle_diff as f64 / total_diff as f64)) * 100.0;
                        return (usage.clamp(0.0, 100.0) as f32 * 10.0).round() / 10.0;
                    }
                    return 0.0;
                }
                None => {
                    *guard = Some((idle, total));
                    return 0.0;
                }
            }
        }

        // Fallback to wmic
        silent_command("wmic")
            .args(["cpu", "get", "LoadPercentage", "/value"])
            .output()
            .ok()
            .and_then(|o| {
                let out = String::from_utf8_lossy(&o.stdout).to_string();
                out.lines()
                    .find(|l| l.starts_with("LoadPercentage="))
                    .and_then(|l| l.split('=').nth(1))
                    .and_then(|v| v.trim().parse::<f32>().ok())
            })
            .unwrap_or(0.0)
    }

    /// Reads GPU usage on Windows (nvidia-smi if NVIDIA, otherwise 0)
    #[cfg(target_os = "windows")]
    fn read_gpu_usage_windows() -> f32 {
        // Try nvidia-smi for NVIDIA GPUs
        if let Ok(out) = silent_command("nvidia-smi")
            .args(["--query-gpu=utilization.gpu", "--format=csv,noheader,nounits"])
            .output()
        {
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if let Ok(val) = stdout.parse::<f32>() {
                    return val;
                }
            }
        }
        0.0
    }

    // =====================================================================
    // Fallback (Linux / Unknown)
    // =====================================================================
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn detect_system_fallback() -> Self {
        // Read /proc/meminfo for total RAM
        let total_ram_gb = std::fs::read_to_string("/proc/meminfo")
            .ok()
            .and_then(|content| {
                content.lines()
                    .find(|l| l.starts_with("MemTotal:"))
                    .and_then(|l| {
                        l.split_whitespace().nth(1).and_then(|v| v.parse::<u64>().ok())
                    })
            })
            .map(|kb| (kb as f32) / (1024.0 * 1024.0))
            .unwrap_or(16.0);

        let has_cuda = Self::has_cuda_acceleration();
        let (chip_name, accel) = if has_cuda {
            let name = std::process::Command::new("nvidia-smi")
                .args(["--query-gpu=name", "--format=csv,noheader"])
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.lines().next().unwrap_or("").trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "NVIDIA GPU".to_string());
            (name, "NVIDIA CUDA".to_string())
        } else {
            let lspci_out = std::process::Command::new("lspci")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .unwrap_or_default()
                .to_lowercase();
            if lspci_out.contains("amd") || lspci_out.contains("radeon") {
                ("AMD GPU".to_string(), "AMD ROCm / CPU".to_string())
            } else if lspci_out.contains("intel") {
                ("Intel Graphics".to_string(), "Intel oneAPI / CPU".to_string())
            } else {
                ("CPU".to_string(), "CPU".to_string())
            }
        };

        let mut hw = Self {
            device_name: "Linux PC".to_string(),
            chip_name,
            total_ram_gb,
            used_ram_gb: 0.0,
            free_ram_gb: total_ram_gb,
            ai_ram_gb: 0.0,
            system_other_ram_gb: 0.0,
            total_vram_gb: 0.0,
            used_vram_gb: 0.0,
            cpu_usage_percent: 0.0,
            gpu_usage_percent: 0.0,
            is_unified_memory: false,
            acceleration_backend: accel,
        };
        hw.refresh();
        hw
    }

    /// Reads memory consumption strictly used by local AI processes (llama-server, ollama, mlx) on Linux.
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn read_ai_process_ram_gb_linux() -> f32 {
        let mut total_kb: u64 = 0;

        // Inspect /proc/<pid>/comm, /proc/<pid>/cmdline, and /proc/<pid>/status directly
        if let Ok(entries) = std::fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                let name_str = file_name.to_string_lossy();
                if name_str.chars().all(|c| c.is_ascii_digit()) {
                    let pid_path = entry.path();
                    let mut matched = false;

                    if let Ok(comm) = std::fs::read_to_string(pid_path.join("comm")) {
                        let comm_lower = comm.trim().to_lowercase();
                        if comm_lower.contains("llama-server")
                            || comm_lower.contains("ollama")
                            || comm_lower.contains("mlx")
                        {
                            matched = true;
                        }
                    }

                    if !matched {
                        if let Ok(cmdline) = std::fs::read_to_string(pid_path.join("cmdline")) {
                            let cmd_lower = cmdline.to_lowercase();
                            if cmd_lower.contains("llama-server")
                                || cmd_lower.contains("ollama")
                                || cmd_lower.contains("mlx")
                                || (cmd_lower.contains("python") && (cmd_lower.contains("serve") || cmd_lower.contains("model")))
                            {
                                matched = true;
                            }
                        }
                    }

                    if matched {
                        if let Ok(status) = std::fs::read_to_string(pid_path.join("status")) {
                            for line in status.lines() {
                                if line.starts_with("VmRSS:") {
                                    if let Some(val_str) = line.split_whitespace().nth(1) {
                                        if let Ok(kb) = val_str.parse::<u64>() {
                                            total_kb += kb;
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fallback to ps command if /proc scanning yielded 0
        if total_kb == 0 {
            if let Ok(output) = std::process::Command::new("ps")
                .args(["-eo", "pid,rss,comm,args"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let line_lower = line.to_lowercase();
                    if line_lower.contains("llama-server")
                        || line_lower.contains("ollama")
                        || line_lower.contains("mlx")
                    {
                        let mut parts = line.split_whitespace();
                        let _pid = parts.next();
                        if let Some(rss_str) = parts.next() {
                            if let Ok(kb) = rss_str.parse::<u64>() {
                                total_kb += kb;
                            }
                        }
                    }
                }
            }
        }

        let total_gb = (total_kb as f32) / (1024.0 * 1024.0);
        (total_gb * 10.0).round() / 10.0
    }

    /// Reads dedicated NVIDIA VRAM metrics if available on Linux.
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn read_nvidia_vram_linux() -> Option<(f32, f32, f32)> {
        let output = std::process::Command::new("nvidia-smi")
            .args(["--query-gpu=memory.total,memory.used,utilization.gpu", "--format=csv,noheader,nounits"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let out = String::from_utf8_lossy(&output.stdout);
        let first_line = out.lines().next()?;
        let parts: Vec<&str> = first_line.split(',').map(|s| s.trim()).collect();
        if parts.len() >= 3 {
            let total_mb: f32 = parts[0].parse().ok()?;
            let used_mb: f32 = parts[1].parse().ok()?;
            let gpu_util: f32 = parts[2].parse().ok().unwrap_or(0.0);
            Some((total_mb / 1024.0, used_mb / 1024.0, gpu_util))
        } else {
            None
        }
    }

    /// Reads CPU usage percentage from `top` on Linux.
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn read_cpu_usage_linux() -> f32 {
        if let Ok(output) = std::process::Command::new("top")
            .args(["-bn1"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.contains("%Cpu(s):") || line.contains("Cpu(s):") {
                    for part in line.split(',') {
                        let trimmed = part.trim();
                        if trimmed.ends_with("id") || trimmed.ends_with("id,") {
                            if let Some(val_str) = trimmed.split_whitespace().next() {
                                if let Ok(idle) = val_str.parse::<f32>() {
                                    return (100.0 - idle).clamp(0.0, 100.0);
                                }
                            }
                        }
                    }
                }
            }
        }
        0.0
    }

    /// Reads GPU usage percentage from `nvidia-smi` on Linux.
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn read_gpu_usage_linux() -> f32 {
        if let Ok(output) = std::process::Command::new("nvidia-smi")
            .args(["--query-gpu=utilization.gpu", "--format=csv,noheader,nounits"])
            .output()
        {
            if output.status.success() {
                let out = String::from_utf8_lossy(&output.stdout);
                if let Some(first_line) = out.lines().next() {
                    if let Ok(util) = first_line.trim().parse::<f32>() {
                        return util.clamp(0.0, 100.0);
                    }
                }
            }
        }
        0.0
    }
}
