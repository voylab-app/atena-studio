#!/usr/bin/env bash
set -e

# Target triple detection
OS_NAME="$(uname -s)"
ARCH="$(uname -m)"

if [ "$OS_NAME" = "Darwin" ]; then
    if [ "$ARCH" = "arm64" ]; then
        TARGET_TRIPLE="aarch64-apple-darwin"
    elif [ "$ARCH" = "x86_64" ]; then
        TARGET_TRIPLE="x86_64-apple-darwin"
    else
        TARGET_TRIPLE="${ARCH}-apple-darwin"
    fi
elif [ "$OS_NAME" = "Linux" ]; then
    if [ "$ARCH" = "x86_64" ]; then
        TARGET_TRIPLE="x86_64-unknown-linux-gnu"
    elif [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
        TARGET_TRIPLE="aarch64-unknown-linux-gnu"
    else
        TARGET_TRIPLE="${ARCH}-unknown-linux-gnu"
    fi
else
    TARGET_TRIPLE="${ARCH}-unknown"
fi

BIN_DIR="$(cd "$(dirname "$0")/../src-tauri/binaries" && pwd)"
mkdir -p "$BIN_DIR"

echo "=== Atena Sidecars Setup ==="
echo "Operating System: $OS_NAME"
echo "Architecture: $ARCH"
echo "Target Triple: $TARGET_TRIPLE"
echo "Destination: $BIN_DIR"

# 1. llama-server
LLAMA_DEST="$BIN_DIR/llama-server-$TARGET_TRIPLE"
LLAMA_DIR="$BIN_DIR/llama"

# Verify if existing binary actually runs without dyld failure
LLAMA_IS_RUNNABLE=0
if [ -f "$LLAMA_DEST" ]; then
    if LD_LIBRARY_PATH="$BIN_DIR:$LLAMA_DIR" DYLD_LIBRARY_PATH="$BIN_DIR:$LLAMA_DIR" "$LLAMA_DEST" --version >/dev/null 2>&1; then
        LLAMA_IS_RUNNABLE=1
    fi
fi

if [ "$LLAMA_IS_RUNNABLE" -eq 0 ]; then
    echo "Downloading official pre-built llama.cpp bundle for $TARGET_TRIPLE..."
    TMP_LLAMA="/tmp/llama_download_$$"
    mkdir -p "$TMP_LLAMA"
    mkdir -p "$LLAMA_DIR"
    RELEASE_TAG="b10909"
    
    if [ "$TARGET_TRIPLE" = "aarch64-apple-darwin" ]; then
        curl -fsSL "https://github.com/ggml-org/llama.cpp/releases/download/${RELEASE_TAG}/llama-${RELEASE_TAG}-bin-macos-arm64.tar.gz" -o "$TMP_LLAMA/llama.tar.gz" || true
    elif [ "$TARGET_TRIPLE" = "x86_64-apple-darwin" ]; then
        curl -fsSL "https://github.com/ggml-org/llama.cpp/releases/download/${RELEASE_TAG}/llama-${RELEASE_TAG}-bin-macos-x64.tar.gz" -o "$TMP_LLAMA/llama.tar.gz" || true
    elif [ "$TARGET_TRIPLE" = "x86_64-unknown-linux-gnu" ]; then
        curl -fsSL "https://github.com/ggml-org/llama.cpp/releases/download/${RELEASE_TAG}/llama-${RELEASE_TAG}-bin-ubuntu-x64.tar.gz" -o "$TMP_LLAMA/llama.tar.gz" || true
    elif [ "$TARGET_TRIPLE" = "aarch64-unknown-linux-gnu" ]; then
        curl -fsSL "https://github.com/ggml-org/llama.cpp/releases/download/${RELEASE_TAG}/llama-${RELEASE_TAG}-bin-ubuntu-arm64.tar.gz" -o "$TMP_LLAMA/llama.tar.gz" || true
    fi

    if [ -f "$TMP_LLAMA/llama.tar.gz" ]; then
        tar -xzf "$TMP_LLAMA/llama.tar.gz" -C "$LLAMA_DIR" --strip-components=1 2>/dev/null || tar -xzf "$TMP_LLAMA/llama.tar.gz" -C "$LLAMA_DIR"
        if [ -f "$LLAMA_DIR/llama-server" ]; then
            cp "$LLAMA_DIR/llama-server" "$LLAMA_DEST"
            # Copy all companion dynamic libraries alongside the destination binary
            cp "$LLAMA_DIR"/*.dylib "$BIN_DIR/" 2>/dev/null || true
            cp "$LLAMA_DIR"/*.so* "$BIN_DIR/" 2>/dev/null || true
            chmod +x "$BIN_DIR"/*.dylib "$BIN_DIR"/*.so* "$LLAMA_DIR"/*.so* 2>/dev/null || true
            if [ "$OS_NAME" = "Linux" ] && command -v patchelf >/dev/null 2>&1; then
                patchelf --set-rpath '$ORIGIN:$ORIGIN/llama:$ORIGIN/../lib' "$LLAMA_DEST" 2>/dev/null || true
            fi
        fi
    fi
    rm -rf "$TMP_LLAMA"
fi

if [ -f "$LLAMA_DEST" ]; then
    chmod +x "$LLAMA_DEST"
    if [ "$OS_NAME" = "Linux" ] && command -v patchelf >/dev/null 2>&1; then
        patchelf --set-rpath '$ORIGIN:$ORIGIN/llama:$ORIGIN/../lib' "$LLAMA_DEST" 2>/dev/null || true
    fi
    echo "✔ llama-server ready: $LLAMA_DEST"
else
    echo "⚠ Warning: llama-server binary could not be resolved automatically. Place it at $LLAMA_DEST"
fi

# 2. ffmpeg
FFMPEG_DEST="$BIN_DIR/ffmpeg-$TARGET_TRIPLE"
FFMPEG_IS_RUNNABLE=0
if [ -f "$FFMPEG_DEST" ]; then
    if "$FFMPEG_DEST" -version >/dev/null 2>&1; then
        FFMPEG_IS_RUNNABLE=1
    fi
fi

if [ "$FFMPEG_IS_RUNNABLE" -eq 0 ]; then
    echo "Provisioning static standalone ffmpeg for $TARGET_TRIPLE..."
    TMP_FFMPEG="/tmp/ffmpeg_download_$$"
    mkdir -p "$TMP_FFMPEG"

    FFMPEG_URL=""
    if [ "$TARGET_TRIPLE" = "aarch64-apple-darwin" ]; then
        FFMPEG_URL="https://github.com/eugeneware/ffmpeg-static/releases/download/b6.1.1/ffmpeg-darwin-arm64.gz"
    elif [ "$TARGET_TRIPLE" = "x86_64-apple-darwin" ]; then
        FFMPEG_URL="https://github.com/eugeneware/ffmpeg-static/releases/download/b6.1.1/ffmpeg-darwin-x64.gz"
    elif [ "$TARGET_TRIPLE" = "x86_64-unknown-linux-gnu" ]; then
        FFMPEG_URL="https://github.com/eugeneware/ffmpeg-static/releases/download/b6.1.1/ffmpeg-linux-x64.gz"
    elif [ "$TARGET_TRIPLE" = "aarch64-unknown-linux-gnu" ]; then
        FFMPEG_URL="https://github.com/eugeneware/ffmpeg-static/releases/download/b6.1.1/ffmpeg-linux-arm64.gz"
    fi

    if [ -n "$FFMPEG_URL" ]; then
        echo "Downloading $FFMPEG_URL..."
        if curl -fsSL "$FFMPEG_URL" -o "$TMP_FFMPEG/ffmpeg.gz"; then
            gzip -d -f "$TMP_FFMPEG/ffmpeg.gz"
            chmod +w "$FFMPEG_DEST" 2>/dev/null || true
            cp -f "$TMP_FFMPEG/ffmpeg" "$FFMPEG_DEST"
            chmod +x "$FFMPEG_DEST"
        fi
    fi
    rm -rf "$TMP_FFMPEG"
fi

if [ -f "$FFMPEG_DEST" ]; then
    chmod +x "$FFMPEG_DEST"
    echo "✔ ffmpeg ready: $FFMPEG_DEST"
else
    echo "⚠ Warning: ffmpeg not found! Place it at $FFMPEG_DEST"
fi

# 3. uv (Astral Python Package & Tool Manager)
UV_DEST="$BIN_DIR/uv-$TARGET_TRIPLE"
if [ ! -f "$UV_DEST" ]; then
    if [ -x "/opt/homebrew/bin/uv" ]; then
        echo "Copying uv from /opt/homebrew/bin/uv..."
        cp "/opt/homebrew/bin/uv" "$UV_DEST"
    elif [ -x "$HOME/.cargo/bin/uv" ]; then
        echo "Copying uv from $HOME/.cargo/bin/uv..."
        cp "$HOME/.cargo/bin/uv" "$UV_DEST"
    elif command -v uv >/dev/null 2>&1; then
        echo "Copying uv from $(command -v uv)..."
        cp "$(command -v uv)" "$UV_DEST"
    else
        echo "Downloading official standalone uv binary for $TARGET_TRIPLE..."
        TMP_UV="/tmp/uv_download_$$"
        mkdir -p "$TMP_UV"
        UV_TAR_URL="https://github.com/astral-sh/uv/releases/latest/download/uv-${TARGET_TRIPLE}.tar.gz"
        if curl -fsSL "$UV_TAR_URL" -o "$TMP_UV/uv.tar.gz" 2>/dev/null; then
            tar -xzf "$TMP_UV/uv.tar.gz" -C "$TMP_UV"
            FOUND_UV="$(find "$TMP_UV" -type f -name uv | head -n 1)"
            if [ -n "$FOUND_UV" ]; then
                cp "$FOUND_UV" "$UV_DEST"
            fi
        fi
        rm -rf "$TMP_UV"
    fi
fi

if [ -f "$UV_DEST" ]; then
    chmod +x "$UV_DEST"
    echo "✔ uv ready: $UV_DEST"
else
    echo "⚠ Warning: uv not found! Place it at $UV_DEST"
fi

echo "=== Sidecars preparation complete ==="
