# Plugin Development Guide — Atena Studio

Atena Studio provides plugin support to allow the community to customize the **chat loop**, add **custom views**, render specialized components, and configure new modules in a lightweight, modular, and secure manner.

In Atena Studio, core features such as **Procedural Associative Memory** (neural graph, continuous learning, logbook) and **Cloud Providers** (Antigravity, OpenAI, OpenRouter, Ollama) are **100% native to the core**. The plugin ecosystem is designed for community extensions, enterprise tools, and custom workflows installed in `~/.atena/plugins/`.

---

## Plugin Categories and UI Guidelines

To maintain a clean, organized, and focused user experience, each plugin must declare its `category` in the manifest:

| Category | Purpose | Interface Behavior |
|---|---|---|
| `chat_enhancer` | Prompt modifiers, safety filters, and output formatters. | Operates silently via chat loop hooks. |
| `ui_extension` | Dedicated views or new navigation flows. | Registers sidebar tabs (`views`) when enabled. |
| `tool_provider` | Function calling tools and local integrations. | Registered in the assistant's tool selector. |
| `cloud_provider` | AI model providers and cloud services. | **Unified:** Grouped automatically under the "Cloud Providers" section in Settings, without cluttering sidebar navigation. |
| `core_extension` | Heavy core application extensions (e.g., neural graph). | Can inject dedicated views and Settings controls. |
| `custom` | Community experimental extensions. | As declared in `views` and `settings_schema`. |

---

## Execution Types: Frontend (TS/Vue) and Compiled Native (Rust)

Atena supports plugins in two composable formats:

1. **Frontend (TypeScript / Vue)**: Declaring `"entry": "index.js"` or `"index.ts"`. The script can inject UI components, listen to reactive chat events, and register custom views.
2. **Compiled Native (Rust)**: For heavy I/O tasks, numerical processing, hardware drivers, or advanced network connectors, declare `"native_entry": "bin/my_plugin"` (or `"bin/my_plugin.exe"` on Windows).
   - Atena manages the subprocess safely via Stdio IPC (`stdin` / `stdout`).
   - An eventual crash or panic in a native plugin does not compromise the stability of Atena Studio.

---

## Recommended Structure for Standalone Repositories

When creating a plugin to publish in an independent git repository (e.g., `https://github.com/example/atena-plugin-custom`), organize the folder as follows:

```
my-plugin/
├── plugin.json         # Manifest with metadata, schema, hooks, and views
├── README.md           # Quickstart installation guide and overview
├── doc/                # Detailed technical documentation
│   ├── ARCHITECTURE.md
│   └── USAGE.md
├── locales/            # i18n translation catalogs
│   ├── pt-BR.json
│   ├── en.json
│   └── es.json
├── src/                # Frontend code (TypeScript / Vue)
└── native/             # Native Rust code (optional, Stdio IPC)
    ├── Cargo.toml
    └── src/main.rs
```

---

## How to Create a Plugin

1. Create a directory inside `~/.atena/plugins/<your-plugin>/` (or `%USERPROFILE%\.atena\plugins\<your-plugin>\` on Windows). For plugins distributed alongside the Atena source tree, use `./plugins/<your-plugin>/`.
2. Add a `plugin.json` manifest:

```json
{
  "id": "my-custom-plugin",
  "name": "My Custom Plugin",
  "version": "1.0.0",
  "description": "Adds new capabilities to Atena's chat and interface.",
  "author": "Ana",
  "license": "MIT",
  "repository": "https://github.com/example/atena-plugin-custom",
  "readme": "README.md",
  "doc": "doc/ARCHITECTURE.md",
  "locales": "locales",
  "icon": "Blocks",
  "enabled": true,
  "category": "chat_enhancer",
  "entry": "dist/index.js",
  "native_entry": "bin/runner",
  "hooks": [
    "before_chat",
    "on_system_prompt",
    "on_chunk",
    "after_chat_turn"
  ],
  "views": [
    {
      "id": "my-view",
      "title": "My View",
      "icon": "Blocks",
      "location": "sidebar"
    }
  ],
  "settings_schema": [
    {
      "id": "enable_feature",
      "label": "Enable Advanced Feature",
      "description": "Activates custom processing before each response.",
      "type": "boolean",
      "default": true
    },
    {
      "id": "api_key",
      "label": "Service API Key",
      "description": "Authentication token for external API calls.",
      "type": "password",
      "placeholder": "sk-..."
    },
    {
      "id": "max_retries",
      "label": "Maximum Retries",
      "description": "Number of automatic retries upon connection errors.",
      "type": "number",
      "default": 3,
      "min": 1,
      "max": 10,
      "step": 1
    },
    {
      "id": "style_mode",
      "label": "Response Style",
      "type": "select",
      "default": "balanced",
      "options": [
        { "label": "Concise", "value": "concise" },
        { "label": "Balanced", "value": "balanced" },
        { "label": "Detailed", "value": "detailed" }
      ]
    }
  ]
}
```

3. Open Atena Studio, navigate to **Settings → Plugins & Extensions**, and click **Reload**.
4. If the plugin declares a `settings_schema`, a dedicated settings entry with its name and icon will automatically appear in **Settings**, generating the corresponding form inputs. When the plugin is disabled, the menu item and its inputs are removed immediately.

---

## Native Communication with Rust (`native_entry`)

The binary compiled in Rust receives a JSON message on `stdin` in the following format:

```json
{
  "command": "process_tokens",
  "payload": { "text": "example" }
}
```

And responds on `stdout` with JSON:

```json
{
  "status": "success",
  "data": { "result": 42 }
}
```

Using the `usePlugins` composable, the frontend executes native commands as follows:

```typescript
const { invokePluginNative } = usePlugins()
const result = await invokePluginNative('my-custom-plugin', 'process_tokens', { text: 'example' })
```

---

## 🛠️ Compiling and Distributing Your Plugin (Build & Release)

End users of Atena **do not need Node, pnpm, or Rust installed**. All compilation is performed by the plugin author prior to distribution:

1. **If the plugin has a Vue/TypeScript Frontend (`entry`):**
   - The author bundles the code using a bundler (Vite, Rollup, or esbuild) generating `dist/index.js`.
   - The `dist/` directory is packaged into the plugin release archive.
2. **If the plugin includes native Rust code (`native_entry`):**
   - The author compiles in release mode: `cargo build --release`.
   - The binary is placed inside the plugin's `bin/` directory (e.g., `bin/runner` or `bin/runner.exe`).
   - Automation workflows such as GitHub Actions can generate pre-compiled release archives for macOS, Windows, and Linux.
3. **Installation by the user:**
   - The user downloads the ZIP archive corresponding to their platform and extracts it into `~/.atena/plugins/<plugin-name>/`.
   - In Atena Studio, they click **Reload** in **Settings → Plugins & Extensions**.

---

## Full Architecture & Community Documentation

- **Official Repository**: [https://github.com/voylab-app/atena-studio](https://github.com/voylab-app/atena-studio)
- **Plugin Guide**: [https://github.com/voylab-app/atena-studio/blob/main/PLUGINS.md](https://github.com/voylab-app/atena-studio/blob/main/PLUGINS.md)
- **Internal System Architecture**: [doc/08_plugin_system.md](doc/08_plugin_system.md)
