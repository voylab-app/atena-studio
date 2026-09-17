# 08. Plugin and Extension System — Atena Studio

Atena Studio features a modular **Plugin and Extension Architecture**, enabling developers and the community to customize and expand application capabilities:
- **100% Native Core:** Foundational features such as **Associative Cognitive Memory** (neural graph, continuous learning, wake journal) and **Cloud Providers** (Antigravity, OpenAI, OpenRouter, Ollama) are built directly into the core engine.
- **Chat Loop Lifecycle:** Intercept, enrich, and post-process prompts, streaming chunks, and responses via lifecycle hooks.
- **UI Extension Points:** Create sidebar tabs, settings panels, and chat action buttons.
- **Decentralized Extensibility:** Support for third-party plugins installed in the user directory (`~/.atena/plugins/`).

---

## 1. Philosophy: Clean Architecture and Native Core

Atena Studio prioritizes high performance and loose coupling:
- The core application is autonomous and does not require third-party plugins for its primary functions.
- **Associative Cognitive Memory** is built-in and **disabled by default** to ensure maximal speed and light resource utilization; it can be toggled in **Settings > Inference & Thinking**.
- **Cloud Providers** are managed through a unified interface in **Settings > Cloud Providers**.
- Community plugins in `~/.atena/plugins/` provide a pathway for custom tools, integrations, workflows, and dedicated interface views.

### 1.1 Temporal & Timezone Context: Native Core Feature
- Management of **Temporal Context**, date/time formatting, timezone resolution, and timestamp prompt injection for KV Cache optimization is a **100% native Atena Studio feature** (configured in General Settings).

### 1.2 Cognitive Memory Control in Inference & Thinking
1. **Disabled by Default:**
   - The neural graph consumes zero additional memory and runs no asynchronous lookups.
   - The inference loop operates at peak throughput with zero token overhead.
   - The "Memory" tab remains hidden in the sidebar.
2. **When Enabled by User:**
   - The "Memory" tab becomes visible in the sidebar for interactive graph inspection.
   - The model queries associative memory and records events in the session diary.

---

## 2. Anatomy of a Plugin

Local plugins are stored in the Atena Studio data folder:

- **Linux / macOS:** `~/.atena/plugins/<plugin-id>/`
- **Windows:** `%USERPROFILE%\.atena\plugins\<plugin-id>\`

Every plugin must contain a manifest file named `plugin.json`.

### 2.1 Example `plugin.json`

```json
{
  "id": "my-prompt-filter",
  "name": "Prompt Enhancer",
  "version": "1.0.0",
  "description": "Appends guidelines and professional formatting before sending messages to the model.",
  "author": "Atena Community",
  "icon": "Sparkles",
  "enabled": true,
  "category": "chat_enhancer",
  "hooks": [
    "before_chat",
    "on_system_prompt",
    "after_chat_turn"
  ],
  "views": [
    {
      "id": "prompt-enhancer-tab",
      "title": "Templates",
      "icon": "Sparkles",
      "location": "sidebar",
      "description": "Custom prompt template manager."
    }
  ],
  "settings": {
    "strict_mode": true,
    "max_context_tokens": 512
  },
  "settings_schema": [
    {
      "id": "strict_mode",
      "label": "Strict Mode",
      "description": "Enforces exact template matching.",
      "type": "boolean",
      "default": true
    },
    {
      "id": "max_context_tokens",
      "label": "Max Context Tokens",
      "description": "Maximum tokens injected into prompt by this plugin.",
      "type": "number",
      "default": 512,
      "min": 64,
      "max": 4096,
      "step": 64
    },
    {
      "id": "template_tone",
      "label": "Template Tone",
      "type": "select",
      "default": "professional",
      "options": [
        { "label": "Professional", "value": "professional" },
        { "label": "Casual", "value": "casual" },
        { "label": "Academic", "value": "academic" }
      ]
    }
  ]
}
```

### 2.2 Manifest Fields

| Field | Type | Description |
|---|---|---|
| `id` | `string` | Unique identifier in kebab-case format (e.g., `atena-plugin-memory`). |
| `name` | `string` | Human-readable title displayed in the Plugin Manager. |
| `version` | `string` | Semantic version string (e.g., `1.0.0`). |
| `description` | `string` | Concise explanation of plugin functionality. |
| `author` | `string` | Neutral name of author or maintainer organization. |
| `icon` | `string` | Lucide icon name (e.g., `Brain`, `Blocks`, `Sparkles`, `Sliders`). |
| `enabled` | `boolean` | Initial activation status upon installation. |
| `category` | `string` | `core_extension`, `ui_extension`, `chat_enhancer`, `tool_provider`, `cloud_provider`, or `custom`. |
| `entry` | `string` | (Optional) Relative path to bundled frontend JavaScript/TypeScript script (e.g., `dist/index.js`). |
| `native_entry` | `string` | (Optional) Relative path to compiled native Rust binary (e.g., `bin/runner`). |
| `readme` | `string` | (Optional) Relative path to overview markdown file (e.g., `README.md`). |
| `doc` | `string` | (Optional) Relative path to documentation file or folder (e.g., `doc/ARCHITECTURE.md`). |
| `locales` | `string` | (Optional) Relative path to i18n translation folder (e.g., `locales/`). |
| `repository` | `string` | (Optional) Repository URL where source code is maintained. |
| `license` | `string` | (Optional) Distribution license identifier (e.g., `MIT`, `Apache-2.0`). |
| `hooks` | `string[]` | List of chat loop lifecycle hooks subscribed to. |
| `views` | `array` | UI tabs or view extensions registered by the plugin. |
| `settings` | `object` | Default key-value configuration values. |
| `settings_schema` | `array` | Schema of settings fields for automatic UI rendering in **Settings**. |

### 2.3 Dynamic Settings Menus & Form Fields (`settings_schema`)

When a plugin specifies a `settings_schema`:
1. **Automated Navigation Item:** A menu entry bearing the plugin's icon and title is automatically injected into the **Settings** navigation panel.
2. **Dynamic UI Rendering:** Atena Studio generates matching form inputs:
   - `boolean`: macOS-styled toggle switches.
   - `text` / `password`: Text input fields with placeholder and masking support.
   - `number`: Numeric counter inputs with increment/decrement steppers (`-` and `+`) honoring `min`, `max`, and `step`.
   - `select`: Dropdown selectors populated from `options: [{ label, value }]`.
3. **Reactive Persistence:** Changes are persisted to `~/.atena/plugins.json` and passed to the active plugin instance.
4. **Clean Teardown:** Disabling the plugin immediately unmounts its settings panel and menu entry from the UI.

---

## 3. Chat Loop Lifecycle Hooks

Developers can hook into four distinct stages of the conversation cycle:

```text
                  ┌──────────────────────┐
                  │     User Message     │
                  └──────────┬───────────┘
                             │
                  ▼──────────┴───────────▼
                  [ Hook: before_chat ]
                  Modifies messages and inference parameters
                             │
                  ▼──────────┴───────────▼
                  [ Hook: on_system_prompt ]
                  Injects context into system prompt
                             │
                  ▼──────────┴───────────▼
                  Start of Response Streaming
                             │
                  ▼──────────┴───────────▼
                  [ Hook: on_chunk ]
                  Intercepts each emitted token chunk
                             │
                  ▼──────────┴───────────▼
                  End of Response Streaming
                             │
                  ▼──────────┴───────────▼
                  [ Hook: after_chat_turn ]
                  Extracts entities, analyzes metrics, logs
```

### 3.1 `before_chat`
- **Trigger:** Dispatched when the user clicks send or triggers message regeneration.
- **Use Cases:** Moderation filters, document attachment enrichment, dynamic parameter adjustments (temperature, sampling).
- **Payload Schema:**
  ```typescript
  {
    userMessage: string,
    messages: ChatMessage[],
    params: InferenceParams,
    sessionId: string | null,
    sessionTitle: string | null
  }
  ```

### 3.2 `on_system_prompt`
- **Trigger:** Dispatched during final system prompt assembly.
- **Use Cases:** Injecting memory directives, security rules, custom personas, or timezone metadata.
- **Return Value:** Modified or appended system prompt string.

### 3.3 `on_chunk`
- **Trigger:** Invoked for every token fragment received during real-time streaming.
- **Use Cases:** Per-token latency measurements, character counting, early-stop heuristic triggers.

### 3.4 `after_chat_turn`
- **Trigger:** Executed immediately once the assistant finishes generating (`is_done = true`).
- **Use Cases:**
  - Parsing special tags like `<memorize>` or `<skill>` to persist into memory graphs.
  - Recording local analytics or generating conversation summaries.
- **Payload Schema:**
  ```typescript
  {
    userMessage?: string,
    assistantResponse: string,
    sessionId?: string | null,
    sessionTitle?: string | null,
    elapsedMs?: number,
    metrics?: GenerationMetrics
  }
  ```

---

## 4. UI Extensions

Plugins can contribute visual components directly into Atena Studio:

### 4.1 Sidebar Views
Declared in `views` with `"location": "sidebar"`.
When enabled, the view icon appears in the sidebar navigation. If the plugin is disabled, the view unmounts without requiring an app restart.

### 4.2 Integrated Plugin Manager
Users manage plugins from two locations:
1. **Sidebar Shortcut:** Blocks icon (`Blocks`) adjacent to notifications.
2. **Dedicated Settings Panel:** "Plugins & Extensions" view inside **Settings**, with complete version badges, descriptions, toggle controls, and a button to open the plugin folder in the OS file manager.

---

## 5. Practical Tutorial: Building Your First Plugin in 5 Minutes

Here is an example plugin named **`atena-prompt-guard`** that inspects prompt inputs.

### Step 1: Create plugin directory
In terminal:
```bash
mkdir -p ~/.atena/plugins/atena-prompt-guard
```

### Step 2: Create `plugin.json`
Inside that folder, save `plugin.json`:
```json
{
  "id": "atena-prompt-guard",
  "name": "Prompt Guard & Auditor",
  "version": "1.0.0",
  "description": "Inspects incoming prompts and adds best-practice reminders in local chats.",
  "author": "Carlos",
  "icon": "Sparkles",
  "enabled": true,
  "category": "chat_enhancer",
  "hooks": [
    "on_system_prompt",
    "after_chat_turn"
  ],
  "settings": {
    "log_turns": true
  }
}
```

### Step 3: Reload in Atena Studio
1. Open Atena Studio.
2. Go to **Settings → Plugins & Extensions** (or click the Blocks icon in the sidebar).
3. Click **Reload**.
4. The plugin appears in the list with "Community" status, ready to be toggled on or off!

---

## 6. Native Rust Binaries (`native_entry`) and TS/Vue Frontend (`entry`)

Atena Studio supports a hybrid architecture:

### 6.1 TypeScript / Vue Frontend Extensions
- Declared as `"entry": "dist/index.js"` pointing to a compiled frontend bundle.
- Loaded by Atena using `read_plugin_script(pluginId)` to mount reactive components and interact with UI events.

### 6.2 Compiled Native Rust Binaries
For computation-heavy tasks, hardware drivers, or intense numerical processing, plugins can provide compiled Rust binaries:
- Declared via `"native_entry": "bin/runner"` (the `.exe` extension is handled automatically on Windows).
- **Crash Isolation (Process Boundary):** The binary executes as an independent subprocess managed via `stdin`/`stdout` JSON IPC. A panic in the plugin binary will not crash Atena Studio.
- **Communication Flow:**
  1. Atena writes to `stdin`:
     ```json
     { "command": "execute_task", "payload": { "data": [1, 2, 3] } }
     ```
  2. The binary responds on `stdout`:
     ```json
     { "status": "ok", "result": 42 }
     ```

---

## 7. Cloud Providers and `cloud_provider` Category

AI models in Atena Studio are organized into **Local Models** (MLX, GGUF, Ollama) and **Cloud Models**:
1. **"Cloud" Tab in Models View:** Consolidates all remote and cloud endpoints (Google Antigravity, OpenAI, OpenRouter, Ollama, and Custom OpenAI).
2. **Unified Settings ("Cloud Providers"):**
   - Rather than creating fragmented sidebar menus, all plugins with the `cloud_provider` category have their settings unified in one panel.
   - For architecture details, see [`09_cloud_providers.md`](./09_cloud_providers.md).
3. **Canonical Cloud Connector (`atena-plugin-cloud`):**
   - Reference plugin for the community to extend cloud APIs cleanly.
   - The initial onboarding wizard focuses 100% on local AI, keeping cloud setup optional.

---

## 8. Native Backend IPC Commands (Tauri)

For programmatic interactions with the Atena backend, the following Tauri commands are available:

- `list_plugins()`: Returns consolidated installed plugins (official and external).
- `toggle_plugin(id: string, enabled: boolean)`: Toggles active state and persists to `~/.atena/plugins.json`.
- `get_plugins_folder()`: Returns the absolute path to the plugins directory.
- `open_plugins_folder()`: Opens the plugins folder in Finder (macOS), File Explorer (Windows), or default file manager (Linux).
- `save_plugin_settings(pluginId: string, settings: Value)`: Persists plugin settings values.
- `run_plugin_native(pluginId: string, command: string, payload: Value)`: Spawns the compiled Rust binary subprocess with JSON IPC.
- `read_plugin_script(pluginId: string)`: Retrieves the frontend bundle script for dynamic webview injection.

---

## 9. Build, Packaging, and Distribution Cycle for Third-Party Plugins

Atena Studio plugins adhere to a **Zero-Friction end-user policy**: users installing plugins **do not need Node.js, pnpm, or Rust installed**.

Building and bundling is handled exclusively by the plugin author:

```text
       PLUGIN DEVELOPER                            ATENA STUDIO USER
  ┌───────────────────────────────┐              ┌───────────────────────────────┐
  │ 1. TS/Vue or Rust code        │              │ 1. Download or clone folder   │
  │ 2. Run release build          │ ──────────►  │    into ~/.atena/plugins/     │
  │    (Vite / Cargo build)       │ (Release     │ 2. Launch Atena Studio        │
  │ 3. Publish ZIP archive        │  on GitHub)  │ 3. Plugin is active instantly!│
  └───────────────────────────────┘              └───────────────────────────────┘
```

### 9.1 Scenario A: Declarative Plugins
Plugins that only configure prompts, lifecycle hooks, and `settings_schema`:
- **Build:** No compilation required.
- **Distribution:** Only `plugin.json`, `README.md`, and optional `locales/` directory.

### 9.2 Scenario B: Plugins with Custom Frontend (`entry: "dist/index.js"`)
For plugins with custom Vue components:
1. The author develops with Vue 3 and Vite in their own repository.
2. Build in library mode:
   ```bash
   pnpm run build
   ```
   This generates `dist/index.js`.
3. Distribute the bundle including the `dist/` directory.
4. Atena Studio reads the script via `read_plugin_script` and evaluates it in the webview.

### 9.3 Scenario C: Plugins with Native Rust Binaries (`native_entry: "bin/runner"`)
For plugins with intensive compute or native I/O:
1. Build release binaries:
   ```bash
   cargo build --release
   ```
2. Copy the artifact from `target/release/` to `bin/` inside the plugin package:
   - macOS / Linux: `bin/runner`
   - Windows: `bin/runner.exe`
3. Declare `"native_entry": "bin/runner"`.
4. Atena Studio executes it via isolated JSON IPC subprocesses.

### 9.4 Cross-Platform GitHub Actions Automation (CI/CD)
Best practice for native plugins is providing automated GitHub Release ZIPs per target architecture:
- `atena-plugin-xyz-macos-arm64.zip` (Apple Silicon M1/M2/M3/M4)
- `atena-plugin-xyz-macos-x64.zip` (Intel Mac)
- `atena-plugin-xyz-windows-x64.zip` (Windows)
- `atena-plugin-xyz-linux-x64.zip` (Linux)

The user unzips the folder into `~/.atena/plugins/<plugin-id>/` and clicks **Reload** in **Settings → Plugins & Extensions**.
