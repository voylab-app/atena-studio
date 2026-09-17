# Setup sidecars for Atena Studio on Windows (PowerShell)
$ErrorActionPreference = "Stop"

$Arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString().ToLower()
if ($Arch -eq "arm64") {
    $TargetTriple = "aarch64-pc-windows-msvc"
} else {
    $TargetTriple = "x86_64-pc-windows-msvc"
}

$BinDir = Join-Path $PSScriptRoot "..\src-tauri\binaries"
if (-not (Test-Path $BinDir)) {
    New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
}

Write-Host "=== Atena Windows Sidecars Setup ===" -ForegroundColor Cyan
Write-Host "Target triple: $TargetTriple"
Write-Host "Destination: $BinDir"

# 1. uv
$UvDest = Join-Path $BinDir "uv-$TargetTriple.exe"
if (-not (Test-Path $UvDest)) {
    $LocalUv = Get-Command "uv" -ErrorAction SilentlyContinue
    if ($LocalUv) {
        Write-Host "Copying uv from $($LocalUv.Source)..."
        Copy-Item -Path $LocalUv.Source -Destination $UvDest -Force
    } else {
        Write-Host "Downloading uv for Windows..."
        $UvZip = "$env:TEMP\uv-$TargetTriple.zip"
        $UvUrl = "https://github.com/astral-sh/uv/releases/latest/download/uv-$TargetTriple.zip"
        Invoke-WebRequest -Uri $UvUrl -OutFile $UvZip
        Expand-Archive -Path $UvZip -DestinationPath "$env:TEMP\uv_extracted" -Force
        Copy-Item -Path "$env:TEMP\uv_extracted\uv-$TargetTriple\uv.exe" -Destination $UvDest -Force
    }
}
if (Test-Path $UvDest) {
    Write-Host "[OK] uv ready: $UvDest" -ForegroundColor Green
} else {
    Write-Warning "uv not found!"
}

# 2. ffmpeg
$FfmpegDest = Join-Path $BinDir "ffmpeg-$TargetTriple.exe"
if (-not (Test-Path $FfmpegDest)) {
    $LocalFfmpeg = Get-Command "ffmpeg" -ErrorAction SilentlyContinue
    if ($LocalFfmpeg) {
        Write-Host "Copying ffmpeg from $($LocalFfmpeg.Source)..."
        Copy-Item -Path $LocalFfmpeg.Source -Destination $FfmpegDest -Force
    } else {
        Write-Host "Downloading ffmpeg essentials for Windows..."
        $FfmpegZip = "$env:TEMP\ffmpeg.zip"
        $FfmpegUrl = "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip"
        Invoke-WebRequest -Uri $FfmpegUrl -OutFile $FfmpegZip
        Expand-Archive -Path $FfmpegZip -DestinationPath "$env:TEMP\ffmpeg_extracted" -Force
        $ExtractedExe = Get-ChildItem -Path "$env:TEMP\ffmpeg_extracted" -Recurse -Filter "ffmpeg.exe" | Select-Object -First 1
        if ($ExtractedExe) {
            Copy-Item -Path $ExtractedExe.FullName -Destination $FfmpegDest -Force
        }
    }
}
if (Test-Path $FfmpegDest) {
    Write-Host "[OK] ffmpeg ready: $FfmpegDest" -ForegroundColor Green
} else {
    Write-Warning "ffmpeg not found!"
}

# 3. llama-server
$LlamaDest = Join-Path $BinDir "llama-server-$TargetTriple.exe"
if (-not (Test-Path $LlamaDest)) {
    $LocalLlama = Get-Command "llama-server" -ErrorAction SilentlyContinue
    if ($LocalLlama) {
        Write-Host "Copying llama-server from $($LocalLlama.Source)..."
        Copy-Item -Path $LocalLlama.Source -Destination $LlamaDest -Force
    } else {
        Write-Host "Downloading llama.cpp pre-built binary for Windows (universal CPU / all PCs)..."
        $LlamaZip = "$env:TEMP\llama.zip"
        $ReleaseTag = "b10909"
        $LlamaUrl = "https://github.com/ggml-org/llama.cpp/releases/download/$ReleaseTag/llama-$ReleaseTag-bin-win-cpu-x64.zip"
        try {
            Invoke-WebRequest -Uri $LlamaUrl -OutFile $LlamaZip
            Expand-Archive -Path $LlamaZip -DestinationPath "$env:TEMP\llama_extracted" -Force
            $ExtractedLlama = Get-ChildItem -Path "$env:TEMP\llama_extracted" -Recurse -Filter "llama-server.exe" | Select-Object -First 1
            if ($ExtractedLlama) {
                Copy-Item -Path $ExtractedLlama.FullName -Destination $LlamaDest -Force
                $LlamaDir = $ExtractedLlama.DirectoryName
                Get-ChildItem -Path $LlamaDir -Filter "*.dll" | ForEach-Object {
                    Copy-Item -Path $_.FullName -Destination $BinDir -Force
                }
                $LlamaSubDir = Join-Path $BinDir "llama"
                if (-not (Test-Path $LlamaSubDir)) {
                    New-Item -ItemType Directory -Path $LlamaSubDir -Force | Out-Null
                }
                Copy-Item -Path "$LlamaDir\*" -Destination $LlamaSubDir -Recurse -Force
            }
        } catch {
            Write-Warning "Could not auto-download llama-server.exe. Please install llama-server or place llama-server-$TargetTriple.exe in $BinDir"
        }
    }
}
if (Test-Path $LlamaDest) {
    Write-Host "[OK] llama-server ready: $LlamaDest" -ForegroundColor Green
} else {
    Write-Warning "llama-server not found!"
}

Write-Host "=== Sidecars setup complete ===" -ForegroundColor Cyan
