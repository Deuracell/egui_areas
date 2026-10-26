use super::action::AreaAction;
use super::header::{HeaderInfo, draw_header, header_height};
use super::host::AreaHost;
use crate::{EditorKind, NodeId};

/// Draws one area: frame, header, and the editor in the remaining body.
pub(super) fn draw_area(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    id: NodeId,
    kind: &EditorKind,
    info: &HeaderInfo,
    active: bool,
    host: &mut dyn AreaHost,
    actions: &mut Vec<AreaAction>,
) {
    let border = ui.visuals().widgets.noninteractive.bg_stroke;
    ui.painter().rect_filled(rect, super::rounding::radius(), ui.visuals().panel_fill);

    let header = egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), header_height(ui)));
    draw_header(ui, header, kind, info, active, host, actions);

    let body = egui::Rect::from_min_max(egui::pos2(rect.min.x, header.max.y), rect.max);
    let mut body_ui = ui.new_child(egui::UiBuilder::new().id_salt(("area_body", id)).max_rect(body));
    body_ui.set_clip_rect(body.intersect(ui.clip_rect()));
    host.show_editor(&mut body_ui, id, kind, active);

    super::rounding::mask_corners(ui.painter(), rect, super::rounding::backdrop(ui));
    ui.painter().rect_stroke(rect, super::rounding::radius(), border, egui::StrokeKind::Inside);
}
