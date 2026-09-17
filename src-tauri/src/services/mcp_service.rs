use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;

use crate::core::mcp::{McpServerConfig, McpToolDefinition, McpToolWithServer, McpTransportType};
use crate::services::backend::BackendManager;

pub struct McpManager {
    servers: Arc<Mutex<Vec<McpServerConfig>>>,
}

impl McpManager {
    pub fn new() -> Self {
        let default_servers = Self::load_servers_from_disk().unwrap_or_else(|| Self::default_servers());
        Self {
            servers: Arc::new(Mutex::new(default_servers)),
        }
    }

    fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let dir = PathBuf::from(home).join(".config").join("atena");
        let _ = std::fs::create_dir_all(&dir);
        dir.join("mcp_servers.json")
    }

    fn load_servers_from_disk() -> Option<Vec<McpServerConfig>> {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(servers) = serde_json::from_str::<Vec<McpServerConfig>>(&content) {
                    return Some(servers);
                }
            }
        }
        None
    }

    pub fn default_servers() -> Vec<McpServerConfig> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        vec![
            McpServerConfig {
                id: "filesystem".to_string(),
                name: "Filesystem (Arquivos Locais)".to_string(),
                transport: McpTransportType::Stdio,
                command: Some("npx".to_string()),
                args: Some(vec![
                    "-y".to_string(),
                    "@modelcontextprotocol/server-filesystem".to_string(),
                    home,
                ]),
                env: None,
                url: None,
                headers: None,
                enabled: false,
                description: Some("Permite ler, listar e navegar por arquivos e diretórios locais.".to_string()),
                permission_mode: "ask".to_string(),
                tool_permissions: std::collections::HashMap::new(),
                tool_labels: std::collections::HashMap::new(),
                tool_field_labels: std::collections::HashMap::new(),
                disabled_tools: Vec::new(),
            },
            McpServerConfig {
                id: "fetch".to_string(),
                name: "Web Search & Fetch".to_string(),
                transport: McpTransportType::Stdio,
                command: Some("uvx".to_string()),
                args: Some(vec!["duckduckgo-mcp-server".to_string()]),
                env: None,
                url: None,
                headers: None,
                enabled: false,
                description: Some("Permite realizar buscas na web (DuckDuckGo) e extrair conteúdo de páginas diretamente nas conversas.".to_string()),
                permission_mode: "ask".to_string(),
                tool_permissions: std::collections::HashMap::new(),
                tool_labels: std::collections::HashMap::new(),
                tool_field_labels: std::collections::HashMap::new(),
                disabled_tools: Vec::new(),
            },
            McpServerConfig {
                id: "sqlite".to_string(),
                name: "SQLite (Banco de Dados Local)".to_string(),
                transport: McpTransportType::Stdio,
                command: Some("npx".to_string()),
                args: Some(vec![
                    "-y".to_string(),
                    "@modelcontextprotocol/server-sqlite".to_string(),
                    "--db-path".to_string(),
                    "./dados.db".to_string(),
                ]),
                env: None,
                url: None,
                headers: None,
                enabled: false,
                description: Some("Permite consultar, criar tabelas e analisar dados em bancos SQLite locais.".to_string()),
                permission_mode: "ask".to_string(),
                tool_permissions: std::collections::HashMap::new(),
                tool_labels: std::collections::HashMap::new(),
                tool_field_labels: std::collections::HashMap::new(),
                disabled_tools: Vec::new(),
            },
        ]
    }

    pub async fn get_servers(&self) -> Vec<McpServerConfig> {
        let list = self.servers.lock().await;
        list.clone()
    }

    pub async fn save_servers(&self, servers: Vec<McpServerConfig>) -> Result<(), String> {
        let path = Self::config_path();
        let json_str = serde_json::to_string_pretty(&servers)
            .map_err(|e| format!("Erro ao serializar servidores MCP: {}", e))?;
        std::fs::write(&path, json_str)
            .map_err(|e| format!("Erro ao salvar arquivo de configuração MCP: {}", e))?;

        let mut guard = self.servers.lock().await;
        *guard = servers;
        Ok(())
    }

    fn find_in_augmented_path(binary_name: &str) -> Option<PathBuf> {
        let aug_path = BackendManager::augmented_path();
        for dir in std::env::split_paths(&aug_path) {
            let candidate = dir.join(binary_name);
            if candidate.is_file() {
                return Some(candidate);
            }
            #[cfg(target_os = "windows")]
            {
                let candidate_exe = dir.join(format!("{}.exe", binary_name));
                if candidate_exe.is_file() {
                    return Some(candidate_exe);
                }
            }
        }
        None
    }

    fn resolve_command_and_args(server: &McpServerConfig) -> Result<(PathBuf, Vec<String>), String> {
        let raw_cmd = server.command.as_deref().ok_or("Comando stdio não especificado")?.trim();
        if raw_cmd.is_empty() {
            return Err("Comando stdio não pode estar vazio".to_string());
        }

        let mut configured_args = server.args.clone().unwrap_or_default();
        let cmd_str;

        // If the command string contains whitespace and no separate args were supplied, split whitespace
        if raw_cmd.contains(' ') && configured_args.is_empty() {
            let parts: Vec<&str> = raw_cmd.split_whitespace().collect();
            if parts.is_empty() {
                return Err("Comando stdio inválido".to_string());
            }
            cmd_str = parts[0].to_string();
            configured_args = parts[1..].iter().map(|s| s.to_string()).collect();
        } else {
            cmd_str = raw_cmd.to_string();
        }

        // Auto-provision bundled tools if needed
        crate::services::runtime::RuntimeManager::auto_provision_bundled_tools();

        let base_cmd_name = std::path::Path::new(&cmd_str)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&cmd_str);

        // 1. Handle uvx and uv
        if base_cmd_name == "uvx" || base_cmd_name == "uvx.exe" {
            // Find uv binary (either uv or uvx)
            let uv_bin = crate::services::runtime::RuntimeManager::resolve_binary("uv")
                .or_else(|| crate::services::runtime::RuntimeManager::resolve_binary("uvx"))
                .or_else(|| Self::find_in_augmented_path("uv"))
                .or_else(|| Self::find_in_augmented_path("uvx"));

            if let Some(bin) = uv_bin {
                let is_already_tool_run = configured_args.first().map(|s| s.as_str()) == Some("tool")
                    || configured_args.first().map(|s| s.as_str()) == Some("x");
                if !is_already_tool_run {
                    configured_args.insert(0, "run".to_string());
                    configured_args.insert(0, "tool".to_string());
                }
                return Ok((bin, configured_args));
            }
        } else if base_cmd_name == "uv" || base_cmd_name == "uv.exe" {
            let uv_bin = crate::services::runtime::RuntimeManager::resolve_binary("uv")
                .or_else(|| Self::find_in_augmented_path("uv"));

            if let Some(bin) = uv_bin {
                let known_subcommands = [
                    "run", "tool", "pip", "venv", "cache", "python", "version",
                    "self", "tree", "init", "add", "remove", "sync", "lock",
                    "export", "publish", "build", "format", "help"
                ];
                let first_arg = configured_args.first().map(|s| s.as_str()).unwrap_or_default();
                if !known_subcommands.contains(&first_arg) {
                    configured_args.insert(0, "run".to_string());
                    configured_args.insert(0, "tool".to_string());
                }
                return Ok((bin, configured_args));
            }
        }

        // 2. Check if command is an existing file path (absolute or relative)
        let cmd_path = PathBuf::from(&cmd_str);
        if (cmd_path.is_absolute() || cmd_str.contains('/') || cmd_str.contains('\\')) && cmd_path.is_file() {
            return Ok((cmd_path, configured_args));
        }

        // 3. Search in Atena RuntimeManager resolution
        if let Some(resolved) = crate::services::runtime::RuntimeManager::resolve_binary(&cmd_str) {
            return Ok((resolved, configured_args));
        }

        // 4. Search in augmented PATH
        if let Some(resolved) = Self::find_in_augmented_path(&cmd_str) {
            return Ok((resolved, configured_args));
        }

        // 5. Special check for python/python3
        if base_cmd_name == "python" || base_cmd_name == "python3" {
            if let Some(py) = crate::services::runtime::RuntimeManager::get_python() {
                return Ok((py, configured_args));
            }
        }

        // 6. Fallback: return PathBuf from cmd_str
        Ok((PathBuf::from(cmd_str), configured_args))
    }

    fn spawn_stderr_collector(stderr: tokio::process::ChildStderr) -> Arc<Mutex<String>> {
        let buffer = Arc::new(Mutex::new(String::new()));
        let buffer_clone = buffer.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                let mut buf = buffer_clone.lock().await;
                if buf.len() < 8192 {
                    if !buf.is_empty() {
                        buf.push('\n');
                    }
                    buf.push_str(&line);
                }
            }
        });
        buffer
    }

    async fn read_stdio_response(
        reader: &mut tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
        target_id: u64,
        timeout: Duration,
        stderr_buf: Option<&Arc<Mutex<String>>>,
    ) -> Result<Value, String> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            let now = tokio::time::Instant::now();
            if now >= deadline {
                let stderr_info = if let Some(buf) = stderr_buf {
                    let s = buf.lock().await.clone();
                    if !s.is_empty() {
                        format!(" (Detalhes/stderr: {})", s)
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };
                return Err(format!("Tempo limite excedido ao aguardar resposta do servidor MCP{}", stderr_info));
            }
            let remaining = deadline - now;
            let line_opt = match tokio::time::timeout(remaining, reader.next_line()).await {
                Ok(Ok(opt)) => opt,
                Ok(Err(e)) => {
                    let stderr_info = if let Some(buf) = stderr_buf {
                        let s = buf.lock().await.clone();
                        if !s.is_empty() {
                            format!(" (Logs: {})", s)
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    };
                    return Err(format!("Erro de leitura MCP: {}{}", e, stderr_info));
                }
                Err(_) => {
                    let stderr_info = if let Some(buf) = stderr_buf {
                        let s = buf.lock().await.clone();
                        if !s.is_empty() {
                            format!(" (Logs: {})", s)
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    };
                    return Err(format!("Tempo limite excedido ao aguardar linha do processo MCP{}", stderr_info));
                }
            };

            let line = match line_opt {
                Some(l) => l,
                None => {
                    let stderr_info = if let Some(buf) = stderr_buf {
                        let s = buf.lock().await.clone();
                        if !s.is_empty() {
                            format!(" (Logs/Stderr: {})", s)
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    };
                    return Err(format!("Servidor MCP encerrou stdout inesperadamente{}", stderr_info));
                }
            };

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if let Ok(json_resp) = serde_json::from_str::<Value>(trimmed) {
                if json_resp.get("id").and_then(|v| v.as_u64()) == Some(target_id) {
                    if let Some(error) = json_resp.get("error") {
                        return Err(format!("Erro retornado pelo servidor MCP: {:?}", error));
                    }
                    return Ok(json_resp);
                }
                // Skip notifications like {"method": "notifications/message", ...}
            }
        }
    }

    fn parse_http_json_response(raw_text: &str) -> Result<Value, String> {
        let trimmed = raw_text.trim();
        // 1. Direct JSON parse
        if let Ok(v) = serde_json::from_str::<Value>(trimmed) {
            return Ok(v);
        }

        // 2. Parse SSE lines ("data: {...}")
        for line in trimmed.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("data:") {
                let data_str = rest.trim();
                if let Ok(v) = serde_json::from_str::<Value>(data_str) {
                    return Ok(v);
                }
            }
        }

        Err(format!("Não foi possível converter a resposta MCP em JSON válido. Conteúdo recebido:\n{}", trimmed))
    }

    async fn execute_http_rpc(
        server: &McpServerConfig,
        method: &str,
        params: Value,
        timeout_secs: u64,
    ) -> Result<Value, String> {
        let url = server.url.as_deref().ok_or("URL do servidor MCP não informada")?.trim();
        if url.is_empty() {
            return Err("URL do servidor MCP não pode estar vazia".to_string());
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|e| e.to_string())?;

        // 1. Send initialize request (id: 1)
        let init_req = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "roots": { "listChanged": true },
                    "sampling": {}
                },
                "clientInfo": {
                    "name": "Atena Studio",
                    "version": "0.2.0"
                }
            }
        });

        let mut req_init = client.post(url)
            .header("Accept", "application/json, text/event-stream")
            .header("Content-Type", "application/json");

        if let Some(headers_map) = &server.headers {
            for (k, v) in headers_map {
                req_init = req_init.header(k, v);
            }
        }

        let resp_init = req_init.json(&init_req).send().await.map_err(|e| format!("Falha no handshake HTTP MCP para '{}': {}", url, e))?;
        if !resp_init.status().is_success() {
            let status = resp_init.status();
            let text = resp_init.text().await.unwrap_or_default();
            return Err(format!("Falha no handshake initialize (HTTP {}): {}", status, text));
        }

        // Extract session headers (mcp-session-id, x-session-id, session-id, set-cookie)
        let mut session_headers = Vec::new();
        for (header_name, header_val) in resp_init.headers() {
            let name_str = header_name.as_str().to_lowercase();
            if name_str == "mcp-session-id" || name_str == "x-session-id" || name_str == "session-id" || name_str == "set-cookie" {
                if let Ok(val_str) = header_val.to_str() {
                    if name_str == "set-cookie" {
                        session_headers.push(("Cookie".to_string(), val_str.to_string()));
                    } else {
                        session_headers.push((header_name.as_str().to_string(), val_str.to_string()));
                    }
                }
            }
        }

        let raw_init_text = resp_init.text().await.map_err(|e| format!("Erro ao ler resposta do handshake MCP: {}", e))?;
        let json_init = Self::parse_http_json_response(&raw_init_text)?;
        if let Some(err) = json_init.get("error") {
            return Err(format!("Erro retornado no handshake MCP: {:?}", err));
        }

        // 2. Send notifications/initialized
        let notif_req = json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        let mut req_notif = client.post(url)
            .header("Accept", "application/json, text/event-stream")
            .header("Content-Type", "application/json");
        if let Some(headers_map) = &server.headers {
            for (k, v) in headers_map {
                req_notif = req_notif.header(k, v);
            }
        }
        for (k, v) in &session_headers {
            req_notif = req_notif.header(k, v);
        }
        let _ = req_notif.json(&notif_req).send().await;

        // 3. Send target method request (id: 2)
        let main_req = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": method,
            "params": params
        });

        let mut req_main = client.post(url)
            .header("Accept", "application/json, text/event-stream")
            .header("Content-Type", "application/json");

        if let Some(headers_map) = &server.headers {
            for (k, v) in headers_map {
                req_main = req_main.header(k, v);
            }
        }
        for (k, v) in &session_headers {
            req_main = req_main.header(k, v);
        }

        let resp_main = req_main.json(&main_req).send().await.map_err(|e| format!("Falha na requisição '{}' para '{}': {}", method, url, e))?;
        if !resp_main.status().is_success() {
            let status = resp_main.status();
            let text = resp_main.text().await.unwrap_or_default();
            return Err(format!("Servidor MCP HTTP respondeu com status {}: {}", status, text));
        }

        let raw_main_text = resp_main.text().await.map_err(|e| format!("Erro ao ler resposta HTTP MCP: {}", e))?;
        let json_resp = Self::parse_http_json_response(&raw_main_text)?;
        if let Some(err) = json_resp.get("error") {
            return Err(format!("Erro retornado pelo servidor MCP: {:?}", err));
        }

        Ok(json_resp)
    }

    /// Queries an MCP server for its list of available tools
    pub async fn inspect_server_tools(server: &McpServerConfig) -> Result<Vec<McpToolDefinition>, String> {
        match server.transport {
            McpTransportType::Stdio => {
                let (cmd_path, args) = Self::resolve_command_and_args(server)?;
                let aug_path = BackendManager::augmented_path();

                let mut cmd = Command::new(&cmd_path);
                cmd.env("PATH", &aug_path);
                if let Some(py_path) = crate::services::runtime::RuntimeManager::isolated_python() {
                    cmd.env("UV_PYTHON", py_path);
                }
                if let Some(env_map) = &server.env {
                    for (k, v) in env_map {
                        cmd.env(k, v);
                    }
                }
                cmd.args(&args);
                cmd.stdin(Stdio::piped());
                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::piped());

                let cmd_display = cmd_path.file_name().and_then(|n| n.to_str()).unwrap_or_else(|| cmd_path.to_str().unwrap_or("stdio"));
                let mut child = cmd.spawn().map_err(|e| format!("Falha ao iniciar processo MCP '{}': {}", cmd_display, e))?;
                let mut stdin = child.stdin.take().ok_or("Falha ao abrir stdin do processo MCP")?;
                let stdout = child.stdout.take().ok_or("Falha ao abrir stdout do processo MCP")?;
                let stderr_handle = child.stderr.take().map(Self::spawn_stderr_collector);
                let mut reader = BufReader::new(stdout).lines();

                // 1. Send initialize request
                let init_req = json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "initialize",
                    "params": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {
                            "roots": { "listChanged": true },
                            "sampling": {}
                        },
                        "clientInfo": {
                            "name": "Atena Studio",
                            "version": "0.2.0"
                        }
                    }
                });

                let init_payload = format!("{}\n", init_req);
                if let Err(e) = stdin.write_all(init_payload.as_bytes()).await {
                    let stderr_info = if let Some(ref buf) = stderr_handle {
                        let s = buf.lock().await.clone();
                        if !s.is_empty() { format!(" (Stderr: {})", s) } else { String::new() }
                    } else { String::new() };
                    return Err(format!("Erro no handshake MCP: {}{}", e, stderr_info));
                }
                let _ = stdin.flush().await;

                // Read initialize response (id: 1)
                let _ = Self::read_stdio_response(&mut reader, 1, Duration::from_secs(60), stderr_handle.as_ref()).await?;

                // Send notifications/initialized
                let initialized_notif = json!({
                    "jsonrpc": "2.0",
                    "method": "notifications/initialized"
                });
                let _ = stdin.write_all(format!("{}\n", initialized_notif).as_bytes()).await;
                let _ = stdin.flush().await;

                // 2. Query tools/list (id: 2)
                let tools_req = json!({
                    "jsonrpc": "2.0",
                    "id": 2,
                    "method": "tools/list",
                    "params": {}
                });
                let tools_payload = format!("{}\n", tools_req);
                if let Err(e) = stdin.write_all(tools_payload.as_bytes()).await {
                    let stderr_info = if let Some(ref buf) = stderr_handle {
                        let s = buf.lock().await.clone();
                        if !s.is_empty() { format!(" (Stderr: {})", s) } else { String::new() }
                    } else { String::new() };
                    return Err(format!("Erro ao requisitar tools/list: {}{}", e, stderr_info));
                }
                let _ = stdin.flush().await;

                let json_resp = Self::read_stdio_response(&mut reader, 2, Duration::from_secs(20), stderr_handle.as_ref()).await?;

                // Terminate child process gracefully
                let _ = child.kill().await;

                let mut tool_defs = Vec::new();
                if let Some(tools_arr) = json_resp.get("result").and_then(|r| r.get("tools")).and_then(|t| t.as_array()) {
                    for t in tools_arr {
                        if let Some(name) = t.get("name").and_then(|n| n.as_str()) {
                            let desc = t.get("description").and_then(|d| d.as_str()).map(|s| s.to_string());
                            let schema = t.get("inputSchema").cloned().unwrap_or(json!({ "type": "object" }));
                            let label = server.tool_labels.get(name).cloned();
                            let field_labels = server.tool_field_labels.get(name).cloned().unwrap_or_default();
                            tool_defs.push(McpToolDefinition {
                                name: name.to_string(),
                                description: desc,
                                input_schema: schema,
                                label,
                                field_labels,
                            });
                        }
                    }
                }

                Ok(tool_defs)
            }
            McpTransportType::Sse | McpTransportType::Http => {
                let json_resp = Self::execute_http_rpc(server, "tools/list", json!({}), 15).await?;
                let mut tool_defs = Vec::new();
                if let Some(tools_arr) = json_resp.get("result").and_then(|r| r.get("tools")).and_then(|t| t.as_array()) {
                    for t in tools_arr {
                        if let Some(name) = t.get("name").and_then(|n| n.as_str()) {
                            let desc = t.get("description").and_then(|d| d.as_str()).map(|s| s.to_string());
                            let schema = t.get("inputSchema").cloned().unwrap_or(json!({ "type": "object" }));
                            let label = server.tool_labels.get(name).cloned();
                            let field_labels = server.tool_field_labels.get(name).cloned().unwrap_or_default();
                            tool_defs.push(McpToolDefinition {
                                name: name.to_string(),
                                description: desc,
                                input_schema: schema,
                                label,
                                field_labels,
                            });
                        }
                    }
                }
                Ok(tool_defs)
            }
        }
    }

    /// Returns the native built-in tools of Atena (Episodic Memory, History Search, Graph Query)
    pub fn native_atena_tools() -> Vec<McpToolWithServer> {
        let mut f1 = std::collections::HashMap::new();
        f1.insert("query".to_string(), "Search Query".to_string());
        f1.insert("limit".to_string(), "Max Limit".to_string());

        let mut f2 = std::collections::HashMap::new();
        f2.insert("identifier".to_string(), "Episode Identifier".to_string());

        let mut f3 = std::collections::HashMap::new();
        f3.insert("query".to_string(), "Entity / Concept".to_string());

        let mut f4 = std::collections::HashMap::new();
        f4.insert("name".to_string(), "Skill Name".to_string());
        f4.insert("description".to_string(), "Description".to_string());
        f4.insert("triggers".to_string(), "Triggers".to_string());
        f4.insert("steps".to_string(), "Execution Steps".to_string());
        f4.insert("scripts".to_string(), "Automated Scripts".to_string());

        let mut f5 = std::collections::HashMap::new();
        f5.insert("id".to_string(), "Skill Identifier".to_string());
        f5.insert("name".to_string(), "Skill Name".to_string());
        f5.insert("description".to_string(), "Description".to_string());
        f5.insert("triggers".to_string(), "Triggers".to_string());
        f5.insert("steps".to_string(), "Execution Steps".to_string());
        f5.insert("scripts".to_string(), "Automated Scripts".to_string());
        f5.insert("refinement_note".to_string(), "Refinement Note".to_string());

        let mut f6 = std::collections::HashMap::new();
        f6.insert("command".to_string(), "CLI Command".to_string());
        f6.insert("timeout_ms".to_string(), "Timeout (ms)".to_string());

        let mut f7 = std::collections::HashMap::new();
        f7.insert("slug".to_string(), "Skill Identifier".to_string());
        f7.insert("script_file".to_string(), "Script File".to_string());
        f7.insert("file_name".to_string(), "Script File".to_string());
        f7.insert("args".to_string(), "Arguments".to_string());
        f7.insert("timeout_ms".to_string(), "Timeout (ms)".to_string());

        vec![
            McpToolWithServer {
                server_id: "atena_native".to_string(),
                server_name: "Atena Core (Memory & Episodes)".to_string(),
                tool: McpToolDefinition {
                    name: "atena_search_episodes".to_string(),
                    description: Some("Searches cognitive episodes and past conversation history by keywords, dates, or context.".to_string()),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Keywords or search terms for recorded episodes"
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of episodes to return (default: 5)"
                            }
                        },
                        "required": ["query"]
                    }),
                    label: Some("Search Episodes".to_string()),
                    field_labels: f1,
                },
                enabled: true,
                permission_mode: "auto".to_string(),
            },
            McpToolWithServer {
                server_id: "atena_native".to_string(),
                server_name: "Atena Core (Memory & Episodes)".to_string(),
                tool: McpToolDefinition {
                    name: "atena_read_episode".to_string(),
                    description: Some("Reads the complete content of a specific cognitive episode from its number or filename.".to_string()),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "identifier": {
                                "type": "string",
                                "description": "Episode number (e.g. '1', '12') or filename (e.g. 'episodio_20260903_0012.md')"
                            }
                        },
                        "required": ["identifier"]
                    }),
                    label: Some("Read Episode".to_string()),
                    field_labels: f2,
                },
                enabled: true,
                permission_mode: "auto".to_string(),
            },
            McpToolWithServer {
                server_id: "atena_native".to_string(),
                server_name: "Atena Core (Memory & Episodes)".to_string(),
                tool: McpToolDefinition {
                    name: "atena_search_memory".to_string(),
                    description: Some("Queries facts, preferences, and relationships saved in Atena's permanent associative memory graph.".to_string()),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Entity name, subject, or concept to query in memory"
                            }
                        },
                        "required": ["query"]
                    }),
                    label: Some("Search Memory".to_string()),
                    field_labels: f3,
                },
                enabled: true,
                permission_mode: "auto".to_string(),
            },
            McpToolWithServer {
                server_id: "skills".to_string(),
                server_name: "Procedural Skills".to_string(),
                tool: McpToolDefinition {
                    name: "create_procedural_skill".to_string(),
                    description: Some("Creates and registers a new procedural skill/workflow in Atena Studio, with triggers, sequential steps, and optional scripts (.sh, .py, .js).".to_string()),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "name": {
                                "type": "string",
                                "description": "Concise and clear name of the procedural skill (e.g. 'Tech News RSS', 'Disk Space Monitor')"
                            },
                            "description": {
                                "type": "string",
                                "description": "Clear explanation of the skill objective and execution"
                            },
                            "triggers": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "Trigger phrases or keywords activating this skill when requested by user"
                            },
                            "steps": {
                                "type": "array",
                                "items": {
                                     "type": "object",
                                     "properties": {
                                         "order": { "type": "integer", "description": "Sequential step order (1, 2, ...)" },
                                         "instruction": { "type": "string", "description": "Textual instruction describing what this step does" },
                                         "command": { "type": "string", "description": "CLI command to run in terminal (optional)" },
                                         "script_file": { "type": "string", "description": "Script filename to execute inside scripts/ folder (optional)" }
                                     },
                                     "required": ["order", "instruction"]
                                },
                                "description": "Sequential execution steps of the skill"
                            },
                            "scripts": {
                                "type": "array",
                                "items": {
                                     "type": "object",
                                     "properties": {
                                         "filename": { "type": "string", "description": "Script filename (e.g. 'fetch.py', 'audit.sh')" },
                                         "content": { "type": "string", "description": "Full executable source code of the script" }
                                     },
                                     "required": ["filename", "content"]
                                },
                                "description": "Auxiliary script files created in ~/.atena/skills/<slug>/scripts/"
                            }
                        },
                        "required": ["name", "description", "triggers", "steps"]
                    }),
                    label: Some("Create Procedural Skill".to_string()),
                    field_labels: f4,
                },
                enabled: true,
                permission_mode: "ask".to_string(),
            },
            McpToolWithServer {
                server_id: "skills".to_string(),
                server_name: "Procedural Skills".to_string(),
                tool: McpToolDefinition {
                    name: "update_procedural_skill".to_string(),
                    description: Some("Edits, refines, or updates an existing procedural skill in Atena Studio (modifying commands, steps, triggers, scripts, or parameters).".to_string()),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "id": {
                                "type": "string",
                                "description": "Identifier of the skill to edit (e.g. 'skill-tech-news-rss')"
                            },
                            "name": {
                                "type": "string",
                                "description": "Skill name to update (used to locate skill if ID is not provided, or to rename it)"
                            },
                            "description": {
                                "type": "string",
                                "description": "Updated explanation of the skill functionality"
                            },
                            "triggers": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "Updated list of trigger phrases or keywords"
                            },
                            "steps": {
                                "type": "array",
                                "items": {
                                     "type": "object",
                                     "properties": {
                                         "order": { "type": "integer", "description": "Sequential step order (1, 2, ...)" },
                                         "instruction": { "type": "string", "description": "Textual instruction describing what this step does" },
                                         "command": { "type": "string", "description": "CLI command to run" },
                                         "script_file": { "type": "string", "description": "Script filename to execute" }
                                     },
                                     "required": ["order", "instruction"]
                                },
                                "description": "Updated sequential execution steps"
                            },
                            "scripts": {
                                "type": "array",
                                "items": {
                                     "type": "object",
                                     "properties": {
                                         "filename": { "type": "string", "description": "Script filename" },
                                         "content": { "type": "string", "description": "Updated source code of the script" }
                                     },
                                     "required": ["filename", "content"]
                                },
                                "description": "Updated or new script files"
                            },
                            "refinement_note": {
                                "type": "string",
                                "description": "Note or summary explaining the modifications and improvements made in this revision"
                            }
                        },
                        "required": ["name"]
                    }),
                    label: Some("Edit Procedural Skill".to_string()),
                    field_labels: f5,
                },
                enabled: true,
                permission_mode: "ask".to_string(),
            },
            McpToolWithServer {
                server_id: "skills".to_string(),
                server_name: "Procedural Skills".to_string(),
                tool: McpToolDefinition {
                    name: "run_command".to_string(),
                    description: Some("Executes a terminal CLI command associated with a procedural skill or system diagnostic.".to_string()),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "command": {
                                "type": "string",
                                "description": "Exact CLI command to execute in operating system terminal"
                            },
                            "timeout_ms": {
                                "type": "integer",
                                "description": "Execution timeout in milliseconds (optional, default: 120000)"
                            }
                        },
                        "required": ["command"]
                    }),
                    label: Some("Run Command".to_string()),
                    field_labels: f6,
                },
                enabled: true,
                permission_mode: "ask".to_string(),
            },
            McpToolWithServer {
                server_id: "skills".to_string(),
                server_name: "Procedural Skills".to_string(),
                tool: McpToolDefinition {
                    name: "run_skill_script".to_string(),
                    description: Some("Executes a script file (.py, .js, .sh, etc.) located in the scripts/ folder of the specified procedural skill.".to_string()),
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "slug": {
                                "type": "string",
                                "description": "Slug or ID of the procedural skill (e.g. 'tech-news-rss', 'skill-tech-news-rss')"
                            },
                            "script_file": {
                                "type": "string",
                                "description": "Exact name of script file inside skill scripts/ folder (e.g. 'fetch_news.py')"
                            },
                            "args": {
                                "type": "array",
                                "items": { "type": "string" },
                                "description": "Optional command line arguments to pass to the script"
                            },
                            "timeout_ms": {
                                "type": "integer",
                                "description": "Execution timeout in milliseconds (optional, default: 180000)"
                            }
                        },
                        "required": ["slug", "script_file"]
                    }),
                    label: Some("Run Skill Script".to_string()),
                    field_labels: f7,
                },
                enabled: true,
                permission_mode: "ask".to_string(),
            },
        ]
    }

    /// Lists all tools across all enabled MCP servers and native tools
    pub async fn list_all_tools(&self) -> Vec<McpToolWithServer> {
        let mut results = Self::native_atena_tools();
        let servers = self.get_servers().await;

        for server in servers {
            if !server.enabled {
                continue;
            }
            if let Ok(tools) = Self::inspect_server_tools(&server).await {
                for mut tool in tools {
                    if tool.label.is_none() {
                        tool.label = server.tool_labels.get(&tool.name).cloned();
                    }
                    if tool.field_labels.is_empty() {
                        if let Some(fl) = server.tool_field_labels.get(&tool.name) {
                            tool.field_labels = fl.clone();
                        }
                    }
                    let perm = server.tool_permissions.get(&tool.name)
                        .cloned()
                        .unwrap_or_else(|| server.permission_mode.clone());

                    let is_enabled = !server.disabled_tools.contains(&tool.name);

                    results.push(McpToolWithServer {
                        server_id: server.id.clone(),
                        server_name: server.name.clone(),
                        tool,
                        enabled: is_enabled,
                        permission_mode: perm,
                    });
                }
            }
        }

        results
    }

    /// Invokes a specific MCP tool on the target server
    pub async fn call_tool(
        &self,
        server_id: &str,
        tool_name: &str,
        arguments: Value,
    ) -> Result<Value, String> {
        let servers = self.get_servers().await;
        let server = servers.into_iter().find(|s| s.id == server_id).ok_or_else(|| format!("Servidor MCP '{}' não encontrado", server_id))?;

        if !server.enabled {
            return Err(format!("Servidor MCP '{}' está desativado.", server.name));
        }
        if server.disabled_tools.contains(&tool_name.to_string()) {
            return Err(format!("A ferramenta '{}' está desativada no servidor '{}'.", tool_name, server.name));
        }

        match server.transport {
            McpTransportType::Stdio => {
                let (cmd_path, args) = Self::resolve_command_and_args(&server)?;
                let aug_path = BackendManager::augmented_path();

                let mut cmd = Command::new(&cmd_path);
                cmd.env("PATH", &aug_path);
                if let Some(py_path) = crate::services::runtime::RuntimeManager::isolated_python() {
                    cmd.env("UV_PYTHON", py_path);
                }
                if let Some(env_map) = &server.env {
                    for (k, v) in env_map {
                        cmd.env(k, v);
                    }
                }
                cmd.args(&args);
                cmd.stdin(Stdio::piped());
                cmd.stdout(Stdio::piped());
                cmd.stderr(Stdio::piped());

                let cmd_display = cmd_path.file_name().and_then(|n| n.to_str()).unwrap_or_else(|| cmd_path.to_str().unwrap_or("stdio"));
                let mut child = cmd.spawn().map_err(|e| format!("Falha ao iniciar processo MCP '{}': {}", cmd_display, e))?;
                let mut stdin = child.stdin.take().ok_or("Falha ao abrir stdin do processo MCP")?;
                let stdout = child.stdout.take().ok_or("Falha ao abrir stdout do processo MCP")?;
                let stderr_handle = child.stderr.take().map(Self::spawn_stderr_collector);
                let mut reader = BufReader::new(stdout).lines();

                // 1. Handshake
                let init_req = json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "initialize",
                    "params": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {},
                        "clientInfo": { "name": "Atena Studio", "version": "0.2.0" }
                    }
                });
                stdin.write_all(format!("{}\n", init_req).as_bytes()).await.map_err(|e| e.to_string())?;
                stdin.flush().await.map_err(|e| e.to_string())?;

                let _ = Self::read_stdio_response(&mut reader, 1, Duration::from_secs(60), stderr_handle.as_ref()).await?;
                let initialized_notif = json!({ "jsonrpc": "2.0", "method": "notifications/initialized" });
                let _ = stdin.write_all(format!("{}\n", initialized_notif).as_bytes()).await;
                let _ = stdin.flush().await;

                // 2. Call tool (id: 2)
                let call_req = json!({
                    "jsonrpc": "2.0",
                    "id": 2,
                    "method": "tools/call",
                    "params": {
                        "name": tool_name,
                        "arguments": arguments
                    }
                });

                stdin.write_all(format!("{}\n", call_req).as_bytes()).await.map_err(|e| e.to_string())?;
                stdin.flush().await.map_err(|e| e.to_string())?;

                let json_resp = Self::read_stdio_response(&mut reader, 2, Duration::from_secs(45), stderr_handle.as_ref()).await?;

                let _ = child.kill().await;

                let result = json_resp.get("result").cloned().unwrap_or(json!({ "content": [] }));
                Ok(result)
            }
            McpTransportType::Sse | McpTransportType::Http => {
                let json_resp = Self::execute_http_rpc(
                    &server,
                    "tools/call",
                    json!({
                        "name": tool_name,
                        "arguments": arguments
                    }),
                    45,
                ).await?;
                let result = json_resp.get("result").cloned().unwrap_or(json!({ "content": [] }));
                Ok(result)
            }
        }
    }
}
