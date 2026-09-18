# 🤖 12. Economic Autonomous Agent, Background Scheduler & Native Web Tools

Atena Studio is engineered to function not only as a reactive conversational assistant, but as an **economic, proactive, multi-step autonomous agent**. 

This module delivers complete agentic behavior while maintaining a **100% economic resource footprint**: consuming **0% idle CPU and 0 MB VRAM** when waiting for tasks or sleeping in the background.

---

## 1. Architectural Pillars

The autonomous subsystem comprises four native components built directly into the Rust (`src-tauri`) core:

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                      Atena Autonomous Subsystem                         │
├────────────────────────────────┬────────────────────────────────────────┤
│ 1. Multi-Step ReAct Loop       │ 2. Rust Background Scheduler           │
│    • Think ➔ Act ➔ Observe     │    • Tokio Timer Daemon (0% idle CPU)  │
│    • Hard Step Budget (1..10)  │    • SQLite Persistence (scheduled_tasks│
│    • Automatic Error Recovery  │    • Native OS & Webview Notifications │
├────────────────────────────────┼────────────────────────────────────────┤
│ 3. Native Web Tools (Zero MCP) │ 4. Session Task Scratchpad             │
│    • atena_web_search (DDG)    │    • Ephemeral Working Memory          │
│    • atena_fetch_webpage       │    • In-Memory Key-Value Notes         │
│    • Clean Markdown Extractor  │    • Graph Protection (Zero Pollution) │
└────────────────────────────────┴────────────────────────────────────────┘
```

---

## 2. Multi-Step Autonomous ReAct Loop (`AutonomousAgentRunner`)

When users submit goal-oriented requests (such as *"Investigate why test X failed, fix the code, and rerun"*), Atena operates in an autonomous ReAct loop:

$$\text{User Goal} \longrightarrow \Big[ \text{Think} \longrightarrow \text{Tool Action} \longrightarrow \text{Observation / Output} \Big]^N \longrightarrow \text{Conclusion}$$

### 2.1 Hard Safety Budget Limit
To prevent runaway loops, infinite recursion, or token drain, the runner enforces a configurable strict budget ceiling ($N \in [1, 15]$, default: **6 iterations**).
If the step budget is exhausted before natural completion, the runner issues an automatic synthesis turn instructing the model to report all accomplished actions, pending issues, and findings.

### 2.2 Autonomous Error Reflection
When a script, compiler, or CLI tool yields an error or a non-zero exit code:
1. The error message is captured directly in JSON (`{ "status": "error", "error": "..." }`);
2. The runner injects this output into the observation prompt: `[Observation / Tool Output - Step K of N]`;
3. The model inspects the error and self-corrects in the next step without prompting the user to manually intervene.

### 2.3 User Interruption and Safety
Users retain full control via the real-time **Stop / Abort** action in the desktop interface or via cancellation tokens in the Rust backend.

---

## 3. Background Task Scheduler (`BackgroundScheduler`)

While remote gateways (Telegram, Discord) allow remote messaging, the **Background Scheduler** provides proactive capability without user prompting:

### 3.1 Resource Economics
* Implemented as an asynchronous Tokio task in `src-tauri/src/services/scheduler.rs`;
* While waiting for task deadlines, the worker sleeps via Tokio timers. It uses **0% CPU and 0 MB VRAM**;
* It only wakes up at scheduled intervals to check due tasks in SQLite (`~/.atena/atena.db`).

### 3.2 Supported Routine Types
1. **Autonomous Prompt (`autonomous_prompt`)**:
   Wakes up the active model to generate a daily morning briefing, summarize recent memory changes, or compile project digests.
2. **Procedural Skill Execution (`skill`)**:
   Runs terminal diagnostic commands, folder syncs, or automated scripts.
3. **Associative Memory Sleep Cycle (`memory_sleep`)**:
   Runs nocturnal consolidation of the cognitive memory graph (`~/.atena/memory.atena`), pruning stale associations and reinforcing frequent pathways.

### 3.3 Schedule Formats Supported
* Standard 5-field cron: `"0 9 * * *"` (daily at 9:00 AM), `"*/30 * * * *"` (every 30 minutes);
* Convenience intervals: `"@hourly"`, `"@daily"`, `"@weekly"`, `"@every 1h"`, `"@every 30m"`;
* Explicit second intervals: `"interval:3600"`.

### 3.4 Autonomous AI Scheduling & On-Demand Execution via Chat
Users do not need to configure tasks manually. The AI model has direct access to built-in tools:
* **`atena_schedule_task(name, cron_expr, action_type, prompt, description, run_immediately, delivery_channel, delivery_target)`**: Enables the AI to autonomously register proactive routines when instructed by the user in conversation (e.g., *"Atena, schedule a morning tech briefing every day at 9 AM and run it now to test"*). If `run_immediately: true`, the newly created routine runs immediately and returns execution feedback.
* **`atena_run_scheduled_task(identifier)`**: Immediately triggers and executes an existing scheduled task or routine on demand for testing or instant reports (matched by exact ID or name substring).
* **`atena_list_scheduled_tasks()`**: Allows the AI to query all active routines and report back to the user.
* **`atena_cancel_scheduled_task(identifier)`**: Allows the AI to cancel or remove scheduled tasks dynamically.

### 3.5 Multi-Channel Routine Delivery (`delivery_channel`)
Routines can be configured with specific delivery targets depending on user preference and workflow needs:
* **`chat` (Default)**: Creates a clean new session in the desktop chat sidebar (e.g. `📅 Morning Briefing (18/09 09:00)`) with full conversational messages and interactive tool cards.
* **`telegram`**: Directly delivers the formatted briefing to the user's Telegram chat via proactive Bot API messaging without cluttering the desktop chat history. When a task is scheduled while conversing via Telegram, Atena automatically defaults `delivery_channel` to `"telegram"` and binds `delivery_target` to the current `chat_id`.
* **`both`**: Delivers simultaneously to both the desktop chat interface and proactive Telegram notification.
* **`silent`**: Runs completely in the background without creating chat sessions or external notifications, recording the complete run details and output in SQLite for auditing.

### 3.6 Comprehensive Tool Execution Audit Log & Trace (`scheduled_task_runs`)
Every routine execution (whether `chat`, `telegram`, `both`, or `silent`) is persistently audited in SQLite:
* **Run Metadatas**: Execution ID, timestamp, task name, delivery channel, exit status (`success` or `error`), and step count;
* **Tool Usage Summary**: Array of distinct tool names utilized during the autonomous run;
* **Detailed Step-by-Step Trace (`steps_detail`)**: Full timeline recording each tool called, exact JSON arguments passed, execution status, and tool observation/return value;
* **Session Deep-Linking**: When a routine delivers to `chat` or `both`, `session_id` is recorded, allowing instant 1-click navigation from Settings straight into the interactive chat session.

---

## 4. Native Web Tools (`WebTools`)

To liberate users from configuring external Node.js, Python, or heavy MCP servers for web searches:

### 4.1 `atena_web_search`
* Sends lightweight HTTP queries via `reqwest` to a resilient multi-provider search pipeline (Bing, DuckDuckGo);
* Requires **zero API keys** and zero external binaries;
* Decodes wrapped redirect URLs and extracts sanitized URLs, titles, and snippets into structured JSON results:
  ```json
  [
    {
      "title": "Rust 1.85.0 released",
      "url": "https://blog.rust-lang.org/...",
      "snippet": "The Rust team is happy to announce a new version of Rust..."
    }
  ]
  ```

### 4.2 `atena_fetch_webpage`
* Fetches raw webpage HTML;
* Strips `<script>`, `<style>`, `<nav>`, `<footer>`, `<svg>`, and HTML comments;
* Converts headings, paragraphs, bullet lists, code blocks, and tables into clean, readable Markdown;
* Limits response characters (default: 8,000 characters) to prevent context window saturation.

---

## 5. Session Task Scratchpad (`SessionScratchpadManager`)

Transient operational details (such as temporary stack traces, intermediate grep logs, or bash commands) should not pollute the permanent associative knowledge graph.

* **Ephemeral Working Memory**: Managed in-memory per session in `SessionScratchpadManager`;
* **Native Tool Access**:
  * `atena_scratchpad_write(key, content)`: Saves plan checklists, intermediate hypotheses, or compiler errors;
  * `atena_scratchpad_read(key)`: Retrieves temporary notes;
  * `atena_scratchpad_clear()`: Flushes the scratchpad once the overarching task is resolved;
* **Prompt Injection**: Injected dynamically into system context during multi-step runs as `[TASK SCRATCHPAD - EPHEMERAL WORKING MEMORY]`.
