use crate::llm::{LlmAction, LlmProvider, LlmState, LlmStatus};

/// Renders the AI assistant panel.
/// Returns `Some(text)` when the user clicks "Apply to Editor".
pub fn show(ui: &mut egui::Ui, llm: &mut LlmState, doc_content: &str) -> Option<String> {
    llm.poll();
    let mut apply: Option<String> = None;

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

    // ── Row 1: Provider / Model / URL / API key ───────────────────────────────
    ui.horizontal_wrapped(|ui| {
        ui.label("Provider:");
        egui::ComboBox::from_id_salt("llm_provider")
            .selected_text(llm.provider.label())
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
        let model_changed = ui
            .add(egui::TextEdit::singleline(&mut llm.model).desired_width(160.0))
            .lost_focus();
        if model_changed {
            llm.refresh_caps();
        }

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

    // ── Row 2: Capability toggles (only shown when supported) ────────────────
    let caps = llm.caps.clone();
    let any_cap = caps.extended_thinking || caps.vision || caps.system_prompt
        || caps.temperature || caps.json_mode || caps.top_p;

    if any_cap {
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("Capabilities:").weak());

            // 🧠 Extended Thinking
            if caps.extended_thinking {
                ui.add_space(4.0);
                if ui
                    .selectable_label(
                        llm.thinking_enabled,
                        egui::RichText::new("🧠 Think").color(if llm.thinking_enabled {
                            egui::Color32::from_rgb(130, 200, 255)
                        } else {
                            ui.visuals().text_color()
                        }),
                    )
                    .on_hover_text("Enable extended chain-of-thought reasoning (Claude only)")
                    .clicked()
                {
                    llm.thinking_enabled = !llm.thinking_enabled;
                }
                if llm.thinking_enabled {
                    ui.label("Budget:");
                    ui.add(
                        egui::Slider::new(&mut llm.thinking_budget, 1000..=32000)
                            .suffix(" tok")
                            .clamping(egui::SliderClamping::Always),
                    );
                }
            }

            // 🖼 Vision
            if caps.vision {
                ui.add_space(4.0);
                let has_img = llm.image.is_some();
                if ui
                    .selectable_label(
                        has_img,
                        egui::RichText::new(if has_img { "🖼 Image ✓" } else { "🖼 Image" })
                            .color(if has_img {
                                egui::Color32::from_rgb(130, 220, 130)
                            } else {
                                ui.visuals().text_color()
                            }),
                    )
                    .on_hover_text("Drop an image file onto this window, or click to clear")
                    .clicked()
                {
                    // Clicking while image is attached → clear it
                    if has_img {
                        llm.image = None;
                    }
                }
                if let Some(img) = &llm.image {
                    ui.label(
                        egui::RichText::new(format!("📎 {}", img.filename))
                            .weak()
                            .size(11.0),
                    );
                } else {
                    ui.label(
                        egui::RichText::new("← drop image file onto window")
                            .weak()
                            .italics()
                            .size(11.0),
                    );
                }
            }

            // 🌡 Temperature
            if caps.temperature {
                ui.add_space(4.0);
                ui.checkbox(&mut llm.temperature_enabled, "🌡 Temp");
                if llm.temperature_enabled {
                    ui.add(
                        egui::Slider::new(&mut llm.temperature, 0.0..=1.0)
                            .step_by(0.05)
                            .clamping(egui::SliderClamping::Always),
                    );
                }
            }

            // Top-P
            if caps.top_p {
                ui.add_space(4.0);
                ui.checkbox(&mut llm.top_p_enabled, "Top-P");
                if llm.top_p_enabled {
                    ui.add(
                        egui::Slider::new(&mut llm.top_p, 0.0..=1.0)
                            .step_by(0.01)
                            .clamping(egui::SliderClamping::Always),
                    );
                }
            }

            // {} JSON mode
            if caps.json_mode {
                ui.add_space(4.0);
                ui.checkbox(&mut llm.json_mode, "{} JSON mode")
                    .on_hover_text("Force the model to respond with valid JSON");
            }

            // Max tokens (always shown)
            ui.add_space(4.0);
            ui.label("Max tokens:");
            ui.add(
                egui::DragValue::new(&mut llm.max_tokens)
                    .range(256..=128_000)
                    .speed(256.0),
            );
        });

        // System prompt (full-width row when enabled)
        if caps.system_prompt {
            ui.horizontal(|ui| {
                ui.checkbox(&mut llm.system_prompt_enabled, "📝 System prompt:");
                if llm.system_prompt_enabled {
                    ui.add(
                        egui::TextEdit::singleline(&mut llm.system_prompt)
                            .desired_width(f32::INFINITY)
                            .hint_text("You are a helpful assistant…"),
                    );
                }
            });
        }

        ui.add_space(4.0);
    }

    // ── Row 3: Prompt + action buttons + stop ────────────────────────────────
    ui.horizontal(|ui| {
        ui.label("Prompt:");
        ui.add(
            egui::TextEdit::singleline(&mut llm.prompt)
                .desired_width(360.0)
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
            let btn  = egui::Button::new(action.label());
            let resp = ui.add_enabled(!loading, btn).on_hover_text(action.tooltip());
            if resp.clicked() {
                llm.send(action, doc_content);
            }
        }

        // ⏹ Stop button — only shown while loading
        if loading {
            ui.add_space(4.0);
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("⏹ Stop").color(egui::Color32::from_rgb(220, 80, 80)),
                    )
                    .fill(egui::Color32::from_rgb(60, 20, 20)),
                )
                .on_hover_text("Cancel the current request")
                .clicked()
            {
                llm.cancel();
            }
            ui.spinner();
            ui.label(egui::RichText::new("Thinking…").weak().italics());
        }
    });

    ui.add_space(4.0);

    // ── Row 4: Response area ──────────────────────────────────────────────────
    match &llm.status {
        LlmStatus::Idle    => {}
        LlmStatus::Loading => {
            ui.label(egui::RichText::new("⏳ Waiting for response…").weak());
        }
        LlmStatus::Error(e) => {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("⚠️ Error:").color(egui::Color32::RED).strong());
                ui.label(egui::RichText::new(e).color(egui::Color32::RED));
            });
        }
        LlmStatus::Response { text, thinking, action } => {
            let replaces      = action.replaces_editor();
            let text_clone    = text.clone();
            let append_clone  = format!("\n\n---\n\n{text}");
            let thinking_text = thinking.clone();

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

            // Thinking block (collapsible)
            if let Some(th) = thinking_text {
                egui::CollapsingHeader::new(
                    egui::RichText::new("🧠 Reasoning").weak().italics().size(11.0),
                )
                .default_open(false)
                .show(ui, |ui| {
                    let mut th_display = th.clone();
                    egui::ScrollArea::vertical()
                        .id_salt("llm_thinking_scroll")
                        .max_height(100.0)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut th_display)
                                    .desired_width(f32::INFINITY)
                                    .font(egui::TextStyle::Small),
                            );
                        });
                });
            }

            // Response text
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
