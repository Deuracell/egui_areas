use super::action;
use super::active::ActiveArea;
use super::area::draw_area;
use super::dividers::{GAP, interact_dividers, paint_dividers};
use super::header::HeaderInfo;
use super::host::AreaHost;
use super::pixels::to_pixels;
use crate::{Node, NodeId, Rect, Workspace};

/// Draws the whole workspace into the space `ui` has left: the maximized area
/// alone, or every area with draggable dividers between them. `active` is
/// updated from the pointer and tells each editor whether it is the active one.
pub fn show_workspace(
    ui: &mut egui::Ui,
    workspace: &mut Workspace,
    active: &mut ActiveArea,
    host: &mut dyn AreaHost,
) {
    let bounds = ui.available_rect_before_wrap();
    active.update(workspace, bounds, ui.input(|i| i.pointer.hover_pos()));
    let maximized = workspace.maximized().is_some();
    let mut actions = Vec::new();

    if !maximized {
        interact_dividers(ui, &mut workspace.tree, bounds);
    }
    for (id, unit) in workspace.visible_areas(Rect::UNIT) {
        let Some(kind) = workspace.tree.area(id).map(|a| a.kind.clone()) else { continue };
        let info = header_info(workspace, id, &kind);
        let rect = to_pixels(unit, bounds).shrink(GAP / 2.0);
        draw_area(ui, rect, id, &kind, &info, active.is(id), host, &mut actions);
    }
    if !maximized {
        paint_dividers(ui, &workspace.tree, bounds);
    }

    for a in actions {
        // Header menus only offer actions that are valid for the current tree.
        let result = action::apply(workspace, a);
        debug_assert!(result.is_ok(), "area action refused: {result:?}");
    }
}

fn header_info(ws: &Workspace, area: NodeId, kind: &crate::EditorKind) -> HeaderInfo {
    let tree = &ws.tree;
    let sibling = tree.sibling(area);
    HeaderInfo {
        area,
        same_kind_count: tree.areas().iter().filter(|&&a| tree.area(a).is_some_and(|x| &x.kind == kind)).count(),
        can_absorb: sibling.is_some_and(|s| matches!(tree.node(s), Some(Node::Area(_)))),
        can_merge: sibling.is_some(),
        maximized: ws.maximized() == Some(area),
    }
}
