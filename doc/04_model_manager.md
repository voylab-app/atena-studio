# 📦 04. Model Manager and Hugging Face Hub

The **"Models"** view allows you to manage your local artificial intelligence model library and download new models directly from the Hugging Face Hub.

---

## 1. Local Model Discovery and Scanning

The built-in Rust scanner automatically indexes configured directories (default: `~/.atena/models` or custom folders):

* **Automatic Format Detection**: Detects whether a model is `.gguf`, `MLX-LM`, `Safetensors`, or `Ollama`.
* **Extracted Metadata**:
  * Name and architecture (e.g., `Qwen`, `Llama`, `Gemma`, `DeepSeek`);
  * Quantization level (e.g., `4-bit`, `8-bit`, `Q4_K_M`, `Q8_0`);
  * Disk footprint (GB);
  * Native context window size (e.g., `8k`, `32k`, `128k`);
  * Capability badges: 🧠 Thinking, 👁️ Vision, 🔧 MCP Tools.

---

## 2. Integrated Hugging Face Downloader

Atena Studio includes a native client to search and download models without leaving the application:

1. Click the **"Search on Hugging Face"** button on the Models tab;
2. Query models with search terms like `gemma-2`, `qwen2.5`, `deepseek-r1-distill`, or `mlx-community`;
3. Inspect repository details (available quantizations, file sizes, and descriptions);
4. Select target files and click **"Start Download"**;
5. **Real-Time Download Tracking**:
   * Completion percentage (%);
   * Download transfer rate (MB/s);
   * Estimated Time of Arrival (ETA);
   * Ability to pause and cancel active downloads at any point.

---

## 3. Dynamic Loading and Unloading

* **Load Model**: Click on any model card. Atena Studio initiates the corresponding backend daemon (`llama-server` or the MLX Python runtime) in the background.
* **Auto-Load Last Model**: An optional setting restores the previously active model upon launching the app.
* **Unload Model**: Instantly releases 100% of allocated VRAM and system memory.
