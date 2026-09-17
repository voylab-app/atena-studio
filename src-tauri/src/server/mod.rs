pub mod dispatcher;

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use crate::{
    services::backend::ModelLoadProgress,
    AppState, ChatChunkPayload, StreamChatRequest,
};

#[derive(Debug, Clone)]
pub struct ServerArgs {
    pub host: String,
    pub port: u16,
    pub token: Option<String>,
    pub static_dir: Option<String>,
}

#[derive(Clone)]
struct ServerContext {
    app_state: Arc<AppState>,
    auth_token: String,
}

#[derive(Deserialize)]
struct AuthQuery {
    token: Option<String>,
}

#[derive(Deserialize)]
struct InvokeRequest {
    cmd: String,
    args: Option<Value>,
}

/// Runs the Atena Studio Headless Web Server.
pub async fn run_server(args: ServerArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let auth_token = args.token.unwrap_or_else(|| {
        format!("{:016x}{:016x}", rand::random::<u128>(), rand::random::<u128>())
    });

    log::info!("🧠 Initializing persistent cognitive memory engine for server mode...");
    let app_state = Arc::new(AppState::new());

    // Resolve static directory for Nuxt frontend
    let static_dir = if let Some(ref dir) = args.static_dir {
        let p = PathBuf::from(dir);
        if p.exists() {
            Some(p)
        } else {
            None
        }
    } else {
        find_frontend_dist()
    };

    let ctx = ServerContext {
        app_state: app_state.clone(),
        auth_token: auth_token.clone(),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let mut router = Router::new()
        .route("/health", get(health_handler))
        .route("/api/ipc/invoke", post(http_invoke_handler))
        .route("/api/ipc/ws", get(ws_handler))
        .layer(cors)
        .with_state(ctx.clone());

    // If static dist exists, serve it with SPA fallback, otherwise serve fallback HTML
    if let Some(ref dist_path) = static_dir {
        log::info!("📁 Serving frontend assets from: {}", dist_path.display());
        let serve_dir = ServeDir::new(dist_path).fallback(tower_http::services::ServeFile::new(
            dist_path.join("index.html"),
        ));
        router = router.fallback_service(serve_dir);
    } else {
        log::warn!("⚠️ Frontend build directory not found. Serving embedded web landing.");
        router = router.fallback(fallback_landing_page);
    }

    let bind_addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

    let banner = format!(
        "\n\
        ╔════════════════════════════════════════════════════════════════════════╗\n\
        ║                     🚀 Atena Studio Web Server                         ║\n\
        ║                                                                        ║\n\
        ║  Access URL:  http://{}:{}/?token={}                                   \n\
        ║  WS Stream:   ws://{}:{}/api/ipc/ws                                    \n\
        ║  Mode:        Headless Server (Desktop Bridge Active)                  ║\n\
        ║  Status:      ONLINE & READY                                           ║\n\
        ╚════════════════════════════════════════════════════════════════════════╝\n",
        args.host, args.port, auth_token, args.host, args.port
    );
    println!("{}", banner);
    log::info!("Atena Studio server running on http://{}", bind_addr);

    axum::serve(listener, router).await?;

    // On shutdown, stop any running backend processes
    app_state.backend_manager.stop_all_sync();
    if let Ok(engine) = app_state.memory_engine.try_lock() {
        if engine.node_count() > 0 {
            let _ = engine.auto_persist_default();
        }
    }

    Ok(())
}

fn find_frontend_dist() -> Option<PathBuf> {
    // 1. Environment variable override
    if let Ok(dir) = std::env::var("ATENA_STATIC_DIR") {
        let p = PathBuf::from(dir);
        if p.exists() && p.join("index.html").exists() {
            return Some(p);
        }
    }

    // 2. Relative to compile-time CARGO_MANIFEST_DIR (guarantees resolution during dev or cargo run from any CWD)
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dev_candidates = [
        manifest_dir.join("../frontend/.output/public"),
        manifest_dir.join("../frontend/dist"),
        manifest_dir.join(".output/public"),
    ];
    for p in &dev_candidates {
        if p.exists() && p.join("index.html").exists() {
            return Some(p.canonicalize().unwrap_or_else(|_| p.clone()));
        }
    }

    // 3. Relative to current executable (resolves symlinks like /usr/local/bin/atena -> bundle / release dir)
    if let Ok(exe_path) = std::env::current_exe() {
        let real_exe = exe_path.canonicalize().unwrap_or(exe_path);
        if let Some(exe_dir) = real_exe.parent() {
            let exe_candidates = [
                exe_dir.join("../Resources/dist"),
                exe_dir.join("../Resources/frontend/.output/public"),
                exe_dir.join("../Resources"),
                exe_dir.join("../../../frontend/.output/public"),
                exe_dir.join("../../frontend/.output/public"),
                exe_dir.join("frontend/.output/public"),
            ];
            for p in &exe_candidates {
                if p.exists() && p.join("index.html").exists() {
                    return Some(p.canonicalize().unwrap_or_else(|_| p.clone()));
                }
            }
        }
    }

    // 4. Current working directory fallback
    let candidates = [
        "frontend/.output/public",
        "../frontend/.output/public",
        ".output/public",
        "frontend/dist",
        "../frontend/dist",
        "dist",
        "resources/dist",
        "app/dist",
    ];
    for c in &candidates {
        let p = PathBuf::from(c);
        if p.exists() && p.join("index.html").exists() {
            return Some(p.canonicalize().unwrap_or(p));
        }
    }
    None
}

async fn health_handler() -> impl IntoResponse {
    axum::Json(json!({
        "status": "ok",
        "app": "Atena Studio",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

fn check_auth(headers: &HeaderMap, query: &AuthQuery, auth_token: &str) -> bool {
    // 1. Check Query parameter
    if let Some(ref q_token) = query.token {
        if q_token == auth_token {
            return true;
        }
    }

    // 2. Check Authorization Header
    if let Some(auth_val) = headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()) {
        if let Some(bearer) = auth_val.strip_prefix("Bearer ") {
            if bearer.trim() == auth_token {
                return true;
            }
        }
    }

    // 3. Check Cookie
    if let Some(cookie_hdr) = headers.get(header::COOKIE).and_then(|v| v.to_str().ok()) {
        for cookie in cookie_hdr.split(';') {
            let mut parts = cookie.splitn(2, '=');
            if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                if k.trim() == "atena_token" && v.trim() == auth_token {
                    return true;
                }
            }
        }
    }

    false
}

async fn http_invoke_handler(
    State(ctx): State<ServerContext>,
    headers: HeaderMap,
    Query(query): Query<AuthQuery>,
    axum::Json(payload): axum::Json<InvokeRequest>,
) -> Response {
    if !check_auth(&headers, &query, &ctx.auth_token) {
        return (StatusCode::UNAUTHORIZED, "Unauthorized access token").into_response();
    }

    let args = payload.args.unwrap_or(json!({}));
    match dispatcher::dispatch_invoke(&ctx.app_state, &payload.cmd, args).await {
        Ok(result) => axum::Json(json!({ "result": result })).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(json!({ "error": err })),
        )
            .into_response(),
    }
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(ctx): State<ServerContext>,
    headers: HeaderMap,
    Query(query): Query<AuthQuery>,
) -> Response {
    if !check_auth(&headers, &query, &ctx.auth_token) {
        return (StatusCode::UNAUTHORIZED, "Unauthorized access token").into_response();
    }

    ws.on_upgrade(move |socket| handle_socket(socket, ctx))
}

async fn handle_socket(socket: WebSocket, ctx: ServerContext) {
    let (mut sender, mut receiver) = socket.split();
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<Message>();

    // Forwarder task for outgoing WebSocket frames
    let forwarder = tokio::spawn(async move {
        while let Some(msg) = out_rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let app_state = ctx.app_state.clone();

    // Incoming messages processor
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                let parsed: Value = match serde_json::from_str(&text) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                let msg_type = parsed.get("type").and_then(|v| v.as_str()).unwrap_or_default();
                if msg_type == "ping" {
                    let _ = out_tx.send(Message::Text(json!({ "type": "pong" }).to_string()));
                    continue;
                }

                if msg_type == "invoke" {
                    let req_id = parsed.get("id").cloned().unwrap_or(json!(0));
                    let cmd = parsed.get("cmd").and_then(|v| v.as_str()).unwrap_or_default().to_string();
                    let args = parsed.get("args").cloned().unwrap_or(json!({}));

                    let out_tx_clone = out_tx.clone();
                    let app_state_clone = app_state.clone();

                    // Handle streaming commands vs unary commands
                    tokio::spawn(async move {
                        if cmd == "stream_chat" {
                            let channel_id = extract_channel_id(&args, "on_event");
                            let stream_res = handle_stream_chat(app_state_clone, args, channel_id, out_tx_clone.clone()).await;

                            let resp = match stream_res {
                                Ok(_) => json!({ "id": req_id, "type": "response", "result": null }),
                                Err(e) => json!({ "id": req_id, "type": "response", "error": e }),
                            };
                            let _ = out_tx_clone.send(Message::Text(resp.to_string()));
                        } else if cmd == "start_llama_server" || cmd == "start_mlx_server" || cmd == "start_ollama_server" {
                            let channel_id = extract_channel_id(&args, "channel");
                            let server_res = handle_server_start(app_state_clone, &cmd, args, channel_id, out_tx_clone.clone()).await;

                            let resp = match server_res {
                                Ok(_) => json!({ "id": req_id, "type": "response", "result": null }),
                                Err(e) => json!({ "id": req_id, "type": "response", "error": e }),
                            };
                            let _ = out_tx_clone.send(Message::Text(resp.to_string()));
                        } else {
                            match dispatcher::dispatch_invoke(&app_state_clone, &cmd, args).await {
                                Ok(val) => {
                                    let resp = json!({
                                        "id": req_id,
                                        "type": "response",
                                        "result": val
                                    });
                                    let _ = out_tx_clone.send(Message::Text(resp.to_string()));
                                }
                                Err(err) => {
                                    let resp = json!({
                                        "id": req_id,
                                        "type": "response",
                                        "error": err
                                    });
                                    let _ = out_tx_clone.send(Message::Text(resp.to_string()));
                                }
                            }
                        }
                    });
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    forwarder.abort();
}

fn extract_channel_id(args: &Value, key: &str) -> Option<u32> {
    if let Some(val) = args.get(key).and_then(|v| v.as_str()) {
        if let Some(id_str) = val.strip_prefix("__CHANNEL__:") {
            return id_str.parse().ok();
        }
    }
    let camel = match key {
        "on_event" => "onEvent",
        "on_progress" => "onProgress",
        _ => key,
    };
    if let Some(val) = args.get(camel).and_then(|v| v.as_str()) {
        if let Some(id_str) = val.strip_prefix("__CHANNEL__:") {
            return id_str.parse().ok();
        }
    }
    if let Some(obj) = args.as_object() {
        for val in obj.values() {
            if let Some(s) = val.as_str() {
                if let Some(id_str) = s.strip_prefix("__CHANNEL__:") {
                    return id_str.parse().ok();
                }
            }
        }
    }
    None
}

async fn handle_stream_chat(
    state: Arc<AppState>,
    args: Value,
    channel_id: Option<u32>,
    out_tx: mpsc::UnboundedSender<Message>,
) -> Result<(), String> {
    let req: StreamChatRequest = serde_json::from_value(args).map_err(|e| e.to_string())?;

    let tx_clone = out_tx.clone();
    let counter = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let counter_clone = counter.clone();

    let on_chunk = Arc::new(move |chunk: ChatChunkPayload| {
        if let Some(cid) = channel_id {
            let idx = counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let msg = json!({
                "type": "channel",
                "id": cid,
                "index": idx,
                "message": chunk,
            });
            let _ = tx_clone.send(Message::Text(msg.to_string()));
        }
    });

    let res = crate::execute_stream_chat_internal(&state, req, on_chunk).await;

    // Send channel end signal
    if let Some(cid) = channel_id {
        let idx = counter.load(std::sync::atomic::Ordering::SeqCst);
        let end_msg = json!({
            "type": "channel",
            "id": cid,
            "index": idx,
            "end": true,
        });
        let _ = out_tx.send(Message::Text(end_msg.to_string()));
    }

    res
}

async fn handle_server_start(
    state: Arc<AppState>,
    cmd: &str,
    args: Value,
    channel_id: Option<u32>,
    out_tx: mpsc::UnboundedSender<Message>,
) -> Result<(), String> {
    let tx_clone = out_tx.clone();
    let counter = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let counter_clone = counter.clone();

    let progress_cb = Arc::new(move |prog: ModelLoadProgress| {
        if let Some(cid) = channel_id {
            let idx = counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let msg = json!({
                "type": "channel",
                "id": cid,
                "index": idx,
                "message": prog,
            });
            let _ = tx_clone.send(Message::Text(msg.to_string()));
        }
    }) as Arc<dyn Fn(ModelLoadProgress) + Send + Sync>;

    let result = match cmd {
        "start_mlx_server" => {
            let model_path = args.get("modelPath").and_then(|v| v.as_str()).unwrap_or_default();
            let host = args.get("host").and_then(|v| v.as_str()).unwrap_or("127.0.0.1");
            let port = args.get("port").and_then(|v| v.as_u64()).unwrap_or(8080) as u16;
            state.backend_manager.start_mlx_server(model_path, host, port, Some(progress_cb)).await
        }
        "start_llama_server" => {
            let model_path = args.get("modelPath").and_then(|v| v.as_str()).unwrap_or_default();
            let host = args.get("host").and_then(|v| v.as_str()).unwrap_or("127.0.0.1");
            let port = args.get("port").and_then(|v| v.as_u64()).unwrap_or(8080) as u16;
            let context_length = args.get("contextLength").and_then(|v| v.as_u64()).map(|v| v as usize).unwrap_or(8192);
            let kv_cache_quant = args.get("kvCacheQuant").and_then(|v| v.as_str()).map(String::from);
            let flash_attn = args.get("flashAttn").and_then(|v| v.as_bool());
            let prompt_cache = args.get("promptCache").and_then(|v| v.as_bool());

            state.backend_manager.start_llama_server(
                model_path,
                host,
                port,
                context_length,
                kv_cache_quant,
                flash_attn,
                prompt_cache,
                Some(progress_cb),
            ).await
        }
        "start_ollama_server" => {
            state.backend_manager.start_ollama_serve(Some(progress_cb)).await
        }
        _ => Err(format!("Unknown server start command: {}", cmd)),
    };

    if let Some(cid) = channel_id {
        let idx = counter.load(std::sync::atomic::Ordering::SeqCst);
        let end_msg = json!({
            "type": "channel",
            "id": cid,
            "index": idx,
            "end": true,
        });
        let _ = out_tx.send(Message::Text(end_msg.to_string()));
    }

    result
}

async fn fallback_landing_page() -> Html<&'static str> {
    Html(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Atena Studio - Web Server</title>
    <style>
        body {
            background-color: #0c0e16;
            color: #f1f5f9;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            display: flex;
            align-items: center;
            justify-content: center;
            min-height: 100vh;
            margin: 0;
            padding: 20px;
        }
        .card {
            background-color: #121524;
            border: 1px solid #1e2438;
            border-radius: 16px;
            padding: 32px;
            max-width: 540px;
            text-align: center;
            box-shadow: 0 10px 30px rgba(0,0,0,0.5);
        }
        h1 { color: #6366f1; margin-top: 0; }
        code {
            background: #181c2e;
            padding: 4px 8px;
            border-radius: 6px;
            color: #38bdf8;
            font-family: monospace;
        }
        .status {
            display: inline-block;
            background: rgba(16, 185, 129, 0.15);
            color: #34d399;
            border: 1px solid rgba(16, 185, 129, 0.3);
            border-radius: 20px;
            padding: 4px 12px;
            font-size: 12px;
            font-weight: bold;
            margin-bottom: 16px;
        }
    </style>
</head>
<body>
    <div class="card">
        <div class="status">● BACKEND RUNNING</div>
        <h1>Atena Studio Web Server</h1>
        <p>The Rust backend and cognitive memory engine are running successfully.</p>
        <p>To access the complete web interface, please generate the frontend distribution assets by running:</p>
        <p><code>pnpm --prefix frontend run generate</code></p>
        <p style="margin-top: 24px; font-size: 13px; color: #64748b;">
            WebSocket IPC endpoint active at <code>/api/ipc/ws</code>
        </p>
    </div>
</body>
</html>"#)
}
