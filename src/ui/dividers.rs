use super::pixels::{CORNER_HOTSPOT, to_pixels};
use super::snap::{divider_coords, nearest};
use crate::layout::{SplitRect, split_rects};
use crate::ops::resize_split;
use crate::{Axis, NodeId, Rect, Tree};

/// Visible space between neighbouring areas.
pub(super) const GAP: f32 = 2.0;
/// Width of a divider's grab zone: wider than the gap, reaching into the
/// neighbouring areas' edges so the divider is easy to hit.
const HIT_WIDTH: f32 = 6.0;
/// Width of the highlight drawn on a hot divider.
const HIGHLIGHT_WIDTH: f32 = 4.0;
/// Smallest extent an area may be dragged down to.
const MIN_AREA_PX: f32 = 80.0;

/// The strip of `width` centred on the divider of `s`.
fn divider_rect(s: &SplitRect, bounds: egui::Rect, width: f32) -> egui::Rect {
    let parent = to_pixels(s.rect, bounds);
    let half = width / 2.0;
    match s.axis {
        Axis::Horizontal => {
            let x = parent.min.x + parent.width() * s.ratio;
            egui::Rect::from_min_max(egui::pos2(x - half, parent.min.y), egui::pos2(x + half, parent.max.y))
        }
        Axis::Vertical => {
            let y = parent.min.y + parent.height() * s.ratio;
            egui::Rect::from_min_max(egui::pos2(parent.min.x, y - half), egui::pos2(parent.max.x, y + half))
        }
    }
}

/// Grab zone for a divider. It lives in its own layer above the areas, so it
/// wins over whatever an area draws at its edge.
fn divider_response(ui: &egui::Ui, s: &SplitRect, bounds: egui::Rect) -> egui::Response {
    // Stops short of the ends, where area corners with their own grab zones meet it.
    let full = divider_rect(s, bounds, HIT_WIDTH);
    let hit = match s.axis {
        Axis::Horizontal => full.shrink2(egui::vec2(0.0, CORNER_HOTSPOT)),
        Axis::Vertical => full.shrink2(egui::vec2(CORNER_HOTSPOT, 0.0)),
    };
    egui::Area::new(ui.id().with(("divider_area", s.id)))
        .order(egui::Order::Middle)
        .fixed_pos(hit.min)
        .constrain(false)
        .show(ui.ctx(), |ui| ui.allocate_exact_size(hit.size(), egui::Sense::drag()).1)
        .inner
}

/// A line to draw while a divider is snapped onto another one.
#[derive(Clone, Copy)]
pub(super) struct Guide {
    pub axis: Axis,
    pub coord: f32,
}

#[derive(Default)]
pub(super) struct DividerOutcome {
    pub guides: Vec<Guide>,
    /// The pointer is on, or is dragging, a divider.
    pub engaged: bool,
    /// Dividers to highlight: hovered or being dragged.
    pub hot: Vec<NodeId>,
}

/// Handles divider drags, applying them to the tree before the areas are laid
/// out. Dragged dividers snap to other dividers and to the middle of their
/// parent; hold Shift to turn that off.
pub(super) fn interact_dividers(ui: &mut egui::Ui, tree: &mut Tree, bounds: egui::Rect) -> DividerOutcome {
    let snapping = !ui.input(|i| i.modifiers.shift);
    let mut out = DividerOutcome::default();
    for s in split_rects(tree, Rect::UNIT) {
        let response = divider_response(ui, &s, bounds);
        let icon = match s.axis {
            Axis::Horizontal => egui::CursorIcon::ResizeHorizontal,
            Axis::Vertical => egui::CursorIcon::ResizeVertical,
        };
        if response.hovered() || response.dragged() {
            ui.ctx().set_cursor_icon(icon);
            out.engaged = true;
            out.hot.push(s.id);
        }
        let (Some(pos), true) = (response.interact_pointer_pos(), response.dragged()) else { continue };

        let parent = to_pixels(s.rect, bounds);
        let (origin, extent, pointer) = match s.axis {
            Axis::Horizontal => (parent.min.x, parent.width(), pos.x),
            Axis::Vertical => (parent.min.y, parent.height(), pos.y),
        };
        let mut coord = pointer;
        let mut snapped = None;
        if snapping {
            let targets = divider_coords(tree, bounds, s.axis, Some(s.id));
            snapped = nearest(pointer, targets.into_iter().chain([origin + extent / 2.0]));
            coord = snapped.unwrap_or(pointer);
        }
        let mut ratio = (coord - origin) / extent;
        if extent > 2.0 * MIN_AREA_PX {
            let clamped = ratio.clamp(MIN_AREA_PX / extent, 1.0 - MIN_AREA_PX / extent);
            if clamped != ratio {
                snapped = None; // snapping must not push an area below its minimum
            }
            ratio = clamped;
        }
        let _ = resize_split(tree, s.id, ratio); // only fails on NaN pointer positions
        if let Some(c) = snapped {
            out.guides.push(Guide { axis: s.axis, coord: c });
        }
    }
    out
}

/// Highlights the divider under the pointer, and any snap guides.
pub(super) fn paint_dividers(ui: &mut egui::Ui, tree: &Tree, bounds: egui::Rect, outcome: &DividerOutcome) {
    for s in split_rects(tree, Rect::UNIT).into_iter().filter(|s| outcome.hot.contains(&s.id)) {
        let rect = divider_rect(&s, bounds, HIGHLIGHT_WIDTH);
        ui.painter().rect_filled(rect, 1.0, ui.visuals().selection.bg_fill);
    }
    let stroke = egui::Stroke::new(1.0, ui.visuals().selection.stroke.color);
    for g in &outcome.guides {
        match g.axis {
            Axis::Horizontal => ui.painter().vline(g.coord, bounds.y_range(), stroke),
            Axis::Vertical => ui.painter().hline(bounds.x_range(), g.coord, stroke),
        };
    }
}
