use crate::llm::LlmState;
use crate::mdformator::MdFormator;
use crate::ui::{editor, llm_panel, preview};

pub struct MdViewApp {
    pub input: String,
    formator:  MdFormator,
    llm:       LlmState,
}

impl MdViewApp {
    pub fn with_content(content: String) -> Self {
        Self { input: content, formator: MdFormator::new(), llm: LlmState::default() }
    }
}

impl Default for MdViewApp {
    fn default() -> Self {
        Self::with_content(DEFAULT_CONTENT.to_string())
    }
}

impl eframe::App for MdViewApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ── Drag-and-drop file loading ─────────────────────────────────────
        ctx.input(|i| {
            for file in &i.raw.dropped_files {
                if let Some(path) = &file.path {
                    if let Ok(content) = std::fs::read_to_string(path) {
                        self.input = content;
                        self.formator = MdFormator::new();
                    }
                }
            }
        });

        // ── Top menu bar ──────────────────────────────────────────────────
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("mdToView").strong());
                ui.separator();
                if ui.button("📋 Load Stress Test").clicked() {
                    let path = std::path::Path::new(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/tests/stress_test.md"
                    ));
                    if let Ok(content) = std::fs::read_to_string(path) {
                        self.input = content;
                        self.formator = MdFormator::new();
                    }
                }
                if ui.button("🗑 Clear").clicked() {
                    self.input.clear();
                    self.formator = MdFormator::new();
                }

                ui.separator();

                // AI panel toggle
                let ai_label = if self.llm.visible { "🤖 AI ▲" } else { "🤖 AI ▼" };
                if ui.button(ai_label).on_hover_text("Toggle AI assistant panel").clicked() {
                    self.llm.visible = !self.llm.visible;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new("Drop a .md file anywhere to load it")
                            .weak()
                            .size(11.0),
                    );
                });
            });
        });

        // ── AI panel (bottom) ─────────────────────────────────────────────
        if self.llm.visible {
            egui::TopBottomPanel::bottom("llm_panel")
                .resizable(true)
                .default_height(220.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if let Some(new_content) =
                            llm_panel::show(ui, &mut self.llm, &self.input)
                        {
                            // Apply or append AI response to the editor
                            if new_content.starts_with("\n\n---\n\n") {
                                self.input.push_str(&new_content);
                            } else {
                                self.input = new_content;
                            }
                            self.formator = MdFormator::new();
                        }
                    });
                });
        }

        // ── Split editor / preview panels ─────────────────────────────────
        egui::SidePanel::left("editor_panel")
            .resizable(true)
            .default_width(ctx.screen_rect().width() / 2.0)
            .show(ctx, |ui| {
                editor::show(ui, &mut self.input);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            preview::show(ui, &mut self.formator, &self.input);
        });
    }
}

pub fn run(initial_content: Option<String>) -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([600.0, 400.0])
            .with_title("mdToView")
            .with_drag_and_drop(true),
        ..Default::default()
    };

    eframe::run_native(
        "mdtoview",
        options,
        Box::new(move |_cc| {
            let app = match initial_content {
                Some(content) => MdViewApp::with_content(content),
                None          => MdViewApp::default(),
            };
            Ok(Box::new(app))
        }),
    )
}

// ── Default demo content ──────────────────────────────────────────────────────

const DEFAULT_CONTENT: &str = r#"# mdToView — AI-Powered Markdown Editor

## Quick Start

1. **Write or paste** Markdown on the left.
2. **See the live preview** on the right.
3. Click **🤖 AI ▼** to open the AI assistant.

---

## AI Assistant Features

| Action | What it does |
|--------|-------------|
| ✨ Generate | Write markdown from a prompt |
| 🔧 Improve | Fix formatting and structure |
| 📋 Summarize | Condense a long document |
| ❓ Ask | Ask questions about the content |

### Supported Providers
- **Claude** — Anthropic API (needs API key)
- **Ollama** — Runs locally, no key needed (`ollama serve`)
- **OpenAI-Compatible** — OpenAI, Groq, Mistral, LM Studio, etc.

---

## Markdown Features

**Bold**, _italic_, ~~strikethrough~~, `inline code`

> Blockquotes work too.

- [x] Task lists
- [ ] And nested lists
  - Sub-item

```rust
fn main() {
    println!("Hello from mdToView!");
}
```

Math: $e^{i\pi} + 1 = 0$, Greek: $\alpha$, $\beta$, $\Omega$

> Drop any `.md` file onto this window to preview it instantly.
"#;
