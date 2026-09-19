# 🌐 10. Internationalization (i18n) and Locales Architecture

**Atena Studio** is designed for users worldwide. The ecosystem features a comprehensive, reactive, and extensible internationalization architecture spanning from the Nuxt/Vue frontend to native invocations and system prompts in the Rust backend.

---

## 1. Directory Structure and Supported Locales

Translation catalogs reside in `frontend/app/locales/`:

```text
frontend/app/locales/
├── pt-BR.json    # Portuguese (Brazil) - Reference catalog
├── en.json       # English (US)
├── es.json       # Spanish (Español)
├── zh-CN.json    # Simplified Chinese (简体中文)
└── ru.json       # Russian (Русский)
```

Each JSON file shares an identical key structure organized by functional domain:
* `common`: Universal actions (Save, Cancel, Delete, Confirm, Close).
* `sidebar`: Primary navigation items, VRAM/RAM memory counters, and notification panel.
* `header`: Active model status, privacy indicator, and theme switcher.
* `chat`: Messaging interface, starter prompts, reasoning effort controls, tools, and warnings.
* `params`: Inference parameters (temperature, context window, repetition penalty, system prompt).
* `cloud`: Cloud provider configurations (Antigravity, OpenAI, OpenRouter, Ollama, custom endpoints).
* `settings`: Global settings panels, directories, runtime switches, and plugin management.

---

## 2. Reactive Composable: `useLocale`

Frontend locale management is centralized in `frontend/app/composables/useLocale.ts`:

```typescript
export interface LocaleOption {
  code: 'en' | 'pt-BR' | 'es' | 'zh-CN' | 'ru'
  name: string
  label: string
  flag: string
  descriptionKey: string
}

export const SUPPORTED_LOCALES: LocaleOption[] = [
  { code: 'en', name: 'English', label: 'EN', flag: '🇺🇸', descriptionKey: 'settings.lang_en_desc' },
  { code: 'pt-BR', name: 'Português (Brasil)', label: 'PT', flag: '🇧🇷', descriptionKey: 'settings.lang_pt_desc' },
  { code: 'es', name: 'Español', label: 'ES', flag: '🇪🇸', descriptionKey: 'settings.lang_es_desc' },
  { code: 'zh-CN', name: '简体中文', label: 'ZH', flag: '🇨🇳', descriptionKey: 'settings.lang_zh_desc' },
  { code: 'ru', name: 'Русский', label: 'RU', flag: '🇷🇺', descriptionKey: 'settings.lang_ru_desc' }
]
```

### Key Capabilities
1. **Automatic Persistence**: Selected locale is stored in browser `localStorage` under the `atena_locale` key.
2. **Instant Reactivity**: Changing language updates views, drawers, navigation, and tooltips without requiring an application reload.
3. **Smart Browser Detection**: On fresh installs without prior configuration, the app detects the operating system language via `navigator.language`.

---

## 3. Scalable Language Selector (`<select>`)

In **Settings > General**, the language switcher is implemented as a styled `<select>` element rather than fixed horizontal buttons:

### Advantages of the `<select>` Component:
* **Unlimited Scalability**: New locales (such as French, German, Japanese, Arabic) can be added to `supportedLocales` without crowding or breaking UI layouts.
* **Accessibility and Usability**: Built-in keyboard navigation, screen reader support, and consistent behavior across compact displays.
* **Refined Aesthetics**: Custom CSS styling with translucent borders, dark backdrop, indicator chevron, and focus rings.

---

## 4. Prompt Localization in Rust Backend

A distinctive feature of Atena Studio is natural-language-driven cognitive capabilities in the Rust/Tauri backend, such as MCP tool humanization (`translate_mcp_tool_labels`).

### Multilingual Support via `target_locale`
To ensure parity across languages, the command accepts a `target_locale: Option<String>` parameter:

```rust
#[command]
async fn translate_mcp_tool_labels(
    tools: Vec<McpToolDefinition>,
    mlx_host: String,
    mlx_port: u16,
    ollama_host: String,
    ollama_port: u16,
    target_locale: Option<String>,
    state: State<'_, AppState>,
) -> Result<McpTranslationResponse, String>
```

1. Frontend passes `targetLocale: currentLocale.value`.
2. Backend selects the matching system prompt and tone:
   * **`en`**: Title Case guidelines and standard technical interface conventions in English.
   * **`es`**: Spanish system prompt and localized dictionary fallbacks.
   * **`zh-CN` / `zh`**: Simplified Chinese prompt guidelines and localized technical term dictionaries.
   * **`ru`**: Russian system prompt and Cyrillic terminology dictionaries.
   * **`pt-BR` / Default**: Portuguese prompt refinements and localized terms.
3. If the model times out or is unreachable, the heuristic fallback parser uses dictionaries tailored to the requested locale.

---

## 5. Adding a New Language

To add support for a new language (e.g., French — `fr`):

1. **Create the translation JSON file**:
   * Duplicate `frontend/app/locales/en.json` to `frontend/app/locales/fr.json`.
   * Translate values while preserving variables such as `{name}`, `{count}`, `{tokens}`.

2. **Register in the Vue i18n plugin (`frontend/app/plugins/i18n.ts`)**:
   ```typescript
   import fr from '~/locales/fr.json'
   
   // Add 'fr' to messages:
   messages: { 'pt-BR': ptBR, en, es, fr }
   ```

3. **Register in `useLocale.ts`**:
   ```typescript
   export const supportedLocales: SupportedLocale[] = [
     // ...
     { code: 'fr', name: 'Français', flag: '🇫🇷' }
   ]
   ```

4. **(Optional) Add backend prompt support (`src-tauri/src/lib.rs`)**:
   * In the `match locale` block of `translate_mcp_tool_labels`, add a `"fr"` case for French humanization.

5. **(Optional) Add native System Tray labels (`src-tauri/src/tray.rs`)**:
   * In `get_tray_labels(locale: &str)`, add a match arm with translated labels (`title`, `show`, `hide`, `quit`, `tooltip`).

---

## 6. Native System Tray Menu Localization

The native desktop menu bar / system tray (`src-tauri/src/tray.rs`) dynamically synchronizes with the user's active locale:
* **Bootstrap Localization**: On launch, `setup_tray` initializes with the language saved in `AppConfig::load().language`.
* **Dynamic Runtime Updates**: When the user changes language in the frontend (`useLocale.ts` or Settings), the IPC command `update_tray_locale(locale)` rebuilds the tray menu and tooltip immediately without restarting the application.
* **Supported Languages**: Built-in support across English (`en`), Portuguese (`pt-BR`), Spanish (`es`), Simplified Chinese (`zh-CN`), and Russian (`ru`).

---

## 7. Neutrality and Privacy Guidelines

In strict accordance with [`AGENTS.md`](../AGENTS.md):
* Localization files must never contain personal names, developer references, or fixed personal metadata.
* Example texts and documentation samples must always employ neutral, fictional names (such as `Maria`, `Carlos`, `Ana`).
