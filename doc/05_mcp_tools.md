# 🔌 05. Model Context Protocol (MCP) Integration

The **Model Context Protocol (MCP)** is the open standard for connecting large language models to external tools, databases, APIs, and local file systems.

---

## 1. How It Works in Atena Studio

The Rust backend implements a native client for MCP servers communicating over `stdio` (Standard Input / Output) or `SSE` (Server-Sent Events):

```text
┌──────────────────┐          JSON-RPC          ┌───────────────────┐
│   Atena Studio   │ ◄────────────────────────► │    MCP Server     │
│   (Rust Client)  │   tools/list, tools/call   │  (e.g., SQLite,   │
└────────┬─────────┘                            │   Filesystem)     │
         │                                      └───────────────────┘
         ▼
   Schema Injection
  into Model Prompt
```

---

## 2. Configuring MCP Servers

You can register multiple MCP servers in Atena Studio's Settings by specifying:
* **Server Name / ID**: A user-friendly identifier.
* **Command**: e.g., `npx`, `uvx`, `python3`, `node`.
* **Arguments**: e.g., `["-y", "@modelcontextprotocol/server-filesystem", "/path/to/workspace"]`.
* **Environment Variables**: API keys or custom directory paths.

---

## 3. Execution Lifecycle and Security

1. **Dynamic Tool Inspection**: Atena Studio requests available tools via `tools/list` and formats them into JSON Schema specifications compliant with the LLM.
2. **Tool Calling**: When the model determines an external action is required, it emits a structured function call request.
3. **User Approval Flow**: By default, Atena Studio displays an in-chat card requiring the user to **Approve** or **Reject** tool execution before running, ensuring complete control and safety.
4. **Auto-Execution (Optional)**: Users can enable automatic execution for non-destructive read-only tools.

---

## 4. AI Humanization and Localization of Tool Labels (`translate_mcp_tool_labels`)

### The Challenge of Technical Identifiers
The majority of community MCP servers (such as database connectors, Git tools, file systems, and financial integrations) expose tool and parameter names as raw code identifiers in English (`snake_case` or `kebab-case`), such as `get_upcoming_transactions`, `start_date`, `overdue_days`, and `query_financial_records`.

For end users, these raw identifiers create visual noise and degrade the conversational experience in task-driven chats.

### Hybrid Translation Architecture
To resolve this without compromising speed or system resources, Atena Studio provides the Rust command `translate_mcp_tool_labels`:
* **Background LLM Inference**: Employs the active local or cloud language model to generate natural interface titles (e.g., `get_upcoming_transactions` ➔ *"Upcoming Transactions & Pending Payments"*, `start_date` ➔ *"Start Date"*).
* **Four Operational Safeguards**:
  1. *Concurrency Lock (`MCP_TRANSLATION_LOCK`)*: Prevents redundant concurrent invocations that could compete for GPU/CPU resources.
  2. *Readiness Verification*: Only invokes model inference if the local engine daemon is responsive over HTTP.
  3. *Controlled Batching*: Limits the count of tool schemas processed in a single prompt (maximum of 12 per call).
  4. *Fast Timeout & Heuristic Fallback*: In the event of a timeout (10–15 seconds) or engine unavailability, a deterministic heuristic parser instantly sanitizes identifiers.

### Multilingual Support (`target_locale`)
The command accepts an optional `target_locale: Option<String>` parameter:
* **`en` (English)**: Instructs the LLM and heuristic parser to format names into elegant Title Case and natural UI labels (e.g., *"Upcoming Transactions"*, *"Start Date"*). The heuristic fallback cleans up acronyms (ID, URL, API, MCP) and applies clean capitalization.
* **`es` (Spanish)**: Applies Spanish system instructions and localized heuristic dictionary entries (e.g., *"Transacciones Próximas"*, *"Fecha Inicial"*).
* **`pt-BR` (Default / Baseline)**: Uses Portuguese prompt refinements and comprehensive localized mappings.

---

## 5. Built-in Native Tools and Fine-Grained Disabling (`atena_native`)

In addition to external MCP servers, Atena Studio embeds first-class native tools registered under the built-in MCP server (`atena_native`, transport: `builtin`):
* **`atena_web_search`**: Public web search powered by a two-tier resilient engine: fast stealth HTTP with modern browser client headers (`sec-ch-ua`, `sec-fetch-*`), multi-provider organic card parsing (Bing, DuckDuckGo), with an automatic fallback to an in-app Headless Browser Simulation Engine (`BrowserEngine`) in native WebKit/WebView2 to bypass bot challenges without tokens or external binaries.
* **`atena_fetch_webpage`**: Webpage reader converting HTML to sanitized Markdown text. Includes automatic detection of JavaScript Single-Page Applications (SPAs like React/Vue/Next.js) or bot challenges, dynamically rendering the page via an ephemeral, incognito headless Tauri Webview (`visible: false`) with strict RAII lifecycle cleanup to extract dynamic DOM content.
* **`atena_scratchpad_write` / `atena_scratchpad_read` / `atena_scratchpad_clear`**: Working memory scratchpad for active tasks.
* **`atena_schedule_task` / `atena_list_scheduled_tasks` / `atena_cancel_scheduled_task` / `atena_run_scheduled_task`**: Proactive background routines and automation scheduler.

### Optimizing for Local LLM Latency
When running small or quantized local LLMs (via Ollama, LM Studio, or MLX), injecting numerous tool schemas substantially degrades prompt processing speed, consumes context window limits, and increases first-token latency.
To eliminate this bottleneck:
1. **MCP Visibility**: The `atena_native` server is displayed alongside external servers in the MCP tools panel and Settings with a `BUILTIN` badge.
2. **Granular Control**: Each of the 9 native tools can be toggled on or off individually via the MCP tool inspection modal, and the entire `atena_native` server can be toggled inactive.
3. **Prompt Filtering**: Disabled native tools are excluded from the system prompt schema injection during inference and are rejected at the backend execution gateway if invoked.
4. **Memory and Skills Isolation**: Associative memory tools (`atena_search_memory`, `atena_search_episodes`, `atena_read_episode`) and procedural skills (`run_command`, `run_skill_script`) have dedicated toggles in the Cognitive Memory settings and are kept out of the MCP tools modal to avoid clutter and redundancy.

Refer to `doc/12_autonomous_agent_and_scheduler.md` for architectural details.

