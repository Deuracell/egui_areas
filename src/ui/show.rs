use super::action::{self, AreaAction};
use super::active::ActiveArea;
use super::area::draw_area;
use super::corner_drag::handle_corner_drags;
use super::dividers::{GAP, interact_dividers, paint_dividers};
use super::hint::TilingHint;
use super::header::HeaderInfo;
use super::host::AreaHost;
use super::pixels::to_pixels;
use super::shortcuts;
use crate::{Node, NodeId, Rect, Workspace};

/// Draws the whole workspace into the space `ui` has left: the maximized area
/// alone, or every area with draggable dividers between them. `active` is
/// updated from the pointer and tells each editor whether it is the active one.
/// Returns what the status bar should tell the user about tiling.
pub fn show_workspace(
    ui: &mut egui::Ui,
    workspace: &mut Workspace,
    active: &mut ActiveArea,
    host: &mut dyn AreaHost,
) -> TilingHint {
    let bounds = ui.available_rect_before_wrap();
    active.update(workspace, bounds, ui.input(|i| i.pointer.hover_pos()), ui.input(|i| i.pointer.any_down()));
    let maximized = workspace.maximized().is_some();
    let mut actions = Vec::new();
    let mut drawn = Vec::new();

    ui.painter().rect_filled(bounds, 0.0, super::rounding::backdrop(ui));
    let mut hint = if maximized { TilingHint::Maximized } else { TilingHint::None };
    let mut dividers = Default::default();
    if !maximized {
        dividers = interact_dividers(ui, &mut workspace.tree, bounds);
        if dividers.engaged {
            hint = TilingHint::Divider;
        }
    }
    for (id, unit) in workspace.visible_areas(Rect::UNIT) {
        let Some(kind) = workspace.tree.area(id).map(|a| a.kind.clone()) else { continue };
        let info = header_info(workspace, id, &kind);
        let rect = to_pixels(unit, bounds).shrink(GAP / 2.0);
        draw_area(ui, rect, id, &kind, &info, active.is(id), host, &mut actions);
        drawn.push((id, rect));
    }
    active.set_held(active.get().is_some_and(|a| host.holds_focus(a)));
    if !maximized {
        paint_dividers(ui, &workspace.tree, bounds, &dividers);
        if let Some(h) = handle_corner_drags(ui, workspace, bounds, &drawn, &mut actions) {
            hint = h;
        }
    }
    shortcuts::collect(ui, active, &mut actions);

    for a in actions {
        // When the active area is closed, focus goes to whoever takes its space.
        let heir = match &a {
            AreaAction::MergeIntoNeighbour { area } if active.is(*area) => {
                workspace.tree.sibling(*area).and_then(|s| workspace.tree.first_area(s))
            }
            _ => None,
        };
        // Header menus only offer actions that are valid for the current tree.
        let result = action::apply(workspace, a);
        debug_assert!(result.is_ok(), "area action refused: {result:?}");
        if let (Ok(()), Some(heir)) = (result, heir) {
            active.set(heir);
        }
    }
    hint
}

fn header_info(ws: &Workspace, area: NodeId, kind: &crate::EditorKind) -> HeaderInfo {
    let tree = &ws.tree;
    let sibling = tree.sibling(area);
    HeaderInfo {
        area,
        view: tree.area(area).expect("drawn nodes are areas").view,
        same_kind_count: tree.areas().iter().filter(|&&a| tree.area(a).is_some_and(|x| &x.kind == kind)).count(),
        can_absorb: sibling.is_some_and(|s| matches!(tree.node(s), Some(Node::Area(_)))),
        can_merge: sibling.is_some(),
        maximized: ws.maximized() == Some(area),
    }
}
