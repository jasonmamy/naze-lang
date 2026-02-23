//! AI prompt provider adapters for server-side execution.
//! Used by both `dev.rs` (dev server) and `serve.rs` (production SSR server).

use std::collections::HashMap;

/// A resolved prompt request ready to send to an AI provider.
pub struct PromptRequest {
    pub provider: String,
    pub system: String,
    pub user: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f64,
}

/// Response from an AI provider.
pub struct PromptResponse {
    pub text: String,
}

/// Execute a prompt against the appropriate AI provider.
pub async fn execute_prompt(req: &PromptRequest) -> Result<PromptResponse, String> {
    match req.provider.as_str() {
        "openai" => call_openai(req).await,
        "anthropic" => call_anthropic(req).await,
        "ollama" => call_ollama(req).await,
        other => call_openai_compatible(req, other).await,
    }
}

/// Resolve `{variable}` interpolations in a template string.
pub fn resolve_interpolations(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = String::with_capacity(template.len());
    let mut chars = template.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '{' {
            let mut var_name = String::new();
            for c in chars.by_ref() {
                if c == '}' {
                    break;
                }
                var_name.push(c);
            }
            if let Some(val) = vars.get(&var_name) {
                result.push_str(val);
            } else {
                // Keep the original interpolation if variable not found
                result.push('{');
                result.push_str(&var_name);
                result.push('}');
            }
        } else {
            result.push(ch);
        }
    }
    result
}

// ─── OpenAI ─────────────────────────────────────────────────────────────────

async fn call_openai(req: &PromptRequest) -> Result<PromptResponse, String> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY environment variable not set".to_string())?;

    let model = if req.model.is_empty() {
        "gpt-4o"
    } else {
        &req.model
    };

    let body = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": req.system },
            { "role": "user", "content": req.user }
        ],
        "max_tokens": req.max_tokens,
        "temperature": req.temperature,
    });

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("OpenAI request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("OpenAI error ({}): {}", status, text));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("OpenAI response parse error: {}", e))?;

    let text = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(PromptResponse { text })
}

// ─── Anthropic ──────────────────────────────────────────────────────────────

async fn call_anthropic(req: &PromptRequest) -> Result<PromptResponse, String> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| "ANTHROPIC_API_KEY environment variable not set".to_string())?;

    let model = if req.model.is_empty() {
        "claude-sonnet-4-5-20250929"
    } else {
        &req.model
    };

    let body = serde_json::json!({
        "model": model,
        "max_tokens": req.max_tokens,
        "system": req.system,
        "messages": [
            { "role": "user", "content": req.user }
        ],
    });

    let client = reqwest::Client::new();
    let resp = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Anthropic request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Anthropic error ({}): {}", status, text));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Anthropic response parse error: {}", e))?;

    let text = json["content"][0]["text"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(PromptResponse { text })
}

// ─── Ollama ─────────────────────────────────────────────────────────────────

async fn call_ollama(req: &PromptRequest) -> Result<PromptResponse, String> {
    let base_url =
        std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());

    let model = if req.model.is_empty() {
        "llama3"
    } else {
        &req.model
    };

    let body = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": req.system },
            { "role": "user", "content": req.user }
        ],
        "stream": true,
        "options": {
            "num_ctx": 16384,
            "num_predict": req.max_tokens
        },
    });

    let client = reqwest::Client::new();
    let mut resp = client
        .post(format!("{}/api/chat", base_url))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Ollama request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Ollama error ({}): {}", status, text));
    }

    // Stream NDJSON chunks, printing a dot per token so the user sees progress
    let mut full_text = String::new();
    let mut token_count: u32 = 0;
    let mut buf = String::new();

    while let Some(chunk) = resp.chunk().await.map_err(|e| format!("Ollama stream error: {}", e))? {
        buf.push_str(&String::from_utf8_lossy(&chunk));
        // Each chunk may contain one or more newline-delimited JSON objects
        while let Some(newline_pos) = buf.find('\n') {
            let line: String = buf.drain(..=newline_pos).collect();
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                if let Some(content) = json["message"]["content"].as_str() {
                    full_text.push_str(content);
                    token_count += 1;
                    // Print a dot every 20 tokens to show progress
                    if token_count.is_multiple_of(20) {
                        eprint!(".");
                    }
                }
            }
        }
    }
    // Process any remaining data in buffer (no trailing newline)
    let remaining = buf.trim();
    if !remaining.is_empty() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(remaining) {
            if let Some(content) = json["message"]["content"].as_str() {
                full_text.push_str(content);
            }
        }
    }
    if token_count > 0 {
        eprintln!(" ({} tokens)", token_count);
    }

    Ok(PromptResponse { text: full_text })
}

// ─── Generic OpenAI-compatible ──────────────────────────────────────────────

async fn call_openai_compatible(
    req: &PromptRequest,
    provider: &str,
) -> Result<PromptResponse, String> {
    let env_prefix = provider.to_uppercase().replace('-', "_");
    let api_url = std::env::var(format!("{}_API_URL", env_prefix))
        .map_err(|_| format!("{}_API_URL environment variable not set", env_prefix))?;
    let api_key = std::env::var(format!("{}_API_KEY", env_prefix)).unwrap_or_default();

    let model = if req.model.is_empty() {
        provider
    } else {
        &req.model
    };

    let body = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": req.system },
            { "role": "user", "content": req.user }
        ],
        "max_tokens": req.max_tokens,
        "temperature": req.temperature,
    });

    let client = reqwest::Client::new();
    let mut request = client
        .post(&api_url)
        .header("Content-Type", "application/json")
        .json(&body);

    if !api_key.is_empty() {
        request = request.header("Authorization", format!("Bearer {}", api_key));
    }

    let resp = request
        .send()
        .await
        .map_err(|e| format!("{} request failed: {}", provider, e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("{} error ({}): {}", provider, status, text));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("{} response parse error: {}", provider, e))?;

    // Try OpenAI-compatible response format
    let text = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(PromptResponse { text })
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_interpolations_basic() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        vars.insert("topic".to_string(), "Rust".to_string());

        let result = resolve_interpolations("Hello {name}, tell me about {topic}.", &vars);
        assert_eq!(result, "Hello Alice, tell me about Rust.");
    }

    #[test]
    fn test_resolve_interpolations_missing_var() {
        let vars = HashMap::new();
        let result = resolve_interpolations("Hello {unknown}!", &vars);
        assert_eq!(result, "Hello {unknown}!");
    }

    #[test]
    fn test_resolve_interpolations_no_vars() {
        let vars = HashMap::new();
        let result = resolve_interpolations("No variables here.", &vars);
        assert_eq!(result, "No variables here.");
    }

    #[test]
    fn test_resolve_interpolations_dotted_ref() {
        let mut vars = HashMap::new();
        vars.insert("user.name".to_string(), "Bob".to_string());
        let result = resolve_interpolations("Hi {user.name}!", &vars);
        assert_eq!(result, "Hi Bob!");
    }
}
