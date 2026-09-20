// services/plugins.rs — Atena Studio Plugin & Extension Manager
// Enables developers to extend Atena with chat hooks, new views, and local integrations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use crate::core::process::silent_command;

/// Plugin type or category
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginCategory {
    CoreExtension,
    UiExtension,
    ChatEnhancer,
    ToolProvider,
    CloudProvider,
    Custom,
}

/// Selection option for 'select' field type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSettingOption {
    pub label: String,
    pub value: serde_json::Value,
}

/// Definition of a dynamic configuration field exposed by the plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSettingField {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub field_type: String, // "boolean" | "text" | "number" | "select" | "password"
    #[serde(default)]
    pub default: Option<serde_json::Value>,
    #[serde(default)]
    pub options: Option<Vec<PluginSettingOption>>,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub step: Option<f64>,
    #[serde(default)]
    pub placeholder: Option<String>,
}

/// Interface view exposed by the plugin in UI (Sidebar, Drawer, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginViewDefinition {
    pub id: String,
    pub title: String,
    pub icon: Option<String>,
    pub location: Option<String>, // "sidebar", "chat_action", "settings"
    pub description: Option<String>,
}

/// Plugin manifest (`plugin.json`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_category")]
    pub category: PluginCategory,
    #[serde(default)]
    pub hooks: Vec<String>, // "before_chat", "on_system_prompt", "on_chunk", "after_chat_turn"
    #[serde(default)]
    pub views: Vec<PluginViewDefinition>,
    #[serde(default)]
    pub is_builtin: bool,
    #[serde(default)]
    pub entry: Option<String>, // Frontend script (TS/JS, e.g.: "dist/index.js")
    #[serde(default)]
    pub native_entry: Option<String>, // Native compiled Rust binary (e.g.: "bin/engine")
    #[serde(default)]
    pub readme: Option<String>,
    #[serde(default)]
    pub doc: Option<String>,
    #[serde(default)]
    pub locales: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub settings: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub settings_schema: Vec<PluginSettingField>,
}

fn default_true() -> bool {
    true
}

fn default_category() -> PluginCategory {
    PluginCategory::Custom
}

/// Consolidated plugin information displayed on the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub icon: Option<String>,
    pub enabled: bool,
    pub is_builtin: bool,
    pub category: PluginCategory,
    pub hooks: Vec<String>,
    pub views: Vec<PluginViewDefinition>,
    pub directory: Option<String>,
    pub entry: Option<String>,
    pub native_entry: Option<String>,
    pub readme: Option<String>,
    pub doc: Option<String>,
    pub locales: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
    pub has_settings: bool,
    pub settings_schema: Vec<PluginSettingField>,
    pub settings_values: HashMap<String, serde_json::Value>,
}

/// Persisted state of user plugin preferences
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginsConfig {
    pub enabled_plugins: HashMap<String, bool>,
    #[serde(default)]
    pub plugin_settings: HashMap<String, HashMap<String, serde_json::Value>>,
}

pub struct PluginManager;

impl PluginManager {
    /// Returns base plugins directory at `~/.atena/plugins` dynamically and neutrally
    pub fn get_plugins_directory() -> PathBuf {
        let base_dir = if cfg!(target_os = "windows") {
            let userprofile = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".to_string());
            PathBuf::from(&userprofile).join(".atena").join("plugins")
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(&home).join(".atena").join("plugins")
        };

        if !base_dir.exists() {
            let _ = fs::create_dir_all(&base_dir);
        }

        base_dir
    }

    /// Returns config file path `~/.atena/plugins.json`
    fn get_config_filepath() -> PathBuf {
        let base_dir = if cfg!(target_os = "windows") {
            let userprofile = std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\Users\\Default".to_string());
            PathBuf::from(&userprofile).join(".atena")
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(&home).join(".atena")
        };

        if !base_dir.exists() {
            let _ = fs::create_dir_all(&base_dir);
        }

        base_dir.join("plugins.json")
    }

    /// Loads plugin activation preferences
    pub fn load_config() -> PluginsConfig {
        let path = Self::get_config_filepath();
        if let Ok(data) = fs::read_to_string(&path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            PluginsConfig::default()
        }
    }

    /// Saves plugin activation preferences
    pub fn save_config(config: &PluginsConfig) -> Result<(), String> {
        let path = Self::get_config_filepath();
        let json = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Error serializing plugins.json: {}", e))?;
        fs::write(&path, json).map_err(|e| format!("Error writing plugins.json: {}", e))?;
        Ok(())
    }

    /// Bundled plugins directory (no plugins are pre-installed by default)
    pub fn get_bundled_plugins_directory() -> Option<PathBuf> {
        None
    }

    /// Returns the list of official built-in plugins.
    /// Atena Studio adopts a ZERO pre-installed plugins philosophy:
    /// optional plugins (such as Cognitive Memory and AGY Connector) are maintained in independent
    /// repositories and can be installed by the user into ~/.atena/plugins/.
    pub fn get_builtin_plugins() -> Vec<PluginManifest> {
        Vec::new()
    }

    /// Resolves configuration values combining manifest defaults and saved overrides
    fn resolve_plugin_settings(
        manifest: &PluginManifest,
        config: &PluginsConfig,
    ) -> HashMap<String, serde_json::Value> {
        let mut values = HashMap::new();
        // 1. Schema defaults
        for field in &manifest.settings_schema {
            if let Some(ref def) = field.default {
                values.insert(field.id.clone(), def.clone());
            }
        }
        // 2. Settings map defaults
        for (k, v) in &manifest.settings {
            values.insert(k.clone(), v.clone());
        }
        // 3. User saved overrides
        if let Some(saved) = config.plugin_settings.get(&manifest.id) {
            for (k, v) in saved {
                values.insert(k.clone(), v.clone());
            }
        }
        values
    }

    /// Scans the `~/.atena/plugins/` directory for plugins installed by developers/community
    pub fn scan_external_plugins() -> Vec<(PluginManifest, PathBuf)> {
        let plugins_dir = Self::get_plugins_directory();
        let mut result = Vec::new();

        if let Ok(entries) = fs::read_dir(&plugins_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let manifest_path = path.join("plugin.json");
                    if manifest_path.exists() && manifest_path.is_file() {
                        if let Ok(content) = fs::read_to_string(&manifest_path) {
                            if let Ok(manifest) = serde_json::from_str::<PluginManifest>(&content) {
                                result.push((manifest, path));
                            }
                        }
                    }
                }
            }
        }

        result
    }

    /// Lists all plugins (built-in + external) applying saved preferences
    pub fn list_all_plugins() -> Vec<PluginInfo> {
        let config = Self::load_config();
        let mut list = Vec::new();

        // 1. Built-in plugins
        for builtin in Self::get_builtin_plugins() {
            let is_enabled = config
                .enabled_plugins
                .get(&builtin.id)
                .copied()
                .unwrap_or(builtin.enabled);

            let settings_values = Self::resolve_plugin_settings(&builtin, &config);
            let has_settings = !builtin.settings.is_empty() || !builtin.settings_schema.is_empty();

            let directory = Self::get_bundled_plugins_directory()
                .map(|p| p.join(&builtin.id).to_string_lossy().to_string());

            list.push(PluginInfo {
                id: builtin.id,
                name: builtin.name,
                version: builtin.version,
                description: builtin.description,
                author: builtin.author,
                icon: builtin.icon,
                enabled: is_enabled,
                is_builtin: true,
                category: builtin.category,
                hooks: builtin.hooks,
                views: builtin.views,
                directory,
                entry: builtin.entry,
                native_entry: builtin.native_entry,
                readme: builtin.readme,
                doc: builtin.doc,
                locales: builtin.locales,
                repository: builtin.repository,
                license: builtin.license,
                has_settings,
                settings_schema: builtin.settings_schema,
                settings_values,
            });
        }

        // 2. External plugins scanned in ~/.atena/plugins/
        for (manifest, path) in Self::scan_external_plugins() {
            let is_enabled = config
                .enabled_plugins
                .get(&manifest.id)
                .copied()
                .unwrap_or(manifest.enabled);

            let settings_values = Self::resolve_plugin_settings(&manifest, &config);
            let has_settings = !manifest.settings.is_empty() || !manifest.settings_schema.is_empty();

            list.push(PluginInfo {
                id: manifest.id,
                name: manifest.name,
                version: manifest.version,
                description: manifest.description,
                author: manifest.author,
                icon: manifest.icon,
                enabled: is_enabled,
                is_builtin: false,
                category: manifest.category,
                hooks: manifest.hooks,
                views: manifest.views,
                directory: Some(path.to_string_lossy().to_string()),
                entry: manifest.entry,
                native_entry: manifest.native_entry,
                readme: manifest.readme,
                doc: manifest.doc,
                locales: manifest.locales,
                repository: manifest.repository,
                license: manifest.license,
                has_settings,
                settings_schema: manifest.settings_schema,
                settings_values,
            });
        }

        list
    }

    /// Executes a command on the plugin's native Rust binary via stdin/stdout JSON IPC
    pub fn run_plugin_native(
        plugin_id: &str,
        command: &str,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let mut binary_path = None;

        // Search in external plugins
        for (manifest, path) in Self::scan_external_plugins() {
            if manifest.id == plugin_id {
                if let Some(ref entry) = manifest.native_entry {
                    let full = path.join(entry);
                    if full.exists() {
                        binary_path = Some(full);
                        break;
                    }
                }
            }
        }

        // Search in internal/bundled plugins
        if binary_path.is_none() {
            if let Some(bundled_dir) = Self::get_bundled_plugins_directory() {
                let p_dir = bundled_dir.join(plugin_id);
                let manifest_path = p_dir.join("plugin.json");
                if manifest_path.exists() {
                    if let Ok(content) = fs::read_to_string(&manifest_path) {
                        if let Ok(manifest) = serde_json::from_str::<PluginManifest>(&content) {
                            if let Some(ref entry) = manifest.native_entry {
                                let full = p_dir.join(entry);
                                if full.exists() {
                                    binary_path = Some(full);
                                }
                            }
                        }
                    }
                }
            }
        }

        let bin = binary_path.ok_or_else(|| {
            format!(
                "Native binary not found for plugin '{}'. Verify the executable exists and that 'native_entry' is correct in plugin.json.",
                plugin_id
            )
        })?;

        let mut child = silent_command(&bin)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn native binary for plugin '{}': {}", plugin_id, e))?;

        let input_obj = serde_json::json!({
            "command": command,
            "payload": payload,
        });

        let input_bytes = serde_json::to_vec(&input_obj)
            .map_err(|e| format!("Error serializing payload for native plugin: {}", e))?;

        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            let _ = stdin.write_all(&input_bytes);
            let _ = stdin.write_all(b"\n");
        }

        let output = child
            .wait_with_output()
            .map_err(|e| format!("Error waiting for response from native plugin '{}': {}", plugin_id, e))?;

        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Native plugin '{}' exited with error: {}", plugin_id, err_msg.trim()));
        }

        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let res: serde_json::Value = serde_json::from_str(stdout_str.trim())
            .unwrap_or_else(|_| serde_json::json!({ "output": stdout_str.trim() }));

        Ok(res)
    }

    /// Reads the plugin's frontend script for dynamic evaluation in the WebView
    pub fn read_plugin_script(plugin_id: &str) -> Result<String, String> {
        for (manifest, path) in Self::scan_external_plugins() {
            if manifest.id == plugin_id {
                if let Some(ref entry) = manifest.entry {
                    let file_path = path.join(entry);
                    if file_path.exists() {
                        return fs::read_to_string(&file_path)
                            .map_err(|e| format!("Error reading plugin script: {}", e));
                    }
                }
            }
        }

        if let Some(bundled_dir) = Self::get_bundled_plugins_directory() {
            let p_dir = bundled_dir.join(plugin_id);
            let manifest_path = p_dir.join("plugin.json");
            if manifest_path.exists() {
                if let Ok(content) = fs::read_to_string(&manifest_path) {
                    if let Ok(manifest) = serde_json::from_str::<PluginManifest>(&content) {
                        if let Some(ref entry) = manifest.entry {
                            let file_path = p_dir.join(entry);
                            if file_path.exists() {
                                return fs::read_to_string(&file_path)
                                    .map_err(|e| format!("Error reading plugin script: {}", e));
                            }
                        }
                    }
                }
            }
        }

        Err(format!("No frontend script found for plugin '{}'", plugin_id))
    }

    /// Saves settings for a specific plugin
    pub fn save_plugin_settings(
        id: &str,
        settings: HashMap<String, serde_json::Value>,
    ) -> Result<(), String> {
        let mut config = Self::load_config();
        config.plugin_settings.insert(id.to_string(), settings);
        Self::save_config(&config)?;
        Ok(())
    }

    /// Returns saved settings for a specific plugin
    pub fn get_plugin_settings(id: &str) -> HashMap<String, serde_json::Value> {
        let config = Self::load_config();
        config.plugin_settings.get(id).cloned().unwrap_or_default()
    }

    /// Toggles a plugin active state by ID
    pub fn toggle_plugin(id: &str, enabled: bool) -> Result<bool, String> {
        let mut config = Self::load_config();
        config.enabled_plugins.insert(id.to_string(), enabled);
        Self::save_config(&config)?;
        Ok(enabled)
    }

    /// Checks if a specific plugin is enabled
    pub fn is_plugin_enabled(id: &str) -> bool {
        let config = Self::load_config();
        if let Some(&enabled) = config.enabled_plugins.get(id) {
            return enabled;
        }

        // Check default for built-in
        for builtin in Self::get_builtin_plugins() {
            if builtin.id == id {
                return builtin.enabled;
            }
        }

        // Check manifest for external
        for (manifest, _) in Self::scan_external_plugins() {
            if manifest.id == id {
                return manifest.enabled;
            }
        }

        false
    }
}
