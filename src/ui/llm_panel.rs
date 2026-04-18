use crate::llm::{LlmAction, LlmProvider, LlmState, LlmStatus};

/// Renders the collapsible AI assistant bottom panel.
/// Returns `Some(text)` when the user clicks "Apply to Editor".
pub fn show(ui: &mut egui::Ui, llm: &mut LlmState, doc_content: &str) -> Option<String> {
    llm.poll();
    let mut apply: Option<String> = None;

    // ── Toggle button in the header bar (called from app.rs menu) ────────────
    // The actual panel body is drawn here inside a BottomPanel.
    if !llm.visible {
        return None;
    }

    ui.separator();

    egui::CollapsingHeader::new(
        egui::RichText::new("🤖  AI Assistant").strong().size(13.0),
    )
    .default_open(true)
    .show(ui, |ui| {
        apply = panel_body(ui, llm, doc_content);
    });

    apply
}

fn panel_body(ui: &mut egui::Ui, llm: &mut LlmState, doc_content: &str) -> Option<String> {
    let mut apply: Option<String> = None;

    // ── Settings row ─────────────────────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.label("Provider:");
        let current_label = llm.provider.label();
        egui::ComboBox::from_id_salt("llm_provider")
            .selected_text(current_label)
            .show_ui(ui, |ui| {
                for p in LlmProvider::all() {
                    let selected = &llm.provider == p;
                    if ui.selectable_label(selected, p.label()).clicked() && !selected {
                        llm.change_provider(p.clone());
                    }
                }
            });

        ui.separator();
        ui.label("Model:");
        ui.add(egui::TextEdit::singleline(&mut llm.model).desired_width(160.0));

        ui.separator();
        ui.label("Base URL:");
        ui.add(egui::TextEdit::singleline(&mut llm.base_url).desired_width(220.0));

        if llm.provider.needs_api_key() {
            ui.separator();
            ui.label("API Key:");
            ui.add(
                egui::TextEdit::singleline(&mut llm.api_key)
                    .password(true)
                    .desired_width(200.0)
                    .hint_text("sk-…"),
            );
        }
    });

    ui.add_space(4.0);

    // ── Prompt + action buttons row ───────────────────────────────────────────
    ui.horizontal(|ui| {
        ui.label("Prompt:");
        ui.add(
            egui::TextEdit::singleline(&mut llm.prompt)
                .desired_width(400.0)
                .hint_text("Describe what you want…"),
        );

        ui.add_space(8.0);

        let loading = llm.is_loading();

        for action in [
            LlmAction::Generate,
            LlmAction::Improve,
            LlmAction::Summarize,
            LlmAction::Ask,
        ] {
            let btn = egui::Button::new(action.label());
            let resp = ui.add_enabled(!loading, btn).on_hover_text(action.tooltip());
            if resp.clicked() {
                llm.send(action, doc_content);
            }
        }

        if loading {
            ui.spinner();
            ui.label(egui::RichText::new("Thinking…").weak().italics());
        }
    });

    ui.add_space(4.0);

    // ── Response area ─────────────────────────────────────────────────────────
    match &llm.status {
        LlmStatus::Idle => {}

        LlmStatus::Loading => {
            ui.label(egui::RichText::new("⏳ Waiting for response…").weak());
        }

        LlmStatus::Error(e) => {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("⚠️ Error:").color(egui::Color32::RED).strong());
                ui.label(egui::RichText::new(e).color(egui::Color32::RED));
            });
        }

        LlmStatus::Response { text, action } => {
            let replaces = action.replaces_editor();
            // Clone what we need before any mutation
            let text_clone   = text.clone();
            let append_clone = format!("\n\n---\n\n{text}");

            let mut dismiss = false;
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("✅ Response ready").strong());
                if replaces {
                    if ui
                        .button("📋 Apply to Editor")
                        .on_hover_text("Replace editor content with this response")
                        .clicked()
                    {
                        apply   = Some(text_clone.clone());
                        dismiss = true;
                    }
                    if ui
                        .button("➕ Append to Editor")
                        .on_hover_text("Append response at the end of the editor")
                        .clicked()
                    {
                        apply   = Some(append_clone);
                        dismiss = true;
                    }
                }
                if ui.button("🗑 Dismiss").clicked() {
                    dismiss = true;
                }
            });

            // Show response text (read-only multiline)
            let mut display = text_clone.clone();
            egui::ScrollArea::vertical()
                .id_salt("llm_response_scroll")
                .max_height(120.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut display)
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace),
                    );
                });

            if dismiss {
                llm.status = LlmStatus::Idle;
            }
        }
    }

    apply
}
