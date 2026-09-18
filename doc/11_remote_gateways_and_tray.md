# 📡 11. Remote Messaging Gateways & Background System Tray

Atena Studio supports native **background execution (daemon mode)**, cross-platform **system tray / menu bar integration**, and bidirectional **remote messaging gateways** (Telegram Bot, Discord Bot).

This architecture frees the user from being required to keep the main application window open and active on desktop. Conversations can occur remotely from a smartphone, tablet, or secondary computer while leveraging the local workstation's hardware, active AI models, associative cognitive memory, and SQLite session persistence.

---

## 1. System Tray & Background Execution (`src-tauri/src/tray.rs`)

Atena Studio implements native system tray support across **macOS (Status Bar)**, **Windows (Notification Area / System Tray)**, and **Linux (AppIndicator / StatusNotifier)** using Tauri v2's native `TrayIconBuilder`.

### Lifecycle and Behavior
1. **Close to Tray (`close_to_tray`):**
   * When enabled in Settings, clicking the window close button (`X`) intercepts `tauri::WindowEvent::CloseRequested` via `api.prevent_close()` and calls `window.hide()`.
   * The application process remains running silently in the background.
2. **Left-Click Toggle:**
   * Clicking the tray icon automatically toggles window visibility (`show()` + `set_focus()` if hidden, `hide()` if visible).
3. **macOS Adaptive Template Icon:**
   * On macOS, the menu bar uses a dedicated monochrome silhouette icon (`icons/tray-icon.png`) with `.icon_as_template(true)`.
   * The status bar icon automatically and dynamically adapts to macOS system themes: rendering in crisp white on dark menu bars and black on light menu bars, complying with Apple Human Interface Guidelines.
4. **Context Menu:**
   * **Status Header:** Displays current connection status (`● Atena Studio`).
   * **Show Atena Studio:** Restores, unminimizes, and focuses the main desktop window.
   * **Hide to Tray:** Hides the main window to background.
   * **Quit Atena Studio:** Triggers an explicit graceful termination, saving window state, auto-persisting the associative memory graph (`auto_persist_default()`), stopping background sidecars, and aborting active gateway listeners.

---

## 2. Remote Messaging Gateways (`src-tauri/src/services/gateways/`)

Gateways serve as bidirectional adapters connecting external chat platforms to Atena's internal inference and cognitive memory pipeline.

```text
src-tauri/src/services/gateways/
├── mod.rs         # GatewaysManager, GatewayStatus, lifecycle hooks, and token test endpoints
├── telegram.rs    # Telegram Bot Long-Polling runner, command parser, typing indicators, and message chunker
└── discord.rs     # Discord Bot REST/Gateway runner, typing triggers, and 2000-character chunker
```

### Shared Gateway Architecture
* **`GatewaysManager`:**
  * Coordinates background Tokio tasks for each gateway.
  * Dynamically starts, stops, or reloads gateways when `save_app_config` is called without requiring application restarts.
  * Exposes `get_status_reports()` reporting `connected`, `connecting`, `offline`, or `error` state.
* **Unified Cognitive Pipeline:**
  * When a remote message is received, it routes directly into `execute_stream_chat_internal(&state, req, on_chunk)`.
  * The active local model (MLX, LLaMA, Ollama) or connected Cloud provider processes the prompt.
  * Long-term facts, procedural skills, and episodic memory are queried and updated.
  * Internal reasoning tags (`<think>`) and autonomous memory tags (`<memory ... />`) are stripped from user-facing chat output before sending.
* **SQLite Persistence Parity:**
  * Conversations are recorded directly into `~/.atena/atena.db` under dedicated sessions (e.g. `telegram_<chat_id>` or `discord_<channel_id>`).
  * Remote chats immediately appear in the Atena Studio desktop recents sidebar, preserving chat history and continuity across devices.

---

## 3. Telegram Bot Gateway (`telegram.rs`)

Implemented using Tokio asynchronous HTTP long-polling against the official Telegram Bot API with zero heavy third-party framework overhead:

* **Long-Polling Loop:** Queries `/getUpdates?offset={offset}&timeout=25`.
* **Typing Indicator:** Sends `/sendChatAction` with action `typing` every 4 seconds while inference is in progress.
* **Message Chunking:** Telegram enforces a 4096-character limit per message. Responses exceeding 3800 characters are automatically split at newline or word boundaries.
* **Commands Supported:**
  * `/start` or `/help`: Overview of available commands and bot information.
  * `/new`: Clears active session context and starts a fresh conversation.
  * `/model`: Displays the currently loaded AI model and backend engine.
  * `/memory <query>`: Queries the local associative memory graph for facts and concepts.
  * `/id`: Returns the user's Telegram numeric User ID.

---

## 4. Discord Bot Gateway (`discord.rs`)

Connects Atena to private Discord channels or direct messages:

* **Authentication:** Uses standard `Bot <token>` authorization against Discord API v10 (`https://discord.com/api/v10`).
* **Typing Indicator:** Triggers `/channels/{id}/typing` periodically during model generation.
* **Message Chunking:** Discord enforces a strict 2000-character message limit. Responses exceeding 1900 characters are safely split and delivered in sequential messages.
* **Commands Supported:**
  * `!atena help`: Displays commands and instructions.
  * `!atena new`: Resets conversation context for the channel.
  * `!atena model`: Displays current model status.
  * `!atena id`: Displays user's Discord ID.

---

## 5. Security & Whitelist Guardrail

Because Atena Studio integrates local file tools, shell execution, MCP servers, and private personal memories, remote gateways enforce a strict **User ID Whitelist (`allowed_user_ids`)**:

1. **Authorization Check:**
   * Every incoming update checks the sender's ID against the configured whitelist.
   * If a user is not in the whitelist, the bot responds with an unauthorized notification stating their ID and instructing them to add it to Atena Studio Settings.
2. **Channel Restrictions (Discord):**
   * Configurable `allowed_channel_ids` limits bot listening to specific designated channels within a Discord guild.

---

## 6. Configuration Schema (`config.json`)

Settings are stored in `~/.atena/config.json`:

```json
{
  "run_in_background": true,
  "close_to_tray": true,
  "gateways": {
    "telegram": {
      "enabled": true,
      "bot_token": "123456789:ABCdefGHIjklMNOpqrSTUvwxYZ...",
      "allowed_user_ids": [123456789],
      "enable_memory": true,
      "enable_tools": false,
      "custom_system_prompt": null
    },
    "discord": {
      "enabled": false,
      "bot_token": "",
      "allowed_user_ids": [],
      "allowed_channel_ids": [],
      "enable_memory": true,
      "enable_tools": false,
      "custom_system_prompt": null
    }
  }
}
```

---

## 7. Step-by-Step Setup Guides (How to Obtain Tokens)

### A. How to Create a Telegram Bot Token & Find Your User ID

Creating a private Telegram bot takes less than 2 minutes:

1. **Open Telegram & Search for BotFather:**
   * In Telegram search, type `@BotFather` (look for the official blue verification badge).
   * Or click: [https://t.me/BotFather](https://t.me/BotFather).
2. **Start the Bot Creation Flow:**
   * Send the command `/start` followed by `/newbot`.
   * **Choose a display name:** e.g., `Atena Assistant` or `My Private AI`.
   * **Choose a unique username:** must end in `bot` (e.g., `my_atena_personal_bot`).
3. **Copy the HTTP API Token:**
   * BotFather will reply with a congratulatory message containing your token:
     ```text
     Use this token to access the HTTP API:
     7891234567:AAFl0xP...-oXyZ123456
     ```
   * Paste this token into **Atena Studio → Settings → Gateways → Telegram → Bot API Token**.
4. **Obtain Your Telegram Numeric User ID (Whitelist Security):**
   * Start a chat with your new bot and send `/start`.
   * If your whitelist is empty, the bot will automatically reply with:
     ```text
     ⚠️ Access Restricted
     Your Telegram User ID is: 123456789.
     ```
   * Alternatively, search for the official utility `@userinfobot` on Telegram and send `/start` to see your numeric `Id`.
   * Paste your numeric ID into the **Authorized Telegram User IDs (Whitelist)** field in Atena Studio and click **Save Changes**.

---

### B. How to Create a Discord Bot Token & Invite it to Your Server

1. **Access the Discord Developer Portal:**
   * Navigate to: [https://discord.com/developers/applications](https://discord.com/developers/applications) and sign in with your Discord account.
2. **Create an Application:**
   * Click the **"New Application"** button in the top right.
   * Enter a name (e.g., `Atena Studio`) and click **Create**.
3. **Configure the Bot & Obtain the Token:**
   * On the left sidebar menu, click **"Bot"**.
   * Under the **"Token"** section, click **"Reset Token"** (enter your 2FA code if prompted).
   * Click **"Copy"** and paste the token into **Atena Studio → Settings → Gateways → Discord → Bot Token**.
4. **Enable Privileged Gateway Intents (MANDATORY):**
   * On the same **"Bot"** page, scroll down to the **"Privileged Gateway Intents"** section.
   * Enable the toggle for:
     * **Message Content Intent** (Required for Atena to read and answer messages).
   * Click **"Save Changes"** at the bottom of the Discord portal.
5. **Invite the Bot to Your Discord Server:**
   * On the left sidebar menu, go to **"OAuth2" → "URL Generator"**.
   * Under **"Scopes"**, check `bot`.
   * Under **"Bot Permissions"**, check:
     * `View Channels`
     * `Send Messages`
     * `Read Message History`
   * Copy the generated URL at the bottom, paste it into your web browser, and choose the server you want to add Atena to.
6. **Obtain Channel ID and User ID (Whitelist):**
   * In Discord settings: go to **User Settings → Advanced → Enable Developer Mode**.
   * Right-click on your user avatar → **Copy User ID**.
   * Right-click on the specific channel you want Atena to listen to → **Copy Channel ID**.
   * Paste these into Atena Studio's Discord settings and click **Save Changes**.

