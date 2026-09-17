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
