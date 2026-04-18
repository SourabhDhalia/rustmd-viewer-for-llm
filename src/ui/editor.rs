/// Renders the left-side Markdown input panel with a live stats bar.
pub fn show(ui: &mut egui::Ui, input: &mut String) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Markdown Input").strong().size(14.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let lines = input.lines().count();
                let words = input.split_whitespace().count();
                let chars = input.chars().count();
                ui.label(
                    egui::RichText::new(format!("{chars} chars  {words} words  {lines} lines"))
                        .weak()
                        .size(11.0),
                );
            });
        });
        ui.separator();
        egui::ScrollArea::vertical()
            .id_salt("editor_scroll")
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(input)
                        .desired_width(f32::INFINITY)
                        .desired_rows(40)
                        .font(egui::TextStyle::Monospace)
                        .hint_text("Paste your Markdown here..."),
                );
            });
    });
}
