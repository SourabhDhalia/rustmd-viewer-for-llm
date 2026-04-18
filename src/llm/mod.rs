pub mod client;

use std::sync::mpsc::{channel, Receiver, Sender};

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

    /// Whether the response should offer to replace the editor content.
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

// ── State ─────────────────────────────────────────────────────────────────────

pub enum LlmStatus {
    Idle,
    Loading,
    Response { text: String, action: LlmAction },
    Error(String),
}

pub struct LlmState {
    pub provider: LlmProvider,
    pub api_key:  String,
    pub model:    String,
    pub base_url: String,
    pub prompt:   String,
    pub status:   LlmStatus,
    pub visible:  bool,
    rx: Option<Receiver<Result<(String, LlmAction), String>>>,
}

impl Default for LlmState {
    fn default() -> Self {
        let provider = LlmProvider::Claude;
        Self {
            model:    provider.default_model().to_string(),
            base_url: provider.default_base_url().to_string(),
            provider,
            api_key:  String::new(),
            prompt:   String::new(),
            status:   LlmStatus::Idle,
            visible:  false,
            rx:       None,
        }
    }
}

impl LlmState {
    pub fn is_loading(&self) -> bool {
        matches!(self.status, LlmStatus::Loading)
    }

    /// Dispatch a request in a background thread.
    pub fn send(&mut self, action: LlmAction, doc_content: &str) {
        let prompt = action.build_prompt(&self.prompt, doc_content);
        self.status = LlmStatus::Loading;

        let (tx, rx): (Sender<_>, Receiver<_>) = channel();
        self.rx = Some(rx);

        let provider = self.provider.clone();
        let api_key  = self.api_key.clone();
        let model    = self.model.clone();
        let base_url = self.base_url.clone();

        std::thread::spawn(move || {
            let result = client::call(&provider, &api_key, &model, &base_url, &prompt)
                .map(|text| (text, action));
            let _ = tx.send(result);
        });
    }

    /// Poll the background thread — call once per frame.
    pub fn poll(&mut self) {
        if let Some(rx) = &self.rx {
            if let Ok(result) = rx.try_recv() {
                self.rx = None;
                self.status = match result {
                    Ok((text, action)) => LlmStatus::Response { text, action },
                    Err(e)             => LlmStatus::Error(e),
                };
            }
        }
    }

    pub fn change_provider(&mut self, p: LlmProvider) {
        self.model    = p.default_model().to_string();
        self.base_url = p.default_base_url().to_string();
        self.provider = p;
    }
}
