# 09. Cloud Artificial Intelligence Providers — Atena Studio

**Atena Studio** integrates a unified native architecture for cloud artificial intelligence providers and remote servers, designed to coexist harmoniously with local inference backends (MLX-LM and llama.cpp).

---

## 1. Philosophy: Unification Without Fragmentation

The application consolidates all cloud services under a single configuration interface and streaming pipeline:
- **Unified Settings Panel:** All providers (Google Antigravity, OpenAI, OpenRouter, Ollama, and custom endpoints) are configured under **"Cloud Providers"** (icon `Cloud`), preventing scattered menus across the sidebar.
- **Credential Isolation:** API keys are never relayed to external telemetry servers or stored in log files. Storage resides strictly on the local machine in `~/.atena/config.json`.
- **Generic SSE Engine (`BackendType::CloudOpenAi`):** Any endpoint adhering to the standard OpenAI `/v1/chat/completions` protocol (OpenAI, OpenRouter, Groq, Together, DeepSeek API, vLLM) shares the same asynchronous Rust streaming orchestrator.

---

## 2. Supported Native Providers

### 2.1 Google Antigravity (AGY)
- **Protocol:** Stdio IPC with the native `agy` CLI (`--input-format=stream-json --output-format=stream-json`).
- **Features:** Interactive terminal authentication, daily/weekly quota monitoring, and access to DeepMind models (Gemini 3.8 Flash, Pro, and Claude Sonnet).
- **Cognitive Effort:** Configurable reasoning levels: `low`, `medium`, and `high`.

### 2.2 Official Google Gemini (Google AI Studio)
- **Endpoint:** `https://generativelanguage.googleapis.com/v1beta/openai/chat/completions`.
- **Authentication:** `Authorization: Bearer <AIzaSy...>`.
- **Models:** Gemini 2.5 Pro, Gemini 2.5 Flash, Gemini 2.5 Flash-Lite, Gemini 1.5 Pro, and Gemini 1.5 Flash.
- **Features:** Context windows up to 2 million tokens, high throughput, reasoning capability, and multimodal vision.

### 2.3 OpenAI
- **Endpoint:** `https://api.openai.com/v1/chat/completions` (with custom base URL support).
- **Authentication:** `Authorization: Bearer <sk-...>`.
- **Models:** Dynamic model discovery via `GET /v1/models` with standard support for `gpt-4o`, `gpt-4o-mini`, `o1`, and `o3-mini`.
- **Features:** SSE streaming, reasoning token extraction, MCP tool calling, and vision.

### 2.4 Groq Cloud (LPU Inference Engine)
- **Endpoint:** `https://api.groq.com/openai/v1/chat/completions`.
- **Authentication:** `Authorization: Bearer <gsk_...>`.
- **Models:** Dynamic model discovery via `GET https://api.groq.com/openai/v1/models`, with curated defaults for Meta Llama 3.3 70B Versatile (`llama-3.3-70b-versatile`), Meta Llama 3.1 8B Instant (`llama-3.1-8b-instant`), DeepSeek R1 Distill Llama 70B (`deepseek-r1-distill-llama-70b`), Mixtral 8x7B (`mixtral-8x7b-32768`), Gemma 2 9B (`gemma2-9b-it`), and Meta Llama 3.2 11B Vision (`llama-3.2-11b-vision-preview`).
- **Features:** Ultra-low latency streaming powered by Groq LPU™ (Language Processing Unit) tensor chips, OpenAI-compatible SSE streaming, function calling, vision, and dynamic status validation.

### 2.5 OpenRouter
- **Endpoint:** `https://openrouter.ai/api/v1/chat/completions`.
- **Authentication:** `Authorization: Bearer <sk-or-v1-...>`, with `HTTP-Referer` and `X-Title` headers.
- **Models:** Broad access to hundreds of open-source and proprietary models (Anthropic Claude 3.5 Sonnet, DeepSeek R1, Meta Llama 3.3, Google Gemini 2.0).
- **Validation:** Dedicated key verification endpoint `GET /auth/key`.

### 2.6 Ollama (Local & Remote)
- **Endpoint:** `http://<host>:<port>/api/chat` and `/api/tags`.
- **Configuration:** Dynamic host (`127.0.0.1` or LAN IP) and configurable port (default: `11434`).
- **Health Check & Synchronization:** Live status ping and local model catalog synchronization.

### 2.7 Custom OpenAI-Compatible Providers (Multiple)
- **Purpose:** Connect multiple endpoints simultaneously for private or commercial infrastructures:
  - **Cloud APIs:** Together AI, DeepSeek API, Mistral AI, Fireworks, etc.
  - **Local/LAN Servers:** LM Studio (port 1234), vLLM, Text Generation WebUI, LocalAI.
- **Management:** Add, edit, toggle, and remove individual endpoints with per-endpoint connectivity testing.

---

## 3. Data Structures and Persistence (`AppConfig`)

Cloud settings are saved in `~/.atena/config.json` under the `cloud_providers` object:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CloudProvidersConfig {
    pub gemini_enabled: bool,
    pub gemini_api_key: String,
    pub gemini_selected_model: Option<String>,

    pub groq_enabled: bool,
    pub groq_api_key: String,
    pub groq_selected_model: Option<String>,

    pub openai_enabled: bool,
    pub openai_api_key: String,
    pub openai_base_url: Option<String>,
    pub openai_selected_model: Option<String>,

    pub openrouter_enabled: bool,
    pub openrouter_api_key: String,
    pub openrouter_selected_model: Option<String>,

    pub antigravity_enabled: bool,
    pub antigravity_default_effort: String,

    pub custom_providers: Vec<CustomEndpointConfig>,
    pub disabled_models: Vec<String>,
    pub cached_models: HashMap<String, Vec<DiscoveredModel>>,
}
```

---

## 4. Execution Lifecycle and Rust Streaming Pipeline

```text
┌────────────────────────────────────────────────────────┐
│                    Atena Frontend                      │
│     (invoke 'stream_chat' with Cloud/OpenAI model)     │
└──────────────────────────┬─────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────┐
│            BackendManager::execute_stream              │
│       (BackendType::CloudOpenAi / reqwest SSE)         │
└──────────────────────────┬─────────────────────────────┘
                           │
              ┌────────────┴────────────┐
              ▼                         ▼
   ┌───────────────────────┐ ┌───────────────────────┐
   │ Real-Time Delta &     │ │  Thinking & MCP       │
   │ Token Parsing         │ │  Tool Call Extraction │
   └──────────┬────────────┘ └──────────┬────────────┘
              │                         │
              └────────────┬────────────┘
                           ▼
┌────────────────────────────────────────────────────────┐
│          Efficiency & Throughput Calculations          │
│       (Tokens/s, Prefill TPS, Finish Reason, Cache)    │
└──────────────────────────┬─────────────────────────────┘
                           │
                           ▼
┌────────────────────────────────────────────────────────┐
│             Chunk Emission via IPC Channel             │
│            (ChatChunkPayload -> Frontend)              │
└────────────────────────────────────────────────────────┘
```

---

## 5. Available Tauri Commands

| Command | Signature | Description |
|---|---|---|
| `test_cloud_provider_connection` | `(provider, apiKey, baseUrl) -> Result<String, String>` | Validates API credentials and checks connection latency asynchronously. |
| `fetch_provider_models` | `(provider, apiKey, baseUrl) -> Result<Vec<DiscoveredModel>, String>` | Queries remote `/models` endpoint directly to discover and return model capabilities. |
| `get_cloud_providers_config` | `() -> Result<CloudProvidersConfig, String>` | Loads dedicated cloud providers configuration from disk. |
| `save_cloud_providers_config` | `(cloudProviders) -> Result<(), String>` | Persists dedicated cloud providers configuration to disk. |
| `get_app_config` | `() -> Result<AppConfig, String>` | Reads full application configuration from disk. |
| `save_app_config` | `(config) -> Result<(), String>` | Persists updated configuration into `~/.atena/config.json`. |
| `get_agy_usage` | `() -> Result<AgyUsageData, String>` | Queries weekly model quotas via Antigravity CLI. |
| `start_agy_login` | `() -> Result<(), String>` | Spawns a terminal window for Google Cloud interactive authentication. |
