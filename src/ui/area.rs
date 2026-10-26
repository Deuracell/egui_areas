use super::header::{HEADER_HEIGHT, draw_header};
use super::host::AreaHost;
use crate::{EditorKind, NodeId};

/// Draws one area: frame, header, and the editor in the remaining body.
pub(super) fn draw_area(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    id: NodeId,
    kind: &EditorKind,
    host: &mut dyn AreaHost,
) {
    let border = ui.visuals().widgets.noninteractive.bg_stroke;
    ui.painter().rect_filled(rect, 2.0, ui.visuals().panel_fill);

    let header = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), HEADER_HEIGHT));
    draw_header(ui, header, kind, host);

    let body = egui::Rect::from_min_max(egui::pos2(rect.min.x, header.max.y), rect.max);
    let mut body_ui = ui.new_child(egui::UiBuilder::new().id_salt(("area_body", id)).max_rect(body));
    body_ui.set_clip_rect(body.intersect(ui.clip_rect()));
    host.show_editor(&mut body_ui, id, kind);

    ui.painter().rect_stroke(rect, 2.0, border, egui::StrokeKind::Inside);
}
