use super::host::AreaHost;
use crate::EditorKind;

pub(super) const HEADER_HEIGHT: f32 = 22.0;

/// Draws the strip above an area's body.
pub(super) fn draw_header(ui: &mut egui::Ui, rect: egui::Rect, kind: &EditorKind, host: &dyn AreaHost) {
    ui.painter().rect_filled(rect, 0.0, ui.visuals().faint_bg_color);
    let mut strip = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink2(egui::vec2(6.0, 0.0)))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    strip.label(egui::RichText::new(host.editor_name(kind)).small().strong());
}
