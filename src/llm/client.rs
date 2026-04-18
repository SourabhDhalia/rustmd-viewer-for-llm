use super::LlmProvider;

/// Route to the correct provider and return the assistant reply text.
pub fn call(
    provider: &LlmProvider,
    api_key:  &str,
    model:    &str,
    base_url: &str,
    prompt:   &str,
) -> Result<String, String> {
    match provider {
        LlmProvider::Claude           => claude(api_key, model, base_url, prompt),
        LlmProvider::Ollama           => ollama(model, base_url, prompt),
        LlmProvider::OpenAiCompatible => openai_compat(api_key, model, base_url, prompt),
    }
}

// ── Claude (Anthropic Messages API) ──────────────────────────────────────────

fn claude(api_key: &str, model: &str, base_url: &str, prompt: &str) -> Result<String, String> {
    if api_key.is_empty() {
        return Err("Claude requires an API key. Add it in the AI panel settings.".into());
    }
    let url  = format!("{base_url}/v1/messages");
    let body = serde_json::json!({
        "model": model,
        "max_tokens": 4096,
        "messages": [{ "role": "user", "content": prompt }]
    });

    let resp: serde_json::Value = ureq::post(&url)
        .set("x-api-key", api_key)
        .set("anthropic-version", "2023-06-01")
        .set("content-type", "application/json")
        .send_json(body)
        .map_err(|e| format!("Claude request failed: {e}"))?
        .into_json()
        .map_err(|e| format!("Claude response parse error: {e}"))?;

    resp["content"][0]["text"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| {
            resp["error"]["message"]
                .as_str()
                .map(|m| format!("Claude error: {m}"))
                .unwrap_or_else(|| format!("Unexpected Claude response: {resp}"))
        })
}

// ── Ollama (local) ────────────────────────────────────────────────────────────

fn ollama(model: &str, base_url: &str, prompt: &str) -> Result<String, String> {
    let url  = format!("{base_url}/api/chat");
    let body = serde_json::json!({
        "model": model,
        "messages": [{ "role": "user", "content": prompt }],
        "stream": false
    });

    let resp: serde_json::Value = ureq::post(&url)
        .set("content-type", "application/json")
        .send_json(body)
        .map_err(|e| format!("Ollama request failed: {e}\n\nIs Ollama running? Start it with: ollama serve"))?
        .into_json()
        .map_err(|e| format!("Ollama response parse error: {e}"))?;

    resp["message"]["content"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| format!("Unexpected Ollama response: {resp}"))
}

// ── OpenAI-compatible (OpenAI / Groq / Mistral / LM Studio / etc.) ───────────

fn openai_compat(api_key: &str, model: &str, base_url: &str, prompt: &str) -> Result<String, String> {
    let url  = format!("{base_url}/v1/chat/completions");
    let body = serde_json::json!({
        "model": model,
        "messages": [{ "role": "user", "content": prompt }]
    });

    let mut req = ureq::post(&url).set("content-type", "application/json");
    if !api_key.is_empty() {
        req = req.set("Authorization", &format!("Bearer {api_key}"));
    }

    let resp: serde_json::Value = req
        .send_json(body)
        .map_err(|e| format!("OpenAI-compat request failed: {e}"))?
        .into_json()
        .map_err(|e| format!("OpenAI-compat response parse error: {e}"))?;

    resp["choices"][0]["message"]["content"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| {
            resp["error"]["message"]
                .as_str()
                .map(|m| format!("API error: {m}"))
                .unwrap_or_else(|| format!("Unexpected response: {resp}"))
        })
}
