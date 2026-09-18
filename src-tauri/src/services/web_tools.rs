use std::time::Duration;
use serde::{Deserialize, Serialize};
use base64::Engine as _;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

pub struct WebTools;

impl WebTools {
    /// Performs a zero-dependency lightweight web search via multi-provider resilient engine (Bing, DuckDuckGo)
    pub async fn search(query: &str, max_results: usize) -> Result<Vec<WebSearchResult>, String> {
        let trimmed_query = query.trim();
        if trimmed_query.is_empty() {
            return Err("Search query cannot be empty".to_string());
        }

        let limit = if max_results == 0 { 5 } else { max_results.min(15) };

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(12))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        // 1. Primary engine: Bing search (fast, zero-API-key, rich organic results)
        if let Ok(bing_results) = Self::search_bing(&client, trimmed_query, limit).await {
            if !bing_results.is_empty() {
                return Ok(bing_results);
            }
        }

        // 2. Secondary engine: DuckDuckGo HTML / Lite
        if let Ok(ddg_results) = Self::search_duckduckgo(&client, trimmed_query, limit).await {
            if !ddg_results.is_empty() {
                return Ok(ddg_results);
            }
        }

        // 3. Tertiary engine: DuckDuckGo Instant Answer API
        if let Ok(ddg_api_results) = Self::search_duckduckgo_api(&client, trimmed_query, limit).await {
            if !ddg_api_results.is_empty() {
                return Ok(ddg_api_results);
            }
        }

        Ok(Vec::new())
    }

    /// Primary search implementation using Bing
    pub async fn search_bing(client: &reqwest::Client, query: &str, max_results: usize) -> Result<Vec<WebSearchResult>, String> {
        let url = format!("https://www.bing.com/search?q={}", urlencoding(query));
        let resp = client
            .get(&url)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "en-US,en;q=0.9,pt-BR;q=0.8,pt;q=0.7")
            .send()
            .await
            .map_err(|e| format!("Bing search request failed: {}", e))?;

        if !resp.status().is_success() {
            return Ok(Vec::new());
        }

        let html = resp.text().await.unwrap_or_default();
        Ok(Self::parse_bing_html(&html, max_results))
    }

    /// Secondary search implementation using DuckDuckGo Lite & HTML
    pub async fn search_duckduckgo(client: &reqwest::Client, query: &str, max_results: usize) -> Result<Vec<WebSearchResult>, String> {
        let response = client
            .post("https://lite.duckduckgo.com/lite/")
            .form(&[("q", query)])
            .send()
            .await;

        let html = match response {
            Ok(resp) => {
                if !resp.status().is_success() {
                    Self::fallback_search_html(client, query).await?
                } else {
                    resp.text().await.unwrap_or_default()
                }
            }
            Err(_) => Self::fallback_search_html(client, query).await?,
        };

        let mut results = Self::parse_duckduckgo_html(&html, max_results);
        if results.is_empty() {
            results = Self::parse_duckduckgo_lite(&html, max_results);
        }
        Ok(results)
    }

    /// Fallback search using DuckDuckGo Instant Answer API
    pub async fn search_duckduckgo_api(client: &reqwest::Client, query: &str, max_results: usize) -> Result<Vec<WebSearchResult>, String> {
        let url = format!("https://api.duckduckgo.com/?q={}&format=json&no_html=1", urlencoding(query));
        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("DuckDuckGo API request failed: {}", e))?;

        if !resp.status().is_success() {
            return Ok(Vec::new());
        }

        let json: serde_json::Value = resp.json().await.map_err(|e| format!("Failed to parse DDG API JSON: {}", e))?;
        let mut results = Vec::new();
        let limit = if max_results == 0 { 5 } else { max_results.min(15) };

        if let (Some(abs), Some(url)) = (
            json.get("Abstract").and_then(|v| v.as_str()),
            json.get("AbstractURL").and_then(|v| v.as_str()),
        ) {
            if !abs.trim().is_empty() && !url.trim().is_empty() {
                let heading = json.get("Heading").and_then(|v| v.as_str()).unwrap_or("DuckDuckGo Answer");
                results.push(WebSearchResult {
                    title: heading.to_string(),
                    url: url.to_string(),
                    snippet: abs.trim().to_string(),
                });
            }
        }

        if let Some(topics) = json.get("RelatedTopics").and_then(|v| v.as_array()) {
            for topic in topics {
                if results.len() >= limit {
                    break;
                }
                if let (Some(text), Some(first_url)) = (
                    topic.get("Text").and_then(|v| v.as_str()),
                    topic.get("FirstURL").and_then(|v| v.as_str()),
                ) {
                    if !text.trim().is_empty() && !first_url.trim().is_empty() {
                        results.push(WebSearchResult {
                            title: text.split(" - ").next().unwrap_or(text).to_string(),
                            url: first_url.to_string(),
                            snippet: text.to_string(),
                        });
                    }
                }
            }
        }

        Ok(results)
    }

    /// Parses search results from Bing HTML page
    pub fn parse_bing_html(html: &str, max_results: usize) -> Vec<WebSearchResult> {
        let mut results = Vec::new();
        let limit = if max_results == 0 { 5 } else { max_results.min(15) };

        let blocks: Vec<&str> = html.split("b_algo").collect();
        for block in blocks.iter().skip(1) {
            if results.len() >= limit {
                break;
            }

            // Extract H2 section
            let h2_content = if let Some(start) = block.find("<h2") {
                if let Some(tag_close) = block[start..].find('>') {
                    let inner_start = start + tag_close + 1;
                    if let Some(end) = block[inner_start..].find("</h2>") {
                        &block[inner_start..inner_start + end]
                    } else {
                        ""
                    }
                } else {
                    ""
                }
            } else {
                ""
            };

            // Extract href from h2 or block
            let raw_url = if !h2_content.is_empty() {
                extract_href(h2_content).unwrap_or_else(|| extract_first_href(block).unwrap_or_default())
            } else {
                extract_first_href(block).unwrap_or_default()
            };

            if raw_url.is_empty() || raw_url.starts_with("javascript:") {
                continue;
            }

            // Skip Bing ad click redirects
            if raw_url.contains("/aclick?") {
                continue;
            }

            let clean_url = unwrap_bing_url(&raw_url);

            // Extract Title from h2
            let raw_title = if !h2_content.is_empty() {
                h2_content.to_string()
            } else if let Some(title_pos) = block.find("<h2") {
                extract_text_between_tags(&block[title_pos..], ">", "</h2>").unwrap_or_default()
            } else {
                String::new()
            };
            let clean_title = decode_html_entities(&strip_html_tags(&raw_title)).trim().to_string();

            // Extract Snippet from <p> or <div class="b_caption">
            let raw_snippet = if let Some(p_start) = block.find("<p") {
                if let Some(tag_close) = block[p_start..].find('>') {
                    let inner_start = p_start + tag_close + 1;
                    if let Some(end) = block[inner_start..].find("</p>") {
                        block[inner_start..inner_start + end].to_string()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            } else if let Some(cap_pos) = block.find("b_caption") {
                extract_text_between_tags(&block[cap_pos..], ">", "</div>").unwrap_or_default()
            } else {
                String::new()
            };
            let clean_snippet = decode_html_entities(&strip_html_tags(&raw_snippet)).trim().to_string();

            if !clean_url.is_empty() && (!clean_title.is_empty() || !clean_snippet.is_empty()) {
                results.push(WebSearchResult {
                    title: if clean_title.is_empty() { clean_url.clone() } else { clean_title },
                    url: clean_url,
                    snippet: clean_snippet,
                });
            }
        }

        results
    }

    async fn fallback_search_html(client: &reqwest::Client, query: &str) -> Result<String, String> {
        let url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding(query));
        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Web search request failed: {}", e))?;

        resp.text()
            .await
            .map_err(|e| format!("Failed to read search response body: {}", e))
    }

    /// Parses search results from DuckDuckGo HTML page
    pub fn parse_duckduckgo_html(html: &str, max_results: usize) -> Vec<WebSearchResult> {
        let mut results = Vec::new();
        let limit = if max_results == 0 { 5 } else { max_results.min(15) };

        // Split by result block in DuckDuckGo HTML
        let blocks: Vec<&str> = html.split("class=\"result ").collect();
        for block in blocks.iter().skip(1) {
            if results.len() >= limit {
                break;
            }

            // Extract URL from <a class="result__url" href="..."> or <a class="result__snippet" ...>
            let url = if let Some(href_start) = block.find("class=\"result__url\"") {
                let slice = &block[href_start..];
                extract_href(slice).unwrap_or_default()
            } else if let Some(href_start) = block.find("class=\"result__snippet\"") {
                let slice = &block[href_start..];
                extract_href(slice).unwrap_or_default()
            } else {
                extract_first_href(block).unwrap_or_default()
            };

            // Clean DuckDuckGo redirect URL if necessary (/l/?uddg=...)
            let clean_url = clean_ddg_url(&url);

            // Extract Title from class="result__title"
            let title = if let Some(title_pos) = block.find("result__title") {
                let slice = &block[title_pos..];
                extract_text_between_tags(slice, ">", "</a>").unwrap_or_default()
            } else {
                String::new()
            };

            // Extract Snippet from class="result__snippet"
            let snippet = if let Some(snip_pos) = block.find("result__snippet") {
                let slice = &block[snip_pos..];
                extract_text_between_tags(slice, ">", "</a>")
                    .or_else(|| extract_text_between_tags(slice, ">", "</td>"))
                    .unwrap_or_default()
            } else {
                String::new()
            };

            let clean_title = decode_html_entities(&strip_html_tags(&title));
            let clean_snippet = decode_html_entities(&strip_html_tags(&snippet));

            if !clean_url.is_empty() && (!clean_title.is_empty() || !clean_snippet.is_empty()) {
                results.push(WebSearchResult {
                    title: if clean_title.is_empty() { clean_url.clone() } else { clean_title },
                    url: clean_url,
                    snippet: clean_snippet,
                });
            }
        }

        results
    }

    /// Parses search results from DuckDuckGo Lite page
    pub fn parse_duckduckgo_lite(html: &str, max_results: usize) -> Vec<WebSearchResult> {
        let mut results = Vec::new();
        let limit = if max_results == 0 { 5 } else { max_results.min(15) };

        // DuckDuckGo Lite uses table rows with class="result-link" and class="result-snippet"
        let rows: Vec<&str> = html.split("class=\"result-link\"").collect();
        for row in rows.iter().skip(1) {
            if results.len() >= limit {
                break;
            }

            let url = extract_href(row).unwrap_or_default();
            let clean_url = clean_ddg_url(&url);

            let title = extract_text_between_tags(row, ">", "</a>").unwrap_or_default();

            // Snippet is usually in the next snippet container or td
            let snippet = if let Some(snip_pos) = row.find("result-snippet") {
                let slice = &row[snip_pos..];
                extract_text_between_tags(slice, ">", "</td>").unwrap_or_default()
            } else {
                String::new()
            };

            let clean_title = decode_html_entities(&strip_html_tags(&title));
            let clean_snippet = decode_html_entities(&strip_html_tags(&snippet));

            if !clean_url.is_empty() {
                results.push(WebSearchResult {
                    title: if clean_title.is_empty() { clean_url.clone() } else { clean_title },
                    url: clean_url,
                    snippet: clean_snippet,
                });
            }
        }

        results
    }

    /// Fetches a webpage and converts its content to clean, readable Markdown
    pub async fn fetch_webpage(url: &str, max_characters: Option<usize>) -> Result<String, String> {
        let trimmed_url = url.trim();
        if !trimmed_url.starts_with("http://") && !trimmed_url.starts_with("https://") {
            return Err("URL must start with http:// or https://".to_string());
        }

        let max_chars = max_characters.unwrap_or(8000).clamp(500, 32000);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
            .redirect(reqwest::redirect::Policy::limited(5))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let response = client
            .get(trimmed_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch URL '{}': {}", trimmed_url, e))?;

        let status = response.status();
        if !status.is_success() {
            return Err(format!("HTTP error {} when requesting '{}'", status, trimmed_url));
        }

        let raw_html = response
            .text()
            .await
            .map_err(|e| format!("Failed to read webpage content: {}", e))?;

        let markdown = Self::convert_html_to_markdown(&raw_html);

        if markdown.len() > max_chars {
            let mut truncated = markdown.chars().take(max_chars).collect::<String>();
            truncated.push_str("\n\n... [Content truncated for length] ...");
            Ok(truncated)
        } else {
            Ok(markdown)
        }
    }

    /// Converts raw HTML to clean markdown text
    pub fn convert_html_to_markdown(html: &str) -> String {
        // 1. Remove scripts, styles, heads, svgs, comments
        let cleaned = remove_tag_blocks(html, "<script", "</script>");
        let cleaned = remove_tag_blocks(&cleaned, "<style", "</style>");
        let cleaned = remove_tag_blocks(&cleaned, "<svg", "</svg>");
        let cleaned = remove_tag_blocks(&cleaned, "<noscript", "</noscript>");
        let cleaned = remove_tag_blocks(&cleaned, "<nav", "</nav>");
        let cleaned = remove_tag_blocks(&cleaned, "<footer", "</footer>");
        let cleaned = remove_comments(&cleaned);

        // 2. Extract page title
        let page_title = extract_tag_content(&cleaned, "<title>", "</title>")
            .map(|t| decode_html_entities(&strip_html_tags(&t)))
            .unwrap_or_default();

        // 3. Process structural elements
        let mut text = cleaned;
        // Headings
        for i in 1..=6 {
            let open_tag = format!("<h{}", i);
            let close_tag = format!("</h{}>", i);
            let prefix = format!("\n\n{} ", "#".repeat(i));
            text = replace_tag_with_prefix(&text, &open_tag, &close_tag, &prefix, "\n\n");
        }

        // Paragraphs & Line breaks
        text = text.replace("<br>", "\n").replace("<br/>", "\n").replace("<br />", "\n");
        text = replace_tag_with_prefix(&text, "<p", "</p>", "\n\n", "\n\n");
        text = replace_tag_with_prefix(&text, "<li", "</li>", "\n* ", "");
        text = replace_tag_with_prefix(&text, "<tr", "</tr>", "\n", "");
        text = replace_tag_with_prefix(&text, "<td", "</td>", " | ", "");
        text = replace_tag_with_prefix(&text, "<th", "</th>", " | ", "");
        text = replace_tag_with_prefix(&text, "<pre", "</pre>", "\n```\n", "\n```\n");
        text = replace_tag_with_prefix(&text, "<code", "</code>", "`", "`");

        // 4. Strip remaining HTML tags
        let raw_text = strip_html_tags(&text);
        let decoded = decode_html_entities(&raw_text);

        // 5. Clean up extra blank lines and whitespaces
        let mut clean_lines = Vec::new();
        let mut prev_empty = false;
        for line in decoded.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                if !prev_empty {
                    clean_lines.push("");
                    prev_empty = true;
                }
            } else {
                clean_lines.push(trimmed);
                prev_empty = false;
            }
        }

        let body = clean_lines.join("\n");
        if !page_title.is_empty() {
            format!("# {}\n\n{}", page_title, body.trim())
        } else {
            body.trim().to_string()
        }
    }
}

fn urlencoding(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len() * 3);
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            b' ' => encoded.push('+'),
            _ => {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}

fn extract_href(slice: &str) -> Option<String> {
    let href_needle = "href=\"";
    let start = slice.find(href_needle)? + href_needle.len();
    let end = slice[start..].find('"')? + start;
    Some(slice[start..end].to_string())
}

fn extract_first_href(slice: &str) -> Option<String> {
    extract_href(slice)
}

fn clean_ddg_url(url: &str) -> String {
    if url.contains("uddg=") {
        if let Some(pos) = url.find("uddg=") {
            let encoded = &url[pos + 5..];
            let end = encoded.find('&').unwrap_or(encoded.len());
            return urldecode(&encoded[..end]);
        }
    }
    url.to_string()
}

pub fn unwrap_bing_url(raw_url: &str) -> String {
    let clean = raw_url.replace("&amp;", "&");
    if let Some(pos) = clean.find("/ck/a?") {
        let query_str = &clean[pos + 6..];
        for part in query_str.split('&') {
            if let Some(val) = part.strip_prefix("u=") {
                let unencoded = urldecode(val);
                let b64_payload = if unencoded.len() > 2 { &unencoded[2..] } else { &unencoded };
                let pad_len = (4 - (b64_payload.len() % 4)) % 4;
                let mut padded = b64_payload.to_string();
                for _ in 0..pad_len {
                    padded.push('=');
                }

                let decoded = base64::engine::general_purpose::URL_SAFE
                    .decode(&padded)
                    .or_else(|_| base64::engine::general_purpose::STANDARD.decode(&padded));

                if let Ok(bytes) = decoded {
                    if let Ok(target_url) = String::from_utf8(bytes) {
                        if target_url.starts_with("http://") || target_url.starts_with("https://") {
                            return target_url;
                        }
                    }
                }
            }
        }
    }
    clean
}

fn urldecode(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars();
    while let Some(ch) = chars.next() {
        if ch == '%' {
            let h1 = chars.next().unwrap_or('0');
            let h2 = chars.next().unwrap_or('0');
            if let Ok(byte) = u8::from_str_radix(&format!("{}{}", h1, h2), 16) {
                result.push(byte as char);
            }
        } else if ch == '+' {
            result.push(' ');
        } else {
            result.push(ch);
        }
    }
    result
}

fn extract_text_between_tags(slice: &str, start_delim: &str, end_delim: &str) -> Option<String> {
    let start_pos = slice.find(start_delim)? + start_delim.len();
    let end_pos = slice[start_pos..].find(end_delim)? + start_pos;
    Some(slice[start_pos..end_pos].to_string())
}

fn extract_tag_content(html: &str, open_tag: &str, close_tag: &str) -> Option<String> {
    let start = html.find(open_tag)? + open_tag.len();
    let end = html[start..].find(close_tag)? + start;
    Some(html[start..end].to_string())
}

fn remove_tag_blocks(html: &str, open_prefix: &str, close_tag: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut cursor = 0;

    while let Some(start) = html[cursor..].find(open_prefix) {
        let abs_start = cursor + start;
        result.push_str(&html[cursor..abs_start]);

        if let Some(end) = html[abs_start..].find(close_tag) {
            cursor = abs_start + end + close_tag.len();
        } else {
            cursor = html.len();
            break;
        }
    }

    if cursor < html.len() {
        result.push_str(&html[cursor..]);
    }
    result
}

fn remove_comments(html: &str) -> String {
    remove_tag_blocks(html, "<!--", "-->")
}

fn replace_tag_with_prefix(html: &str, open_tag_prefix: &str, close_tag: &str, prefix: &str, suffix: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut cursor = 0;

    while let Some(start) = html[cursor..].find(open_tag_prefix) {
        let abs_start = cursor + start;
        result.push_str(&html[cursor..abs_start]);

        // Find end of the opening tag '>'
        if let Some(tag_close) = html[abs_start..].find('>') {
            let content_start = abs_start + tag_close + 1;
            result.push_str(prefix);

            if let Some(close_pos) = html[content_start..].find(close_tag) {
                let content_end = content_start + close_pos;
                result.push_str(&html[content_start..content_end]);
                result.push_str(suffix);
                cursor = content_end + close_tag.len();
            } else {
                cursor = content_start;
            }
        } else {
            cursor = abs_start + open_tag_prefix.len();
        }
    }

    if cursor < html.len() {
        result.push_str(&html[cursor..]);
    }
    result
}

fn strip_html_tags(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut in_tag = false;

    for ch in input.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            output.push(ch);
        }
    }
    output
}

fn decode_html_entities(input: &str) -> String {
    input
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
        .replace("&mdash;", "—")
        .replace("&ndash;", "–")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_to_markdown_conversion() {
        let sample = r#"
            <html>
            <head><title>Test Page Title</title></head>
            <body>
                <script>console.log('strip me');</script>
                <h1>Main Heading</h1>
                <p>This is a <b>paragraph</b> with <a href="https://example.com">a link</a>.</p>
                <ul>
                    <li>First bullet</li>
                    <li>Second bullet</li>
                </ul>
            </body>
            </html>
        "#;

        let md = WebTools::convert_html_to_markdown(sample);
        assert!(md.contains("# Test Page Title"));
        assert!(md.contains("# Main Heading"));
        assert!(md.contains("This is a paragraph with a link."));
        assert!(md.contains("* First bullet"));
        assert!(!md.contains("strip me"));
    }

    #[test]
    fn test_parse_duckduckgo_html() {
        let sample = r#"
            <div class="result ">
                <a class="result__url" href="https://duckduckgo.com/l/?uddg=https%3A%2F%2Fexample.com%2Fpage&rut=1">example.com</a>
                <h2 class="result__title"><a href="...">Example Title</a></h2>
                <a class="result__snippet">This is an example snippet from search results.</a>
            </div>
        "#;

        let results = WebTools::parse_duckduckgo_html(sample, 5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].url, "https://example.com/page");
        assert_eq!(results[0].title, "Example Title");
        assert_eq!(results[0].snippet, "This is an example snippet from search results.");
    }

    #[test]
    fn test_unwrap_bing_url() {
        let raw = "https://www.bing.com/ck/a?!&&p=123&u=a1aHR0cHM6Ly9ydXN0LWxhbmcub3JnLw&ntb=1";
        assert_eq!(unwrap_bing_url(raw), "https://rust-lang.org/");

        let direct = "https://example.com/direct";
        assert_eq!(unwrap_bing_url(direct), "https://example.com/direct");
    }

    #[test]
    fn test_parse_bing_html() {
        let sample = r#"
            <li class="b_algo">
                <h2><a href="https://www.bing.com/ck/a?!&&p=1&u=a1aHR0cHM6Ly9ydXN0LWxhbmcub3JnLw&ntb=1"><strong>Rust</strong> Programming Language</a></h2>
                <div class="b_caption"><p>Rust is blazingly fast and memory-efficient.</p></div>
            </li>
        "#;
        let results = WebTools::parse_bing_html(sample, 5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Rust Programming Language");
        assert_eq!(results[0].url, "https://rust-lang.org/");
        assert_eq!(results[0].snippet, "Rust is blazingly fast and memory-efficient.");
    }

    #[tokio::test]
    async fn test_live_web_search() {
        let results = WebTools::search("rust programming", 3).await;
        assert!(results.is_ok(), "Search should not error");
        let list = results.unwrap();
        assert!(!list.is_empty(), "Live search must return results and not empty []");
        assert!(list[0].url.starts_with("http"), "URL must be valid http/https");
        assert!(!list[0].title.is_empty(), "Title must not be empty");
    }
}
