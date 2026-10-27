//! Blender-style corner gestures: drag inward to split, outward to join,
//! with Ctrl to swap. Esc cancels.

use super::action::AreaAction;
use super::corner::Corner;
use super::gesture::{Gesture, Input, Neighbour, Side, classify};
use super::hint::{DragMode, TilingHint};
use super::overlay::paint_gesture;
use super::pixels::{CORNER_HOTSPOT, to_pixels};
use super::snap::divider_coords;
use super::dividers::GAP;
use crate::layout::area_rects;
use crate::ops::absorb_across;
use crate::{Axis, NodeId, Rect, Workspace};


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

/// Takes Esc away from the areas while a corner gesture is under way and marks
/// the gesture cancelled. Call before the areas are drawn: editors read keys
/// while drawing, and Esc must not also cancel a tool they have in progress.
pub(super) fn cancel_on_escape(ui: &egui::Ui) {
    let Some(mut drag) = ui.data(|d| d.get_temp::<Drag>(memory_id())) else { return };
    if !drag.cancelled && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
        drag.cancelled = true;
        ui.data_mut(|d| d.insert_temp(memory_id(), drag));
    }
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
    // A gesture whose area vanished mid-drag (layout switched, area closed) is over.
    let mut drag: Option<Drag> = ui.data(|d| d.get_temp::<Drag>(memory_id())).filter(|d| rect_of(areas, d.area).is_some());

    for &(area, rect) in areas {
        for corner in Corner::ALL {
            let id = ui.id().with(("corner", area, corner));
            let response = ui.interact(corner.hotspot(rect, CORNER_HOTSPOT), id, egui::Sense::drag());
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
                Gesture::PopOut => DragMode::PopOut,
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
    let coords = |axis| divider_coords(&ws.tree, bounds, axis, None);
    let (snap_x, snap_y) = (coords(Axis::Horizontal), coords(Axis::Vertical));
    let gesture = classify(&Input {
        area,
        corner: drag.corner,
        delta: pointer - drag.start,
        pointer,
        ctrl: ui.input(|i| i.modifiers.ctrl),
        shift: ui.input(|i| i.modifiers.shift),
        neighbours: &neighbours_at(areas, drag.area, drag.corner),
        hovered_other: areas.iter().find(|&&(a, r)| a != drag.area && r.contains(pointer)).copied(),
        snap_x: &snap_x,
        snap_y: &snap_y,
    });
    match gesture {
        Gesture::Join { target, target_rect, .. } => match join_result(ws, bounds, drag.area, target) {
            Some(result) => Gesture::Join { target, target_rect, result },
            None => Gesture::Idle,
        },
        other => other,
    }
}

/// Where the area covering the joined stretch would end up, found by trying the
/// join on a copy. `None` if the join would be refused.
fn join_result(ws: &Workspace, bounds: egui::Rect, area: NodeId, target: NodeId) -> Option<egui::Rect> {
    let mut trial = ws.tree.clone();
    let survivor = absorb_across(&mut trial, area, target).ok()?;
    let (_, rect) = area_rects(&trial, Rect::UNIT).into_iter().find(|&(id, _)| id == survivor)?;
    Some(to_pixels(rect, bounds).shrink(GAP / 2.0))
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
        Gesture::Join { target, .. } => Some(AreaAction::JoinAcross { area, target }),
        Gesture::Swap { target, .. } => Some(AreaAction::SwapWith { area, other: target }),
        Gesture::PopOut => Some(AreaAction::PopOut { area }),
    }
}

/// How far outside the corner (and inside along the edge) a neighbour is looked for.
const PROBE: f32 = 6.0;

/// The areas just outside `corner` of `area`: the one beyond its horizontal
/// edge and the one beyond its vertical edge, wherever they are and whatever
/// their size.
fn neighbours_at(areas: &[(NodeId, egui::Rect)], area: NodeId, corner: Corner) -> Vec<Neighbour> {
    let Some(rect) = rect_of(areas, area) else { return Vec::new() };
    let (ix, iy) = (corner.inward_x(), corner.inward_y());
    let at = egui::pos2(if ix > 0.0 { rect.min.x } else { rect.max.x }, if iy > 0.0 { rect.min.y } else { rect.max.y });
    let probes = [
        (at + egui::vec2(-ix * PROBE, iy * PROBE), if ix > 0.0 { Side::Left } else { Side::Right }),
        (at + egui::vec2(ix * PROBE, -iy * PROBE), if iy > 0.0 { Side::Top } else { Side::Bottom }),
    ];
    probes
        .into_iter()
        .filter_map(|(p, side)| {
            let &(id, rect) = areas.iter().find(|&&(a, r)| a != area && r.contains(p))?;
            Some(Neighbour { id, rect, side })
        })
        .collect()
}
