use crate::mdformator::MdFormator;

/// Renders the right-side Markdown preview panel.
pub fn show(ui: &mut egui::Ui, formator: &mut MdFormator, markdown: &str) {
    ui.vertical(|ui| {
        ui.label(egui::RichText::new("Preview").strong().size(14.0));
        ui.separator();
        formator.render_scrollable(ui, markdown);
    });
}
