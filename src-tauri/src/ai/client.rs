//! HTTP client for OpenAI-compatible APIs (Groq, OpenAI, OpenCode Go, custom).
//!
//! All supported providers expose a `/chat/completions` endpoint with Bearer auth.
//! Response parsing tries 4 fallback paths because some providers (OpenCode Go)
//! use slightly different JSON structures (e.g., empty `content` + `reasoning_content`).
//!
//! `list_models()` calls `GET /models` to auto-detect available models.
//! This works with Groq and OpenAI. OpenCode Go does not support it.

use reqwest::Client;
use serde::{Deserialize, Serialize};

const DEFAULT_GROQ_ENDPOINT: &str = "https://api.groq.com/openai/v1";
const DEFAULT_GROQ_MODEL: &str = "llama-3.3-70b-versatile";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub provider: String,
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    pub enabled: bool,
}

impl Default for AiConfig {
    fn default() -> Self {
        AiConfig {
            provider: "groq".to_string(),
            endpoint: DEFAULT_GROQ_ENDPOINT.to_string(),
            api_key: String::new(),
            model: DEFAULT_GROQ_MODEL.to_string(),
            enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
    stream: bool,
}

/// Sends a chat completion request and extracts the response text.
///
/// Parsing tries these fallback paths (in order):
/// 1. `choices[0].message.content` (OpenAI standard)
/// 2. `choices[0].message.reasoning_content` (DeepSeek models)
/// 3. `choices[0].text` (legacy format)
/// 4. Root-level `content` / `response` (alternative formats)
pub async fn chat(
    messages: Vec<ChatMessage>,
    config: &AiConfig,
    max_tokens: u32,
) -> Result<String, String> {
    let client = Client::new();

    let endpoint = if config.endpoint.is_empty() {
        DEFAULT_GROQ_ENDPOINT.to_string()
    } else {
        config.endpoint.clone()
    };

    let model = if config.model.is_empty() {
        DEFAULT_GROQ_MODEL.to_string()
    } else {
        config.model.clone()
    };

    let url = format!("{}/chat/completions", endpoint.trim_end_matches('/'));

    let req = ChatRequest {
        model,
        messages,
        temperature: 0.7,
        max_tokens,
        stream: false,
    };

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("x-api-key", &config.api_key)
        .json(&req)
        .send()
        .await
        .map_err(|e| format!("API request to {} failed: {}", url, e))?;

    let status = resp.status();
    let raw = resp.text().await.unwrap_or_default();

    if !status.is_success() {
        return Err(format!(
            "HTTP {} from {}\nResponse: {}",
            status.as_u16(),
            url,
            &raw[..raw.len().min(500)]
        ));
    }

let val: serde_json::Value = serde_json::from_str(&raw).map_err(|e| {
        format!(
            "Not JSON from {}. Raw: {}",
            url,
            &raw[..raw.len().min(300)]
        )
    })?;

    let choices = val["choices"].as_array();

    let content = choices
        .and_then(|c| c.first())
        .and_then(|c| {
            let c = &c["message"];
            let content = c["content"].as_str().filter(|s| !s.is_empty());
            let reasoning = c["reasoning_content"].as_str().filter(|s| !s.is_empty());
            content.or(reasoning).or_else(|| c["text"].as_str())
        })
        .or_else(|| val["content"].as_str())
        .or_else(|| val["response"].as_str())
        .unwrap_or("")
        .to_string();

    if content.is_empty() {
        let snippet = if let Some(c) = choices {
            serde_json::to_string_pretty(c).unwrap_or_default()
        } else {
            "choices not an array".to_string()
        };
        return Err(format!(
            "Empty content.\nEndpoint: {}\nChoices: {}",
            url,
            &snippet[..snippet.len().min(500)]
        ));
    }

    Ok(content)
}

/// Streams a chat completion via Server-Sent Events (SSE).
/// Parses `data: ` lines from the event stream, extracting `choices[0].delta.content`.
/// Calls `on_chunk` for each text delta, returns the full concatenated response.
/// Used by `evaluate_feynman_stream` to emit `ai-chunk` Tauri events.
pub async fn stream_chat(
    messages: Vec<ChatMessage>,
    config: &AiConfig,
    max_tokens: u32,
    mut on_chunk: impl FnMut(&str),
) -> Result<String, String> {
    let client = Client::new();

    let endpoint = if config.endpoint.is_empty() {
        DEFAULT_GROQ_ENDPOINT.to_string()
    } else {
        config.endpoint.clone()
    };

    let model = if config.model.is_empty() {
        DEFAULT_GROQ_MODEL.to_string()
    } else {
        config.model.clone()
    };

    let url = format!("{}/chat/completions", endpoint.trim_end_matches('/'));

    let req = ChatRequest {
        model, messages, temperature: 0.7, max_tokens, stream: true,
    };

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("x-api-key", &config.api_key)
        .json(&req)
        .send()
        .await
        .map_err(|e| format!("Stream request to {} failed: {}", url, e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let raw = resp.text().await.unwrap_or_default();
        return Err(format!("HTTP {}: {}", status.as_u16(), &raw[..raw.len().min(300)]));
    }

    use futures_util::StreamExt;
    let mut full = String::new();
    let mut stream = resp.bytes_stream();
    let mut buf = String::new();

    while let Some(chunk) = stream.next().await {
        let bytes = chunk.map_err(|e| format!("Stream error: {}", e))?;
        buf.push_str(&String::from_utf8_lossy(&bytes));

        while let Some(nl) = buf.find('\n') {
            let line = buf[..nl].trim().to_string();
            buf = buf[nl + 1..].to_string();

            if line.is_empty() || line.starts_with(':') { continue; }
            if let Some(data) = line.strip_prefix("data: ") {
                if data == "[DONE]" { continue; }
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                    if let Some(t) = val["choices"][0]["delta"]["content"].as_str() {
                        full.push_str(t);
                        on_chunk(t);
                    }
                }
            }
        }
    }

    Ok(full)
}

/// Fetches available chat models from the provider's /models endpoint.
/// Filters out non-chat models (whisper, tts, embed, guard, realtime, translate).
/// Used to populate the dynamic model dropdown in Settings → AI.
pub async fn list_models(config: &AiConfig) -> Result<Vec<ModelInfo>, String> {
    let client = Client::new();
    let endpoint = if config.endpoint.is_empty() {
        DEFAULT_GROQ_ENDPOINT.to_string()
    } else {
        config.endpoint.clone()
    };

    let url = format!("{}/models", endpoint.trim_end_matches('/'));
    let resp = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("x-api-key", &config.api_key)
        .send()
        .await
        .map_err(|e| format!("Cannot fetch models from {}: {}", url, e))?;

    let status = resp.status();
    let raw = resp.text().await.unwrap_or_default();

    if !status.is_success() {
        return Err(format!("HTTP {} from {}. {}", status.as_u16(), url, &raw[..raw.len().min(200)]));
    }

    let val: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| format!("Bad JSON from {}: {}", url, e))?;

    let mut models = Vec::new();
    let data = val["data"].as_array().or_else(|| val["models"].as_array());

    if let Some(items) = data {
        for item in items {
            let id = item["id"].as_str().unwrap_or("").to_string();
            let name = item["id"].as_str().unwrap_or("").to_string();

            if id.is_empty() { continue; }
            if id.contains("whisper") || id.contains("tts") || id.contains("embed") { continue; }
            if id.contains("guard") || id.contains("moderat") || id.contains("safeguard") { continue; }
            if id.contains("realtime") || id.contains("translat") { continue; }
            if id.contains("orpheus") { continue; }

            models.push(ModelInfo { id, name });
        }
    }

    if models.is_empty() {
        return Err(format!("No chat models found. Raw keys: {:?}",
            val.as_object().map(|o| o.keys().collect::<Vec<_>>()).unwrap_or_default()));
    }

    Ok(models)
}
