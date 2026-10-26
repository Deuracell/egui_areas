use super::area::draw_area;
use super::dividers::{GAP, interact_dividers, paint_dividers};
use super::host::AreaHost;
use super::pixels::to_pixels;
use crate::{Rect, Workspace};

/// Draws the whole workspace into the space `ui` has left: the maximized area
/// alone, or every area with draggable dividers between them.
pub fn show_workspace(ui: &mut egui::Ui, workspace: &mut Workspace, host: &mut dyn AreaHost) {
    let bounds = ui.available_rect_before_wrap();
    let maximized = workspace.maximized().is_some();

    if !maximized {
        interact_dividers(ui, &mut workspace.tree, bounds);
    }
    for (id, unit) in workspace.visible_areas(Rect::UNIT) {
        let Some(kind) = workspace.tree.area(id).map(|a| a.kind.clone()) else { continue };
        let rect = to_pixels(unit, bounds).shrink(GAP / 2.0);
        draw_area(ui, rect, id, &kind, host);
    }
    if !maximized {
        paint_dividers(ui, &workspace.tree, bounds);
    }
}
