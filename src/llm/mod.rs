pub mod capabilities;
pub mod client;

use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use capabilities::ModelCapabilities;

// ── Provider ──────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug)]
pub enum LlmProvider {
    Claude,
    Ollama,
    OpenAiCompatible,
}

impl LlmProvider {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Claude           => "Claude (Anthropic)",
            Self::Ollama           => "Ollama (Local)",
            Self::OpenAiCompatible => "OpenAI-Compatible",
        }
    }

    pub fn default_model(&self) -> &'static str {
        match self {
            Self::Claude           => "claude-sonnet-4-6",
            Self::Ollama           => "llama3.2",
            Self::OpenAiCompatible => "gpt-4o",
        }
    }

    pub fn default_base_url(&self) -> &'static str {
        match self {
            Self::Claude           => "https://api.anthropic.com",
            Self::Ollama           => "http://localhost:11434",
            Self::OpenAiCompatible => "https://api.openai.com",
        }
    }

    pub fn needs_api_key(&self) -> bool {
        !matches!(self, Self::Ollama)
    }

    pub fn all() -> &'static [Self] {
        &[Self::Claude, Self::Ollama, Self::OpenAiCompatible]
    }
}

// ── Action ────────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug)]
pub enum LlmAction {
    Generate,
    Improve,
    Summarize,
    Ask,
}

impl LlmAction {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Generate  => "✨ Generate",
            Self::Improve   => "🔧 Improve",
            Self::Summarize => "📋 Summarize",
            Self::Ask       => "❓ Ask",
        }
    }

    pub fn tooltip(&self) -> &'static str {
        match self {
            Self::Generate  => "Generate markdown from your prompt",
            Self::Improve   => "Improve the current editor content",
            Self::Summarize => "Summarize the current editor content",
            Self::Ask       => "Ask a question about the current content",
        }
    }

    pub fn replaces_editor(&self) -> bool {
        !matches!(self, Self::Ask)
    }

    pub fn build_prompt(&self, user_prompt: &str, doc_content: &str) -> String {
        match self {
            Self::Generate => format!(
                "You are a markdown writing assistant. Generate well-structured, \
                 properly formatted CommonMark markdown based on this request:\n\n\
                 {user_prompt}\n\n\
                 Respond ONLY with the markdown. No preamble, no explanation."
            ),
            Self::Improve => format!(
                "You are a markdown expert. Improve the following markdown document — \
                 fix formatting, improve structure, correct grammar, add missing headers \
                 where appropriate. Preserve ALL original content and meaning.\n\n\
                 Document:\n{doc_content}\n\n\
                 Additional instructions: {user_prompt}\n\n\
                 Respond ONLY with the improved markdown. No preamble, no explanation."
            ),
            Self::Summarize => format!(
                "Summarize the following markdown document into a concise markdown \
                 summary using headers and bullet points. Keep all key facts.\n\n\
                 Document:\n{doc_content}\n\n\
                 Respond ONLY with the markdown summary."
            ),
            Self::Ask => format!(
                "Based on the following markdown document, answer this question \
                 concisely and accurately:\n\n\
                 Question: {user_prompt}\n\n\
                 Document:\n{doc_content}"
            ),
        }
    }
}

// ── Attached image ────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct AttachedImage {
    pub media_type: String,   // "image/png", "image/jpeg", etc.
    pub base64:     String,   // standard base64-encoded bytes
    pub filename:   String,   // display name only
}

// ── Call parameters passed to the background thread ──────────────────────────

#[derive(Clone)]
pub struct CallParams {
    pub provider:          LlmProvider,
    pub api_key:           String,
    pub model:             String,
    pub base_url:          String,
    pub prompt:            String,
    pub system_prompt:     Option<String>,
    pub thinking_enabled:  bool,
    pub thinking_budget:   u32,
    pub image:             Option<AttachedImage>,
    pub temperature:       Option<f32>,
    pub max_tokens:        u32,
    pub json_mode:         bool,
    pub top_p:             Option<f32>,
}

// ── Status ────────────────────────────────────────────────────────────────────

pub enum LlmStatus {
    Idle,
    Loading,
    Response { text: String, thinking: Option<String>, action: LlmAction },
    Error(String),
}

// ── State ─────────────────────────────────────────────────────────────────────

pub struct LlmState {
    // Connection
    pub provider: LlmProvider,
    pub api_key:  String,
    pub model:    String,
    pub base_url: String,

    // Prompt
    pub prompt:   String,

    // Capabilities (auto-detected, shown in UI)
    pub caps:     ModelCapabilities,

    // Capability settings (user-controlled)
    pub thinking_enabled:      bool,
    pub thinking_budget:       u32,
    pub image:                 Option<AttachedImage>,
    pub system_prompt:         String,
    pub system_prompt_enabled: bool,
    pub temperature:           f32,
    pub temperature_enabled:   bool,
    pub max_tokens:            u32,
    pub top_p:                 f32,
    pub top_p_enabled:         bool,
    pub json_mode:             bool,

    // UI
    pub visible:  bool,
    pub status:   LlmStatus,
    rx:      Option<Receiver<Result<(String, Option<String>, LlmAction), String>>>,
    cancel:  Option<Arc<AtomicBool>>,
}

impl Default for LlmState {
    fn default() -> Self {
        let provider = LlmProvider::Claude;
        let model    = provider.default_model().to_string();
        let caps     = ModelCapabilities::detect(&provider, &model);
        Self {
            base_url: provider.default_base_url().to_string(),
            provider,
            api_key:  String::new(),
            model,
            caps,
            prompt:   String::new(),

            thinking_enabled:      false,
            thinking_budget:       8000,
            image:                 None,
            system_prompt:         String::new(),
            system_prompt_enabled: false,
            temperature:           0.7,
            temperature_enabled:   false,
            max_tokens:            4096,
            top_p:                 0.95,
            top_p_enabled:         false,
            json_mode:             false,

            visible:  false,
            status:   LlmStatus::Idle,
            rx:       None,
            cancel:   None,
        }
    }
}

impl LlmState {
    pub fn is_loading(&self) -> bool {
        matches!(self.status, LlmStatus::Loading)
    }

    /// Cancel a running request. The background thread will see the flag and
    /// return early; the response is dropped and status resets to Idle.
    pub fn cancel(&mut self) {
        if let Some(flag) = &self.cancel {
            flag.store(true, Ordering::Relaxed);
        }
        self.rx     = None;
        self.cancel = None;
        self.status = LlmStatus::Idle;
    }

    pub fn send(&mut self, action: LlmAction, doc_content: &str) {
        let prompt = action.build_prompt(&self.prompt, doc_content);
        self.status = LlmStatus::Loading;

        let cancel_flag = Arc::new(AtomicBool::new(false));
        self.cancel = Some(cancel_flag.clone());

        let (tx, rx): (Sender<_>, Receiver<_>) = channel();
        self.rx = Some(rx);

        let params = CallParams {
            provider:         self.provider.clone(),
            api_key:          self.api_key.clone(),
            model:            self.model.clone(),
            base_url:         self.base_url.clone(),
            prompt,
            system_prompt:    if self.system_prompt_enabled && !self.system_prompt.is_empty() {
                Some(self.system_prompt.clone())
            } else {
                None
            },
            thinking_enabled: self.thinking_enabled && self.caps.extended_thinking,
            thinking_budget:  self.thinking_budget,
            image:            if self.caps.vision { self.image.clone() } else { None },
            temperature:      if self.temperature_enabled { Some(self.temperature) } else { None },
            max_tokens:       self.max_tokens,
            json_mode:        self.json_mode && self.caps.json_mode,
            top_p:            if self.top_p_enabled { Some(self.top_p) } else { None },
        };

        std::thread::spawn(move || {
            if cancel_flag.load(Ordering::Relaxed) { return; }
            let result = client::call(&params).map(|(text, thinking)| (text, thinking, action));
            if !cancel_flag.load(Ordering::Relaxed) {
                let _ = tx.send(result);
            }
        });
    }

    pub fn poll(&mut self) {
        if let Some(rx) = &self.rx {
            if let Ok(result) = rx.try_recv() {
                self.rx = None;
                self.status = match result {
                    Ok((text, thinking, action)) => LlmStatus::Response { text, thinking, action },
                    Err(e)                       => LlmStatus::Error(e),
                };
            }
        }
    }

    pub fn change_provider(&mut self, p: LlmProvider) {
        self.model    = p.default_model().to_string();
        self.base_url = p.default_base_url().to_string();
        self.caps     = ModelCapabilities::detect(&p, &self.model);
        self.provider = p;
    }

    pub fn refresh_caps(&mut self) {
        self.caps = ModelCapabilities::detect(&self.provider, &self.model);
    }
}
