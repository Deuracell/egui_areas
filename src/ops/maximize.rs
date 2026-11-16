use crate::{NodeId, OpError, Workspace};

/// Makes `area` fill the workspace until restored.
pub fn maximize_area(ws: &mut Workspace, area: NodeId) -> Result<(), OpError> {
    if !ws.tree.contains(area) {
        return Err(OpError::NotFound(area));
    }
    if ws.tree.area(area).is_none() {
        return Err(OpError::NotAnArea(area));
    }
    ws.maximized = Some(area);
    Ok(())
}

pub fn restore_area(ws: &mut Workspace) {
    ws.maximized = None;
}

/// Maximizes `area`, or restores if it is already the maximized one.
pub fn toggle_maximize(ws: &mut Workspace, area: NodeId) -> Result<(), OpError> {
    if ws.maximized() == Some(area) {
        restore_area(ws);
        Ok(())
    } else {
        maximize_area(ws, area)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Axis, EditorKind, UnitRect, AreaTree, ops::{close_area, split_area}};

    fn ws() -> (Workspace, NodeId, NodeId) {
        let mut tree = AreaTree::single(EditorKind::new("a"));
        let a = tree.root();
        let b = split_area(&mut tree, a, Axis::Horizontal, 0.5, EditorKind::new("b")).unwrap();
        (Workspace::new("w", tree), a, b)
    }

    #[test]
    fn maximized_area_fills_bounds_and_restores() {
        let (mut w, a, _) = ws();
        assert_eq!(w.visible_areas(UnitRect::UNIT).len(), 2);
        maximize_area(&mut w, a).unwrap();
        assert_eq!(w.visible_areas(UnitRect::UNIT), vec![(a, UnitRect::UNIT)]);
        toggle_maximize(&mut w, a).unwrap();
        assert_eq!(w.visible_areas(UnitRect::UNIT).len(), 2);
    }

    #[test]
    fn closing_the_maximized_area_ends_maximization() {
        let (mut w, a, b) = ws();
        maximize_area(&mut w, b).unwrap();
        close_area(&mut w.tree, b).unwrap();
        assert_eq!(w.maximized(), None);
        assert_eq!(w.visible_areas(UnitRect::UNIT), vec![(a, UnitRect::UNIT)]);
    }

    #[test]
    fn maximizing_a_split_is_refused() {
        let (mut w, _, _) = ws();
        let root = w.tree.root();
        assert_eq!(maximize_area(&mut w, root), Err(OpError::NotAnArea(root)));
    }
}
