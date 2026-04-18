use super::LlmProvider;

/// What a specific provider+model combination actually supports.
/// Detected automatically from the model name so the UI can show
/// only the relevant capability toggles.
#[derive(Clone, Debug, Default)]
pub struct ModelCapabilities {
    /// Extended chain-of-thought reasoning (Claude 3.7+ / 4.x)
    pub extended_thinking: bool,
    /// Send images alongside the prompt (Claude 3+, GPT-4o/vision, LLaVA…)
    pub vision: bool,
    /// Separate system prompt field
    pub system_prompt: bool,
    /// Temperature slider (0.0 – 1.0)
    pub temperature: bool,
    /// Force JSON output (OpenAI gpt-* models)
    pub json_mode: bool,
    /// Top-P / nucleus sampling
    pub top_p: bool,
}

impl ModelCapabilities {
    /// Derive capabilities from the chosen provider and the model name the
    /// user typed. Pattern-matches on lower-cased model name substrings so it
    /// keeps working for future model versions without code changes.
    pub fn detect(provider: &LlmProvider, model: &str) -> Self {
        let m = model.to_lowercase();
        match provider {
            LlmProvider::Claude => Self {
                extended_thinking: claude_supports_thinking(&m),
                vision:            claude_supports_vision(&m),
                system_prompt:     true,
                temperature:       true,
                json_mode:         false,
                top_p:             true,
            },
            LlmProvider::OpenAiCompatible => Self {
                extended_thinking: false,
                vision:            openai_supports_vision(&m),
                system_prompt:     true,
                temperature:       true,
                json_mode:         m.starts_with("gpt-"),
                top_p:             true,
            },
            LlmProvider::Ollama => Self {
                extended_thinking: false,
                vision:            ollama_supports_vision(&m),
                system_prompt:     true,
                temperature:       true,
                json_mode:         false,
                top_p:             true,
            },
        }
    }
}

// ── Per-provider helpers ──────────────────────────────────────────────────────

fn claude_supports_thinking(m: &str) -> bool {
    // claude-3-7-*, claude-opus-4*, claude-sonnet-4*, claude-haiku-4*, etc.
    m.contains("3-7") || m.contains("opus-4") || m.contains("sonnet-4") || m.contains("haiku-4")
}

fn claude_supports_vision(m: &str) -> bool {
    // All claude-3 and claude-4 family models support vision
    m.contains("claude-3") || m.contains("claude-4")
        || m.contains("opus") || m.contains("sonnet") || m.contains("haiku")
}

fn openai_supports_vision(m: &str) -> bool {
    m.contains("4o") || m.contains("vision") || m.contains("4.5")
        || m.starts_with("o1") || m.starts_with("o3") || m.contains("gpt-4-turbo")
}

fn ollama_supports_vision(m: &str) -> bool {
    // Common multimodal Ollama models
    m.contains("llava") || m.contains("bakllava") || m.contains("moondream")
        || m.contains("cogvlm") || m.contains("vision") || m.contains("-vl")
        || m.contains("minicpm") || m.contains("qwen-vl")
}
