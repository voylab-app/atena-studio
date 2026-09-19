use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use tokio::sync::Semaphore;
use tokio::time::sleep;

use crate::services::web_tools::WebSearchResult;

static APP_HANDLE: Mutex<Option<AppHandle>> = Mutex::new(None);
static WINDOW_COUNTER: AtomicU64 = AtomicU64::new(1);
// Limit concurrent headless scraper windows to max 2 to prevent RAM surges
static CONCURRENCY_SEMAPHORE: Semaphore = Semaphore::const_new(2);

/// RAII Guard ensuring the headless WebviewWindow is unconditionally destroyed on drop
pub struct HeadlessWindowGuard {
    window: Option<WebviewWindow>,
}

impl HeadlessWindowGuard {
    pub fn new(window: WebviewWindow) -> Self {
        Self {
            window: Some(window),
        }
    }

    pub fn window(&self) -> &WebviewWindow {
        self.window.as_ref().expect("Window must be present")
    }
}

impl Drop for HeadlessWindowGuard {
    fn drop(&mut self) {
        if let Some(w) = self.window.take() {
            let label = w.label().to_string();
            log::debug!("🧹 RAII: Closing and destroying headless scraper window '{}'", label);
            if let Some(app) = BrowserEngine::get_handle() {
                let _ = app.run_on_main_thread(move || {
                    let _ = w.destroy();
                });
            } else {
                let _ = w.destroy();
            }
        }
    }
}

pub struct BrowserEngine;

impl BrowserEngine {
    /// Initializes the application handle required to spawn offscreen webviews
    pub fn init_handle(app: AppHandle) {
        if let Ok(mut lock) = APP_HANDLE.lock() {
            *lock = Some(app);
        }
    }

    pub fn get_handle() -> Option<AppHandle> {
        APP_HANDLE.lock().ok().and_then(|lock| lock.clone())
    }

    /// Fetches a web page using a headless WebKit/WebView2 window, executing JavaScript and rendering SPAs
    pub async fn fetch_page_rendered(
        url: &str,
        wait_ms: Option<u64>,
        max_chars: Option<usize>,
    ) -> Result<String, String> {
        let app = Self::get_handle().ok_or_else(|| "Tauri AppHandle not initialized for BrowserEngine".to_string())?;

        let _permit = CONCURRENCY_SEMAPHORE
            .acquire()
            .await
            .map_err(|e| format!("Failed to acquire headless browser semaphore: {}", e))?;

        let window_label = format!("headless-scraper-{}", WINDOW_COUNTER.fetch_add(1, Ordering::Relaxed));
        let target_url = url.trim();
        let parsed_url = target_url.parse().map_err(|e| format!("Invalid URL '{}': {}", target_url, e))?;

        log::info!("🌐 Launching headless browser for: {}", target_url);

        // Spawn window on main thread via run_on_main_thread or direct async builder
        let (create_tx, create_rx) = tokio::sync::oneshot::channel::<Result<WebviewWindow, String>>();
        let app_clone = app.clone();
        let label_clone = window_label.clone();

        app.run_on_main_thread(move || {
            let mut builder = WebviewWindowBuilder::new(&app_clone, &label_clone, WebviewUrl::External(parsed_url))
                .visible(false)
                .focused(false)
                .decorations(false)
                .skip_taskbar(true)
                .inner_size(1.0, 1.0)
                .position(-10000.0, -10000.0)
                .incognito(true)
                .title("Atena Headless Browser");

            #[cfg(target_os = "macos")]
            {
                builder = builder.tabbing_identifier("atena-headless");
            }

            let res = builder.build().map_err(|e| format!("Failed to build headless webview: {}", e));
            let _ = create_tx.send(res);
        }).map_err(|e| format!("Failed to dispatch window creation to main thread: {}", e))?;

        let window = create_rx.await.map_err(|e| format!("Window creation channel dropped: {}", e))??;
        let guard = HeadlessWindowGuard::new(window);

        // Strict timeout on DOM stabilization (default 4.5s, clamped between 1.5s and 8.0s)
        let settle_delay = wait_ms.unwrap_or(3500).clamp(1500, 8000);
        sleep(Duration::from_millis(settle_delay)).await;

        let script = r#"
            (() => {
                try {
                    const clone = document.body ? document.body.cloneNode(true) : document.documentElement.cloneNode(true);
                    const toRemove = clone.querySelectorAll('script, style, svg, noscript, nav, footer, iframe, header, [aria-hidden="true"]');
                    toRemove.forEach(el => el.remove());
                    const title = document.title || '';
                    const text = clone.innerText || clone.textContent || '';
                    return JSON.stringify({
                        title: title.trim(),
                        content: text.trim()
                    });
                } catch(e) {
                    return JSON.stringify({ error: e.toString() });
                }
            })()
        "#;

        let (eval_tx, eval_rx) = tokio::sync::oneshot::channel::<String>();
        let eval_tx_mutex = std::sync::Mutex::new(Some(eval_tx));

        guard.window().eval_with_callback(script, move |result| {
            if let Ok(mut lock) = eval_tx_mutex.lock() {
                if let Some(sender) = lock.take() {
                    let _ = sender.send(result);
                }
            }
        }).map_err(|e| format!("Failed to evaluate extraction script in headless window: {}", e))?;

        let eval_timeout = Duration::from_secs(5);
        let raw_json = tokio::time::timeout(eval_timeout, eval_rx)
            .await
            .map_err(|_| "Timeout waiting for DOM extraction evaluation from headless webview".to_string())?
            .map_err(|e| format!("Evaluation channel closed unexpectedly: {}", e))?;

        // Parse result JSON
        let parsed: serde_json::Value = serde_json::from_str(&raw_json)
            .or_else(|_| {
                // If eval_with_callback serialized a stringified JSON string
                if let Ok(unquoted) = serde_json::from_str::<String>(&raw_json) {
                    serde_json::from_str(&unquoted)
                } else {
                    Err(serde_json::Error::io(std::io::ErrorKind::InvalidData.into()))
                }
            })
            .unwrap_or_default();

        let title = parsed.get("title").and_then(|v| v.as_str()).unwrap_or("");
        let content = parsed.get("content").and_then(|v| v.as_str()).unwrap_or("");

        if content.trim().is_empty() {
            return Err(format!("Headless browser extracted empty body from '{}'", target_url));
        }

        let max_len = max_chars.unwrap_or(8000).clamp(500, 32000);
        let mut markdown = String::new();
        if !title.is_empty() {
            markdown.push_str(&format!("# {}\n\n", title));
        }
        markdown.push_str(content);

        if markdown.len() > max_len {
            let mut truncated = markdown.chars().take(max_len).collect::<String>();
            truncated.push_str("\n\n... [Content truncated for length] ...");
            Ok(truncated)
        } else {
            Ok(markdown)
        }
    }

    /// Performs search by opening search engine in a headless webview and extracting rendered result cards
    pub async fn search_rendered(query: &str, max_results: usize) -> Result<Vec<WebSearchResult>, String> {
        let app = Self::get_handle().ok_or_else(|| "Tauri AppHandle not initialized for BrowserEngine".to_string())?;

        let _permit = CONCURRENCY_SEMAPHORE
            .acquire()
            .await
            .map_err(|e| format!("Failed to acquire headless browser semaphore: {}", e))?;

        let limit = if max_results == 0 { 5 } else { max_results.min(15) };
        let window_label = format!("headless-search-{}", WINDOW_COUNTER.fetch_add(1, Ordering::Relaxed));
        
        let search_url = format!("https://duckduckgo.com/?q={}", crate::services::web_tools::urlencoding(query));
        let parsed_url = search_url.parse().map_err(|e| format!("Invalid search URL: {}", e))?;

        log::info!("🔍 Running headless browser search for query: {}", query);

        let (create_tx, create_rx) = tokio::sync::oneshot::channel::<Result<WebviewWindow, String>>();
        let app_clone = app.clone();
        let label_clone = window_label.clone();

        app.run_on_main_thread(move || {
            let mut builder = WebviewWindowBuilder::new(&app_clone, &label_clone, WebviewUrl::External(parsed_url))
                .visible(false)
                .focused(false)
                .decorations(false)
                .skip_taskbar(true)
                .inner_size(1.0, 1.0)
                .position(-10000.0, -10000.0)
                .incognito(true)
                .title("Atena Headless Browser");

            #[cfg(target_os = "macos")]
            {
                builder = builder.tabbing_identifier("atena-headless");
            }

            let res = builder.build().map_err(|e| format!("Failed to build search webview: {}", e));
            let _ = create_tx.send(res);
        }).map_err(|e| format!("Failed to dispatch search window creation to main thread: {}", e))?;

        let window = create_rx.await.map_err(|e| format!("Window creation channel dropped: {}", e))??;
        let guard = HeadlessWindowGuard::new(window);

        // Wait 3.5 seconds for DuckDuckGo client JS rendering and anomaly bypass
        sleep(Duration::from_millis(3500)).await;

        let script = r#"
            (() => {
                try {
                    const results = [];
                    // 1. DuckDuckGo modern DOM results
                    const items = document.querySelectorAll('article[data-testid="result"], li[data-layout="organic"], div[data-testid="result"], ol.react-results--main li');
                    for (const item of items) {
                        // Skip sponsored and ad cards
                        if (item.classList.contains('result--ad') || item.querySelector('[data-testid="badge-ad"], [data-testid="result-ad"]')) {
                            continue;
                        }

                        const link = item.querySelector('a[data-testid="result-title-a"], h2 a, a[href^="http"]');
                        const snippet = item.querySelector('[data-result="snippet"], .result__snippet, [data-testid="result-snippet"]');
                        if (link && link.href && !link.href.includes('duckduckgo.com/y.js') && !link.href.includes('/aclick?')) {
                            const title = (link.innerText || link.textContent || '').trim();
                            const snipText = snippet ? (snippet.innerText || snippet.textContent || '').trim() : '';
                            if (title.length > 0) {
                                results.push({
                                    title: title,
                                    url: link.href,
                                    snippet: snipText
                                });
                            }
                        }
                    }
                    if (results.length > 0) return JSON.stringify(results);

                    // 2. Generic organic links from headings
                    const anchors = document.querySelectorAll('h2 a, h3 a');
                    for (const a of anchors) {
                        if (a.href && a.href.startsWith('http') && !a.href.includes('duckduckgo.com')) {
                            const t = (a.innerText || a.textContent || '').trim();
                            if (t.length > 3) {
                                results.push({
                                    title: t,
                                    url: a.href,
                                    snippet: ''
                                });
                            }
                        }
                    }
                    return JSON.stringify(results);
                } catch(e) {
                    return JSON.stringify([]);
                }
            })()
        "#;

        let (eval_tx, eval_rx) = tokio::sync::oneshot::channel::<String>();
        let eval_tx_mutex = std::sync::Mutex::new(Some(eval_tx));

        guard.window().eval_with_callback(script, move |result| {
            if let Ok(mut lock) = eval_tx_mutex.lock() {
                if let Some(sender) = lock.take() {
                    let _ = sender.send(result);
                }
            }
        }).map_err(|e| format!("Failed to evaluate search script: {}", e))?;

        let eval_timeout = Duration::from_secs(5);
        let raw_json = tokio::time::timeout(eval_timeout, eval_rx)
            .await
            .map_err(|_| "Timeout waiting for headless search results".to_string())?
            .map_err(|e| format!("Search evaluation channel closed: {}", e))?;

        let parsed: Vec<WebSearchResult> = serde_json::from_str(&raw_json)
            .or_else(|_| {
                if let Ok(unquoted) = serde_json::from_str::<String>(&raw_json) {
                    serde_json::from_str(&unquoted)
                } else {
                    Err(serde_json::Error::io(std::io::ErrorKind::InvalidData.into()))
                }
            })
            .unwrap_or_default();

        let mut filtered = Vec::new();
        for mut r in parsed {
            r.url = crate::services::web_tools::clean_ddg_url(&r.url);
            r.title = r.title.split_whitespace().collect::<Vec<_>>().join(" ");
            r.snippet = r.snippet.split_whitespace().collect::<Vec<_>>().join(" ");

            if !r.url.is_empty() && !r.url.contains("duckduckgo.com") && !r.title.is_empty() {
                filtered.push(r);
            }
            if filtered.len() >= limit {
                break;
            }
        }

        Ok(filtered)
    }
}
