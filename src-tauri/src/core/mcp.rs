use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum McpTransportType {
    Stdio,
    Sse,
    Http,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub id: String,
    pub name: String,
    pub transport: McpTransportType,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Option<Vec<String>>,
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_permission_mode")]
    pub permission_mode: String,
    #[serde(default)]
    pub tool_permissions: HashMap<String, String>,
    #[serde(default)]
    pub tool_labels: HashMap<String, String>,
    #[serde(default)]
    pub tool_field_labels: HashMap<String, HashMap<String, String>>,
    #[serde(default)]
    pub disabled_tools: Vec<String>,
}

fn default_true() -> bool {
    true
}

fn default_permission_mode() -> String {
    "ask".to_string()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpToolDefinition {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "inputSchema", default)]
    pub input_schema: Value,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub field_labels: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpToolWithServer {
    pub server_id: String,
    pub server_name: String,
    pub tool: McpToolDefinition,
    pub enabled: bool,
    #[serde(default = "default_permission_mode")]
    pub permission_mode: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
    #[serde(default)]
    pub server_id: Option<String>,
    #[serde(default)]
    pub server_name: Option<String>,
    #[serde(default)]
    pub permission_mode: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub rejection_reason: Option<String>,
    #[serde(default)]
    pub result: Option<Value>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub field_labels: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct McpToolResult {
    pub call_id: String,
    pub tool_name: String,
    pub content: Value,
    pub is_error: bool,
}

