use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;

use super::{CallParams, LlmProvider};

/// Route to the correct provider. Returns (assistant_text, optional_thinking_text).
pub fn call(p: &CallParams) -> Result<(String, Option<String>), String> {
    match &p.provider {
        LlmProvider::Claude           => claude(p),
        LlmProvider::Ollama           => ollama(p),
        LlmProvider::OpenAiCompatible => openai_compat(p),
    }
}

// ── Claude (Anthropic Messages API) ──────────────────────────────────────────

fn claude(p: &CallParams) -> Result<(String, Option<String>), String> {
    if p.api_key.is_empty() {
        return Err("Claude requires an API key. Add it in the AI panel settings.".into());
    }
    let url = format!("{}/v1/messages", p.base_url);

    // Build the user content block(s)
    let user_content = if let Some(img) = &p.image {
        serde_json::json!([
            {
                "type": "image",
                "source": {
                    "type": "base64",
                    "media_type": img.media_type,
                    "data": img.base64
                }
            },
            { "type": "text", "text": p.prompt }
        ])
    } else {
        serde_json::json!(p.prompt)
    };

    let mut body = serde_json::json!({
        "model":      p.model,
        "max_tokens": p.max_tokens,
        "messages":   [{ "role": "user", "content": user_content }]
    });

    if let Some(sys) = &p.system_prompt {
        body["system"] = serde_json::json!(sys);
    }
    if let Some(temp) = p.temperature {
        body["temperature"] = serde_json::json!(temp);
    }
    if let Some(tp) = p.top_p {
        body["top_p"] = serde_json::json!(tp);
    }
    if p.thinking_enabled {
        body["thinking"] = serde_json::json!({
            "type":         "enabled",
            "budget_tokens": p.thinking_budget
        });
        // Thinking requires temperature=1 per Anthropic docs
        body["temperature"] = serde_json::json!(1);
    }

    let mut req = ureq::post(&url)
        .set("x-api-key", &p.api_key)
        .set("anthropic-version", "2023-06-01")
        .set("content-type", "application/json");

    if p.thinking_enabled {
        req = req.set("anthropic-beta", "interleaved-thinking-2025-05-14");
    }

    let resp: serde_json::Value = req
        .send_json(body)
        .map_err(|e| format!("Claude request failed: {e}"))?
        .into_json()
        .map_err(|e| format!("Claude response parse error: {e}"))?;

    // Extract text and optional thinking blocks from content array
    let content = resp["content"].as_array().ok_or_else(|| {
        resp["error"]["message"]
            .as_str()
            .map(|m| format!("Claude error: {m}"))
            .unwrap_or_else(|| format!("Unexpected Claude response: {resp}"))
    })?;

    let mut text_parts    = Vec::new();
    let mut thinking_parts = Vec::new();

    for block in content {
        match block["type"].as_str() {
            Some("text")     => if let Some(t) = block["text"].as_str() { text_parts.push(t.to_string()); }
            Some("thinking") => if let Some(t) = block["thinking"].as_str() { thinking_parts.push(t.to_string()); }
            _ => {}
        }
    }

    if text_parts.is_empty() {
        return Err(format!("No text in Claude response: {resp}"));
    }

    let thinking = if thinking_parts.is_empty() { None } else { Some(thinking_parts.join("\n\n")) };
    Ok((text_parts.join("\n\n"), thinking))
}

// ── Ollama (local) ────────────────────────────────────────────────────────────

fn ollama(p: &CallParams) -> Result<(String, Option<String>), String> {
    let url = format!("{}/api/chat", p.base_url);

    let mut user_msg = serde_json::json!({
        "role":    "user",
        "content": p.prompt
    });

    // Ollama vision: pass images as base64 array
    if let Some(img) = &p.image {
        user_msg["images"] = serde_json::json!([img.base64]);
    }

    let mut messages = Vec::new();
    if let Some(sys) = &p.system_prompt {
        messages.push(serde_json::json!({ "role": "system", "content": sys }));
    }
    messages.push(user_msg);

    let mut body = serde_json::json!({
        "model":    p.model,
        "messages": messages,
        "stream":   false
    });

    let mut opts = serde_json::json!({});
    if let Some(temp) = p.temperature { opts["temperature"] = serde_json::json!(temp); }
    if let Some(tp)   = p.top_p       { opts["top_p"]       = serde_json::json!(tp);   }
    if opts.as_object().map(|o| !o.is_empty()).unwrap_or(false) {
        body["options"] = opts;
    }

    let resp: serde_json::Value = ureq::post(&url)
        .set("content-type", "application/json")
        .send_json(body)
        .map_err(|e| format!("Ollama request failed: {e}\n\nIs Ollama running? Start it with: ollama serve"))?
        .into_json()
        .map_err(|e| format!("Ollama response parse error: {e}"))?;

    resp["message"]["content"]
        .as_str()
        .map(|s| (s.to_string(), None))
        .ok_or_else(|| format!("Unexpected Ollama response: {resp}"))
}

// ── OpenAI-compatible (OpenAI / Groq / Mistral / LM Studio / etc.) ───────────

fn openai_compat(p: &CallParams) -> Result<(String, Option<String>), String> {
    let url = format!("{}/v1/chat/completions", p.base_url);

    // Build user content (text or vision array)
    let user_content = if let Some(img) = &p.image {
        let data_url = format!("data:{};base64,{}", img.media_type, img.base64);
        serde_json::json!([
            { "type": "image_url", "image_url": { "url": data_url } },
            { "type": "text",      "text": p.prompt }
        ])
    } else {
        serde_json::json!(p.prompt)
    };

    let mut messages = Vec::new();
    if let Some(sys) = &p.system_prompt {
        messages.push(serde_json::json!({ "role": "system", "content": sys }));
    }
    messages.push(serde_json::json!({ "role": "user", "content": user_content }));

    let mut body = serde_json::json!({
        "model":      p.model,
        "messages":   messages,
        "max_tokens": p.max_tokens
    });

    if let Some(temp) = p.temperature { body["temperature"] = serde_json::json!(temp); }
    if let Some(tp)   = p.top_p       { body["top_p"]       = serde_json::json!(tp);   }
    if p.json_mode {
        body["response_format"] = serde_json::json!({ "type": "json_object" });
    }

    let mut req = ureq::post(&url).set("content-type", "application/json");
    if !p.api_key.is_empty() {
        req = req.set("Authorization", &format!("Bearer {}", p.api_key));
    }

    let resp: serde_json::Value = req
        .send_json(body)
        .map_err(|e| format!("OpenAI-compat request failed: {e}"))?
        .into_json()
        .map_err(|e| format!("OpenAI-compat response parse error: {e}"))?;

    resp["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| (s.to_string(), None))
        .ok_or_else(|| {
            resp["error"]["message"]
                .as_str()
                .map(|m| format!("API error: {m}"))
                .unwrap_or_else(|| format!("Unexpected response: {resp}"))
        })
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Encode raw image bytes to base64 and detect the media type from the header.
pub fn encode_image(bytes: &[u8]) -> (String, String) {
    let media_type = detect_media_type(bytes);
    (media_type, B64.encode(bytes))
}

fn detect_media_type(bytes: &[u8]) -> String {
    match bytes {
        [0x89, 0x50, 0x4E, 0x47, ..] => "image/png",
        [0xFF, 0xD8, 0xFF, ..]       => "image/jpeg",
        [0x47, 0x49, 0x46, ..]       => "image/gif",
        [b'R', b'I', b'F', b'F', _, _, _, _, b'W', b'E', b'B', b'P', ..] => "image/webp",
        _                            => "image/png",
    }.to_string()
}
