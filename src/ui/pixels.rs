use crate::UnitRect;

/// Maps a unit-space rectangle into `bounds`.
pub(super) fn to_pixels(r: UnitRect, bounds: egui::Rect) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(bounds.min.x + r.x * bounds.width(), bounds.min.y + r.y * bounds.height()),
        egui::vec2(r.w * bounds.width(), r.h * bounds.height()),
    )
}

/// Side of the square grab zone at each area corner. Dividers stop this far
/// short of their ends so the corners beside them stay grabbable.
pub(super) const CORNER_HOTSPOT: f32 = 12.0;
