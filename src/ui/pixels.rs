use crate::Rect;

/// Maps a unit-space rectangle into `bounds`.
pub(super) fn to_pixels(r: Rect, bounds: egui::Rect) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(bounds.min.x + r.x * bounds.width(), bounds.min.y + r.y * bounds.height()),
        egui::vec2(r.w * bounds.width(), r.h * bounds.height()),
    )
}
