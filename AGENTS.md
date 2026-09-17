# Development and Privacy Guidelines — Atena Studio

## 🌍 Multi-User Architecture and Neutrality
1. **Public and Distributed Product:** Atena Studio is an application intended for any end user. Source code, prompts, documentation, examples, and unit tests **MUST NOT** contain real names, nicknames, personal data, preferences, or references to specific developers (the project must be 100% impersonal and neutral).
2. **Fictitious and Neutral Names:** When it is necessary to include examples in prompts, tests, placeholders, or documentation, always use neutral fictitious names such as `Maria`, `Carlos`, `Ana`.
3. **Dynamic User Identity:** The AI has no preconceived knowledge of who is operating it. All identity must be learned dynamically during interaction through the `User` subject node.
4. **No Hardcoded Personal Paths:** Never use absolute paths containing system usernames (e.g., fixed system paths). Always use dynamic environment variable resolution (`$HOME`, `$USERPROFILE`) or relative directories.

## 📚 Mandatory Consultation and Use of Documentation (`doc/`)
1. **Consult Documentation Before Implementing:** Always check and consult the relevant technical documents in the `doc/` directory before creating, modifying, or refactoring components, business logic, or integrations.
2. **Adhere to Established Standards:** The `doc/` directory describes the official architecture of Atena Studio:
   - `doc/01_associative_memory.md` (Associative memory, graphs, and episodes)
   - `doc/02_neural_visualizer.md` (Neural visualizer and 3D shaders)
   - `doc/03_chat_and_inference.md` (Chat, streaming, history, and sessions)
   - `doc/04_model_manager.md` (GGUF and MLX model manager)
   - `doc/05_mcp_tools.md` (MCP tools, authorization, and function calling)
   - `doc/06_tauri_backend_architecture.md` (Rust/Tauri backend and IPC)
   - `doc/07_distribution_and_releases.md` (Packaging and distribution)
   - `doc/08_plugin_system.md` (Plugin system and lifecycle)
   - `doc/09_cloud_providers.md` (Cloud AI providers)
   - `doc/10_internationalization_and_locales.md` (Internationalization guidelines)
3. **Keep Documentation Synchronized:** Any architectural change or significant new capability must be properly reflected in the corresponding documentation in `doc/`.

## 🌐 Strict Internationalization (i18n)
1. **No Hardcoded Strings:** No user-visible string (titles, buttons, labels, placeholders, tooltips, toasts, error messages, or dialogs) may be placed directly in Vue or TypeScript code.
2. **Mandatory Language Parity:** Any new string MUST be added simultaneously with the exact same key name across all supported catalogs:
   - `frontend/app/locales/pt-BR.json`
   - `frontend/app/locales/en.json`
   - `frontend/app/locales/es.json`
   - `frontend/app/locales/zh-CN.json`
   - `frontend/app/locales/ru.json`
3. **Consistent Interpolation:** Variables such as `{name}`, `{count}`, `{tokens}`, etc., must preserve the exact same naming across all translations.
4. **Architectural Reference:** For any questions regarding translation and pluralization patterns, consult `doc/10_internationalization_and_locales.md`.

## 💻 Codebase Language and Documentation (English Standard)
1. **English as Standard and Default Language:** All source code, code comments, docstrings, variable/function/class names, error logs, technical documentation (`doc/`), guides, specifications, and root documentation files (`README.md`, `PLUGINS.md`, etc.) **MUST** be written strictly in English.
2. **Default Project Language:** The default language of the project is English. Non-English languages are only permitted inside localized translation catalogs (`frontend/app/locales/*.json`) or runtime multi-language test fixtures.
3. **No Non-English Comments or Docs:** Never write code comments or documentation in Portuguese, Spanish, or any language other than English (`//`, `/* */`, `<!-- -->`, `///`, `#`, `.md`).
4. **Commit Messages & PRs:** All commit messages, branch names, pull request descriptions, and issue discussions must follow English conventions.

