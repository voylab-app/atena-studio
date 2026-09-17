# 💬 03. Chat and Inference Engines

The Atena Studio Chat module is engineered to provide a fluid, private, and high-speed experience for large language models and multimodal vision directly on your local computer.

---

## 1. Supported Execution Engines

Atena Studio supports strictly local inference (with native hardware acceleration and full privacy) as well as optional high-speed cloud connections:

| Backend | Model Format | Type / Acceleration | Description |
| :--- | :--- | :--- | :--- |
| **MLX-LM** | MLX folders / Hugging Face | **Local** (Apple Silicon GPU / Metal) | Apple's native framework for M1/M2/M3/M4 Macs, supporting Reasoning (Thinking) models and VLMs (Vision). |
| **llama.cpp** | `.gguf` files | **Local** (Metal / CPU / CUDA) | Lightweight execution via the embedded `llama-server` sidecar with Prompt Caching and Flash Attention. |
| **Ollama** | Ollama models | **Local** (Localhost API `11434`) | Transparent integration with local or networked Ollama instances. |
| **Google Antigravity (AGY)** | Cloud models (Gemini / DeepMind) | **Cloud** (Google DeepMind Cloud) | Rapid connection via `agy` CLI with real-time quota monitoring and zero local RAM footprint. |

---

## 2. Response Streaming and Real-Time Metrics

The interface calculates generation performance token by token in real time:

* **Generation Speed (TPS)**: Displays the live token output rate in tokens per second ($tok/s$).
* **Token Counts**: Prompt input tokens, reasoning/thinking tokens, and completion output tokens.
* **Memory Consumption**:
  * **AI Model Memory (VRAM / Unified Memory)**: Exact memory allocated by the loaded model.
  * **Global System RAM**: Visual breakdown between AI consumption, other applications, and free system memory.

---

## 3. Multimodality and File Attachments

The chat interface supports various file formats via drag-and-drop or the attachment button:

### 3.1 Multimodal Vision (VLMs)
* Supported image formats: `.png`, `.jpg`, `.jpeg`, `.webp`, `.gif`, `.svg`.
* Vision-capable models (such as Qwen2-VL, Gemma 3, Llama 3.2-Vision) process images directly on the GPU.

### 3.2 Audio Transcription (Local GPU Whisper)
* Supported audio formats: `.mp3`, `.wav`, `.m4a`, `.ogg`, `.flac`.
* On macOS Apple Silicon, Atena Studio runs **MLX Whisper** directly on the GPU, transcribing speech to text within seconds without transmitting audio data to external servers.

### 3.3 Documents and Code Snippets
* Ingestion of `.pdf`, `.txt`, `.md`, `.json`, `.csv`, `.py`, `.rs`, `.ts`, `.vue`, and other source files directly into the conversation context window.

---

## 4. Integration with Associative Memory

During conversations:
1. The backend inspects user message tokens for matches in the memory graph;
2. When relevant matches are found, it silently injects an `[ASSOCIATIVE ACTIVE MEMORY: ...]` block into the System Prompt;
3. It analyzes factual statements to continuously learn and reinforce connections in `~/.atena/memory.atena`.

---

## 5. Conversation Lifecycle and Archiving

To maintain a clean and focused workspace, Atena Studio offers comprehensive chat organization capabilities:

* **Chat Archiving**:
  * Users can archive individual conversations via the session hover action or bulk-archive multiple conversations using the multi-selection toolbar.
  * Archived chats are automatically hidden from active Recents and Project trees in the primary sidebar.
* **Archived Chats Modal**:
  * Accessible from the sidebar and from **Settings > General**, the dedicated modal allows searching, previewing message history snippets, restoring (unarchiving), and permanently deleting archived sessions.
* **Archived State & Seamless Resumption**:
  * When navigating to an archived conversation, an informative banner is displayed with a one-click **Unarchive** action.
  * Sending a new message to an archived conversation automatically unarchives the session, restoring it to the active list without interruption.
* **Unified SQLite Persistence**:
  * Conversations and message histories are persisted directly to the embedded SQLite database (`~/.atena/atena.db`), ensuring transactional consistency, eliminating browser localStorage quota limits, and maintaining synchronized real-time state across desktop webviews and web browsers.

