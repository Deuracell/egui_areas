//! Blender-style corner gestures: drag inward to split, outward to join,
//! with Ctrl to swap. Esc cancels.

use super::action::AreaAction;
use super::corner::Corner;
use super::gesture::{Gesture, Input, Neighbour, Side, classify};
use super::hint::{DragMode, TilingHint};
use super::overlay::paint_gesture;
use super::pixels::to_pixels;
use super::snap::divider_coords;
use crate::layout::split_rects;
use crate::{Axis, Node, NodeId, Rect, Workspace};

/// Side of the square grab zone at each area corner.
const HOTSPOT: f32 = 12.0;

#[derive(Clone, Copy)]
struct Drag {
    area: NodeId,
    corner: Corner,
    start: egui::Pos2,
    cancelled: bool,
}

fn memory_id() -> egui::Id {
    egui::Id::new("workspace_corner_drag")
}

/// Registers corner grab zones, runs the active gesture, and queues its result.
/// Call after the areas are drawn so the zones sit on top of their contents.
/// Returns what the status bar should tell the user, if anything.
pub(super) fn handle_corner_drags(
    ui: &mut egui::Ui,
    ws: &Workspace,
    bounds: egui::Rect,
    areas: &[(NodeId, egui::Rect)],
    actions: &mut Vec<AreaAction>,
) -> Option<TilingHint> {
    let mut hovering = false;
    let mut drag: Option<Drag> = ui.data(|d| d.get_temp(memory_id()));

    for &(area, rect) in areas {
        for corner in Corner::ALL {
            let id = ui.id().with(("corner", area, corner));
            let response = ui.interact(corner.hotspot(rect, HOTSPOT), id, egui::Sense::drag());
            if response.hovered() || response.dragged() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
                hovering = true;
            }
            if response.drag_started() {
                let start = response.interact_pointer_pos().unwrap_or(rect.center());
                drag = Some(Drag { area, corner, start, cancelled: false });
            }
            if response.drag_stopped() {
                if let Some(d) = drag.filter(|d| d.area == area && d.corner == corner && !d.cancelled) {
                    if let Some(action) = resolve(ui, ws, bounds, areas, &d) {
                        actions.push(action);
                    }
                }
                drag = None;
            }
        }
    }

    let mut hint = hovering.then_some(TilingHint::Corner);
    if let Some(d) = drag.as_mut() {
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            d.cancelled = true;
        }
        if d.cancelled {
            hint = None;
        } else {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
            let gesture = current_gesture(ui, ws, bounds, areas, d);
            if let Some(source) = rect_of(areas, d.area) {
                paint_gesture(ui, bounds, source, &gesture);
            }
            let ctrl = ui.input(|i| i.modifiers.ctrl);
            hint = Some(TilingHint::Dragging(match gesture {
                Gesture::Idle if ctrl => DragMode::SwapPending,
                Gesture::Idle => DragMode::Undecided,
                Gesture::Split { .. } => DragMode::Split,
                Gesture::Join { .. } => DragMode::Join,
                Gesture::Swap { .. } => DragMode::Swap,
            }));
        }
    }
    ui.data_mut(|data| match drag {
        Some(d) => data.insert_temp(memory_id(), d),
        None => data.remove::<Drag>(memory_id()),
    });
    hint
}

fn rect_of(areas: &[(NodeId, egui::Rect)], id: NodeId) -> Option<egui::Rect> {
    areas.iter().find(|&&(a, _)| a == id).map(|&(_, r)| r)
}

fn current_gesture(
    ui: &egui::Ui,
    ws: &Workspace,
    bounds: egui::Rect,
    areas: &[(NodeId, egui::Rect)],
    drag: &Drag,
) -> Gesture {
    let Some(area) = rect_of(areas, drag.area) else { return Gesture::Idle };
    let pointer = ui.ctx().pointer_latest_pos().unwrap_or(drag.start);
    let snapping = !ui.input(|i| i.modifiers.shift);
    let coords = |axis| if snapping { divider_coords(&ws.tree, bounds, axis, None) } else { Vec::new() };
    let (snap_x, snap_y) = (coords(Axis::Horizontal), coords(Axis::Vertical));
    classify(&Input {
        area,
        corner: drag.corner,
        delta: pointer - drag.start,
        pointer,
        ctrl: ui.input(|i| i.modifiers.ctrl),
        neighbour: neighbour_of(ws, drag.area, bounds),
        hovered_other: areas.iter().find(|&&(a, r)| a != drag.area && r.contains(pointer)).copied(),
        snap_x: &snap_x,
        snap_y: &snap_y,
    })
}

fn resolve(
    ui: &egui::Ui,
    ws: &Workspace,
    bounds: egui::Rect,
    areas: &[(NodeId, egui::Rect)],
    drag: &Drag,
) -> Option<AreaAction> {
    let area = drag.area;
    match current_gesture(ui, ws, bounds, areas, drag) {
        Gesture::Idle => None,
        Gesture::Split { axis, ratio, .. } => Some(AreaAction::Split { area, axis, ratio }),
        Gesture::Join { .. } => Some(AreaAction::AbsorbNeighbour { area }),
        Gesture::Swap { target, .. } => Some(AreaAction::SwapWith { area, other: target }),
    }
}

/// The area's sibling in the tree, where it is on screen, and which side of the area it touches.
/// Only a single area counts: absorbing a whole group of areas would silently
/// remove several editors at once.
fn neighbour_of(ws: &Workspace, area: NodeId, bounds: egui::Rect) -> Option<Neighbour> {
    let parent = ws.tree.parent(area)?;
    let Node::Split { axis, first, .. } = ws.tree.node(parent)? else { return None };
    let sibling = ws.tree.sibling(area)?;
    ws.tree.area(sibling)?;
    let is_first = *first == area;
    let split = split_rects(&ws.tree, Rect::UNIT).into_iter().find(|s| s.id == parent)?;
    let (r1, r2) = split.rect.split(split.axis, split.ratio);
    let side = match (axis, is_first) {
        (Axis::Horizontal, true) => Side::Right,
        (Axis::Horizontal, false) => Side::Left,
        (Axis::Vertical, true) => Side::Bottom,
        (Axis::Vertical, false) => Side::Top,
    };
    Some(Neighbour {
        id: sibling,
        rect: to_pixels(if is_first { r2 } else { r1 }, bounds),
        side,
    })
}
