# 🏗️ 06. Tauri Backend & Sidecars Architecture

Atena Studio is engineered as a high-performance native desktop application using **Tauri v2** and **Rust**.

---

## 1. Backend Directory Structure (`src-tauri`)

```text
src-tauri/
├── Cargo.toml            # Rust dependencies (tauri, tokio, zstd, lz4_flex, reqwest, etc.)
├── binaries/             # Embedded sidecar binaries organized by target architecture
├── src/
│   ├── main.rs           # Desktop application entry point
│   ├── lib.rs            # AppState registration, Tauri commands, and lifecycle hooks
│   ├── bin/
│   │   └── memory_demo.rs # Standalone CLI binary for memory engine testing
│   ├── core/
│   │   ├── config.rs     # User configuration models
│   │   ├── hardware.rs   # Hardware monitor for Apple Silicon GPU, CPU, and unified RAM
│   │   ├── memory.rs     # Binary format specification for Nodes, Edges, and .atena header
│   │   ├── mcp.rs        # MCP protocol data structures
│   │   └── model.rs      # Inference types, chat message structs, and performance metrics
│   └── services/
│       ├── backend.rs    # Process orchestrator (MLX, llama-server, Ollama)
│       ├── db.rs         # Embedded SQLite database service (~/.atena/atena.db)
│       ├── downloader.rs # Chunked Hugging Face download manager
│       ├── engine.rs     # Chat execution engine and token streaming pipelines
│       ├── mcp_service.rs# MCP stdio and SSE connection manager
│       ├── memory_engine.rs # Binary associative memory engine and Spreading Activation
│       ├── runtime.rs    # Isolated Python and uv virtual environment manager
│       └── scanner.rs    # Local disk model scanner
```

---

## 2. Embedded Sidecar Binaries (`binaries/`)

To guarantee that the application functions **100% offline without requiring compilation toolchains on the user's system**, necessary executables are provisioned in `src-tauri/binaries/`:

| Sidecar Binary | Purpose | Target Platform |
| :--- | :--- | :--- |
| `llama-server` | C++ HTTP inference server for `.gguf` models with Metal acceleration | `aarch64-apple-darwin` / `x86_64` |
| `ffmpeg` | Fast audio decoding and conversion to PCM float32 16kHz | `aarch64-apple-darwin` |
| `uv` | Ultra-fast Python package and environment manager for MLX Whisper | `aarch64-apple-darwin` |

The `scripts/setup-sidecars.sh` script verifies that all required binaries are present prior to packaging or executing `tauri dev`.

---

## 3. IPC Contracts (Frontend ↔ Rust Invocation)

Communication between the Nuxt/Vue frontend and the Rust core occurs via asynchronous Tauri IPC with channel streaming (`Channel<T>`):

* `stream_chat`: Streams response text chunks, reasoning tokens, and TPS metrics over an active IPC channel.
* `memory_get_full_graph`: Emits the complete node and edge graph representation for Canvas rendering.
* `scan_models`: Returns a comprehensive list of locally discovered models.
* `start_hf_download`: Launches an asynchronous background download job with real-time progress callbacks.
* `db_*`: CRUD operations for conversations, messages, personas, and application settings backed by SQLite.

---

## 4. Headless Server Mode (Browser Access & Remote Execution)

Atena Studio supports a **headless web server mode** inspired by the architecture of VS Code Server / code-server:

```bash
# Start Atena Studio in headless web server mode
./src-tauri/target/debug/app --server --port 7860 --host 0.0.0.0
```

### Supported CLI Flags:
* `--server`: Boots the Tokio async runtime with Axum instead of the native desktop window.
* `--port <port>`: Port to listen on (default: `7860`).
* `--host <address>`: Host interface to bind (default: `127.0.0.1`, use `0.0.0.0` for LAN access).
* `--token <secret>`: Token authentication key (auto-generated if omitted).
* `--static-dir <path>`: Custom directory for static frontend files.

### Web Architecture:
1. **Axum HTTP/WS Server (`src-tauri/src/server/`):**
   * Serves pre-rendered Nuxt SPA assets from `frontend/.output/public` with history-mode fallback.
   * Exposes `POST /api/ipc/invoke` for standard unary IPC requests.
   * Exposes `GET /api/ipc/ws` for bi-directional WebSocket RPC and streaming (`stream_chat`, server progress callbacks).
2. **Frontend IPC Web Adapter (`frontend/app/utils/ipcAdapter.ts`):**
   * Polyfills `window.__TAURI_INTERNALS__` when running inside regular web browsers (Safari, Chrome, Firefox, mobile).
   * Automatically routes `invoke()` and `Channel` subscribers over WebSocket messages with zero changes required in Vue components.

---

## 5. Embedded SQLite Storage (`~/.atena/atena.db`)

To ensure **full parity between the Native Desktop App and the Headless Web Server mode**, persistence is decoupled from browser `localStorage` (which is origin-bound and restricted to 5 MB) and managed directly by the Rust backend using `rusqlite` (bundled C sources, zero external runtime dependencies).

### Database Engine Configuration:
* **Storage Location:** `~/.atena/atena.db` (dynamically contracted / expanded across platforms).
* **Concurrency:** `PRAGMA journal_mode = WAL;`, `PRAGMA synchronous = NORMAL;`, `PRAGMA busy_timeout = 5000;`.
* **Integrity:** `PRAGMA foreign_keys = ON;`.

### Core Schemas:
* **`sessions`**: `id`, `title`, `model_id`, `model_name`, `project_id`, `created_at`, `updated_at`, `is_private`, `pinned`, `archived`, `archived_at`, `metadata`.
* **`messages`**: `id`, `session_id` (FK cascade), `role`, `content`, `thinking_content`, `tool_calls`, `tool_call_id`, `images`, `attachments`, `metrics`, `tokens_count`, `generation_speed_tps`, `timestamp`, `position`, `metadata`.
* **`personas`**: `id`, `name`, `tag`, `icon_name`, `color`, `description`, `system_prompt`, `is_custom`, `updated_at`.
* **`persona_overrides`**: `persona_id`, `system_prompt`, `updated_at`.
* **`app_settings`**: Key-value pair table (`key`, `value`, `updated_at`) for generation parameters, project hierarchies, and UI selections.

---

## 6. Docker & Containerization Architecture

Atena Studio provides first-class containerization support for cloud, home server, and headless server environments via **Docker** and **Docker Compose**.

### Container Execution Model:
* **Headless Server Mode:** The container image runs the compiled binary in server mode (`atena --server --host 0.0.0.0 --port 7860`).
* **Multi-Stage Build:**
  1. `frontend-builder` (`node:22-bookworm-slim`): Compiles the Nuxt SPA using `pnpm run generate`.
  2. `backend-builder` (`rust:1.84-bookworm`): Compiles the Rust backend with Tauri v2 Linux dependencies and provisions Linux sidecars (`llama-server`, `ffmpeg`, `uv`).
  3. `runtime` (`debian:bookworm-slim`): Minimal runtime container running as dedicated non-root user `atena` (UID 1000).
* **Persistence Volume:** Maps `/home/atena/.atena` to a persistent Docker volume (`atena_data`), guaranteeing that `atena.db`, the cognitive memory graph (`brain/`), skills, and plugins persist indefinitely across container upgrades.
* **Environment Variables:**
  * `ATENA_HOST`: Bind address (default `0.0.0.0` in container).
  * `ATENA_PORT`: Listening port (default `7860`).
  * `ATENA_TOKEN`: Optional authentication token.
  * `ATENA_STATIC_DIR`: Directory serving pre-rendered Nuxt SPA files (default `/app/public`).

---

## 7. System Tray & Remote Messaging Gateways

For detailed information on the native background daemon, system tray menu, and external bot gateways (Telegram & Discord), consult [`doc/11_remote_gateways_and_tray.md`](./11_remote_gateways_and_tray.md).
