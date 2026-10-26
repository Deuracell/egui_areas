use super::pixels::to_pixels;
use crate::layout::{SplitRect, split_rects};
use crate::ops::resize_split;
use crate::{Axis, Rect, Tree};

/// Space between neighbouring areas; also the width of a divider's grab zone.
pub(super) const GAP: f32 = 4.0;
/// Smallest extent an area may be dragged down to.
const MIN_AREA_PX: f32 = 80.0;

fn divider_rect(s: &SplitRect, bounds: egui::Rect) -> egui::Rect {
    let parent = to_pixels(s.rect, bounds);
    match s.axis {
        Axis::Horizontal => {
            let x = parent.min.x + parent.width() * s.ratio;
            egui::Rect::from_min_max(egui::pos2(x - GAP / 2.0, parent.min.y), egui::pos2(x + GAP / 2.0, parent.max.y))
        }
        Axis::Vertical => {
            let y = parent.min.y + parent.height() * s.ratio;
            egui::Rect::from_min_max(egui::pos2(parent.min.x, y - GAP / 2.0), egui::pos2(parent.max.x, y + GAP / 2.0))
        }
    }
}

/// Handles divider drags, applying them to the tree before the areas are laid out.
pub(super) fn interact_dividers(ui: &mut egui::Ui, tree: &mut Tree, bounds: egui::Rect) {
    for s in split_rects(tree, Rect::UNIT) {
        let response = ui.interact(divider_rect(&s, bounds), ui.id().with(("divider", s.id)), egui::Sense::drag());
        let icon = match s.axis {
            Axis::Horizontal => egui::CursorIcon::ResizeHorizontal,
            Axis::Vertical => egui::CursorIcon::ResizeVertical,
        };
        if response.hovered() || response.dragged() {
            ui.ctx().set_cursor_icon(icon);
        }
        let (Some(pos), true) = (response.interact_pointer_pos(), response.dragged()) else { continue };

        let parent = to_pixels(s.rect, bounds);
        let (offset, extent) = match s.axis {
            Axis::Horizontal => (pos.x - parent.min.x, parent.width()),
            Axis::Vertical => (pos.y - parent.min.y, parent.height()),
        };
        let mut ratio = offset / extent;
        if extent > 2.0 * MIN_AREA_PX {
            ratio = ratio.clamp(MIN_AREA_PX / extent, 1.0 - MIN_AREA_PX / extent);
        }
        let _ = resize_split(tree, s.id, ratio); // only fails on NaN pointer positions
    }
}

/// Highlights the divider under the pointer.
pub(super) fn paint_dividers(ui: &mut egui::Ui, tree: &Tree, bounds: egui::Rect) {
    for s in split_rects(tree, Rect::UNIT) {
        let rect = divider_rect(&s, bounds);
        let id = ui.id().with(("divider", s.id));
        if ui.ctx().is_being_dragged(id) || ui.rect_contains_pointer(rect) {
            ui.painter().rect_filled(rect, 1.0, ui.visuals().selection.bg_fill);
        }
    }
}
