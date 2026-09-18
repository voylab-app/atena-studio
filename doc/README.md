# 🏛️ Atena Studio — Architecture and Features Guide

Welcome to the official documentation for **Atena Studio**, a high-performance local AI runner and playground focused on privacy, Apple Silicon acceleration (MLX and Metal), and cross-platform support.

---

## 📚 Table of Contents

| File | Title | Description |
| :--- | :--- | :--- |
| [`01_associative_memory.md`](./01_associative_memory.md) | **Procedural Associative Memory** | Compact binary graph engine, Zstd/LZ4 disk compression, LFU Hot Cache, and Synaptic Plasticity. |
| [`02_neural_visualizer.md`](./02_neural_visualizer.md) | **Real-Time Neural Visualizer** | 60 FPS Canvas interface with particle physics, node inspection, and Spreading Activation testbench. |
| [`03_chat_and_inference.md`](./03_chat_and_inference.md) | **Chat & Inference Engines** | Token streaming, real-time tok/s metrics, multimodal support (vision), and local Whisper via MLX. |
| [`04_model_manager.md`](./04_model_manager.md) | **Model Manager & HF Hub** | Local directory scanning, GGUF/MLX/Ollama support, and integrated Hugging Face downloader. |
| [`05_mcp_tools.md`](./05_mcp_tools.md) | **Model Context Protocol (MCP)** | Connection to local/remote MCP servers, dynamic tool inspection, autonomous tool calling, and AI humanization. |
| [`06_tauri_backend_architecture.md`](./06_tauri_backend_architecture.md) | **Tauri Backend & Sidecars Architecture** | Rust backend architecture, sidecar management (`llama-server`, `ffmpeg`, `uv`), and IPC contracts. |
| [`07_distribution_and_releases.md`](./07_distribution_and_releases.md) | **Distribution, Packaging & Releases** | Local build guide, CI/CD pipeline (GitHub Actions), macOS Gatekeeper handling, and sidecar bundling. |
| [`08_plugin_system.md`](./08_plugin_system.md) | **Plugin & Extension System** | Modular plugin architecture, cognitive memory decoupling, chat loop hooks, and UI extensions. |
| [`09_cloud_providers.md`](./09_cloud_providers.md) | **Cloud AI Providers** | Unified remote provider integration (Antigravity, OpenAI, OpenRouter, Ollama, Custom OpenAI) and SSE engine. |
| [`10_internationalization_and_locales.md`](./10_internationalization_and_locales.md) | **Internationalization (i18n) & Locales** | Multi-language architecture (pt-BR, en, es), scalable `<select>` switcher, composables, and backend prompts. |
| [`11_remote_gateways_and_tray.md`](./11_remote_gateways_and_tray.md) | **Remote Messaging Gateways & System Tray** | Native background execution, tray menu bar (macOS, Windows, Linux), and Telegram/Discord bots with cognitive memory. |

---

## 🛠️ Technology Stack

* **Frontend**: Nuxt 4 / Vue 3, Tailwind CSS, Lucide Icons, Canvas API (60 FPS Physics).
* **Backend**: Rust (Tauri v2), Tokio (Async runtime), Zstd, LZ4 Flex, Reqwest SSE.
* **Supported Engines & Providers**:
  * **MLX-LM (Local)**: Native inference on Apple Silicon GPU (Metal) with support for Thinking models and VLMs.
  * **llama.cpp / llama-server (Local)**: Quantized `.gguf` models with hardware acceleration.
  * **Ollama (Local / Remote)**: Orchestration for local models or dedicated networked instances.
  * **MLX Whisper (Local)**: Real-time audio transcription on Mac GPU.
  * **Google Antigravity / AGY (Cloud Optional)**: High-speed cloud multimodal models (Gemini / DeepMind).
  * **Official OpenAI (Cloud)**: GPT-4o, o1, and o3-mini models via official API.
  * **OpenRouter (Cloud)**: Multi-provider catalog (Claude, DeepSeek R1, Llama 3.3).
  * **Custom OpenAI-Compatible (Cloud / Remote)**: Groq, Together, DeepSeek, vLLM, and compatible servers.
