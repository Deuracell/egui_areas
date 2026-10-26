use super::pixels::to_pixels;
use crate::{NodeId, Rect, Workspace};

/// The area the user is working in: the one under the pointer, staying put
/// while the pointer is elsewhere (menu bar, status bar, another window).
/// One source of truth for shortcut routing and context-sensitive UI.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ActiveArea(Option<NodeId>);

impl ActiveArea {
    pub fn get(&self) -> Option<NodeId> {
        self.0
    }

    pub fn is(&self, area: NodeId) -> bool {
        self.0 == Some(area)
    }

    /// Call once per frame, before the areas are drawn, so every area sees the
    /// same answer. `bounds` is where the workspace is drawn.
    pub(super) fn update(&mut self, ws: &Workspace, bounds: egui::Rect, pointer: Option<egui::Pos2>) {
        if let Some(p) = pointer {
            let hit = ws.visible_areas(Rect::UNIT).into_iter().find(|&(_, r)| to_pixels(r, bounds).contains(p));
            if let Some((id, _)) = hit {
                self.0 = Some(id);
            }
        }
        if self.0.is_some_and(|id| ws.tree.area(id).is_none()) {
            self.0 = None; // the active area was closed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::{close_area, maximize_area, split_area};
    use crate::{Axis, EditorKind, Tree};

    fn setup() -> (Workspace, NodeId, NodeId, egui::Rect) {
        let mut tree = Tree::single(EditorKind::new("a"));
        let a = tree.root();
        let b = split_area(&mut tree, a, Axis::Horizontal, 0.5, EditorKind::new("b")).unwrap();
        let bounds = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 100.0));
        (Workspace::new("w", tree), a, b, bounds)
    }

    #[test]
    fn follows_pointer_and_sticks_when_it_leaves() {
        let (ws, a, b, bounds) = setup();
        let mut active = ActiveArea::default();
        active.update(&ws, bounds, Some(egui::pos2(10.0, 50.0)));
        assert!(active.is(a));
        active.update(&ws, bounds, Some(egui::pos2(90.0, 50.0)));
        assert!(active.is(b));
        active.update(&ws, bounds, Some(egui::pos2(500.0, 500.0)));
        assert!(active.is(b), "pointer outside the workspace keeps the last area");
        active.update(&ws, bounds, None);
        assert!(active.is(b));
    }

    #[test]
    fn closed_area_is_dropped() {
        let (mut ws, _, b, bounds) = setup();
        let mut active = ActiveArea::default();
        active.update(&ws, bounds, Some(egui::pos2(90.0, 50.0)));
        close_area(&mut ws.tree, b).unwrap();
        active.update(&ws, bounds, None);
        assert_eq!(active.get(), None);
    }

    #[test]
    fn maximized_area_takes_the_whole_workspace() {
        let (mut ws, a, b, bounds) = setup();
        maximize_area(&mut ws, b).unwrap();
        let mut active = ActiveArea::default();
        active.update(&ws, bounds, Some(egui::pos2(10.0, 50.0)));
        assert!(active.is(b) && !active.is(a));
    }
}
