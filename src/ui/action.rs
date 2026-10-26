use crate::ops::{close_area, join_areas, set_area_kind, split_area, toggle_maximize};
use crate::{Axis, EditorKind, NodeId, OpError, Workspace};

/// Something the user asked for through an area header. Collected while
/// drawing and applied afterwards, so the tree is never edited mid-draw.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum AreaAction {
    SetKind { area: NodeId, kind: EditorKind },
    /// Split in two; the new area shows the same editor as `area`.
    Split { area: NodeId, axis: Axis },
    /// Remove `area`; its neighbour expands into the space.
    MergeIntoNeighbour { area: NodeId },
    /// Remove `area`'s neighbour; `area` expands into the space.
    AbsorbNeighbour { area: NodeId },
    ToggleMaximize { area: NodeId },
}

pub(super) fn apply(ws: &mut Workspace, action: AreaAction) -> Result<(), OpError> {
    match action {
        AreaAction::SetKind { area, kind } => set_area_kind(&mut ws.tree, area, kind),
        AreaAction::Split { area, axis } => {
            let kind = ws.tree.area(area).ok_or(OpError::NotAnArea(area))?.kind.clone();
            split_area(&mut ws.tree, area, axis, 0.5, kind).map(|_| ())
        }
        AreaAction::MergeIntoNeighbour { area } => close_area(&mut ws.tree, area).map(|_| ()),
        AreaAction::AbsorbNeighbour { area } => {
            let neighbour = ws.tree.sibling(area).ok_or(OpError::LastArea)?;
            join_areas(&mut ws.tree, area, neighbour)
        }
        AreaAction::ToggleMaximize { area } => toggle_maximize(ws, area),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ws() -> (Workspace, NodeId) {
        let tree = crate::Tree::single(EditorKind::new("sketch"));
        let a = tree.root();
        (Workspace::new("w", tree), a)
    }

    #[test]
    fn split_duplicates_the_editor_and_absorb_undoes_it() {
        let (mut w, a) = ws();
        apply(&mut w, AreaAction::Split { area: a, axis: Axis::Horizontal }).unwrap();
        let kinds: Vec<_> = w.tree.areas().iter().map(|&i| w.tree.area(i).unwrap().kind.id().to_string()).collect();
        assert_eq!(kinds, ["sketch", "sketch"]);
        apply(&mut w, AreaAction::AbsorbNeighbour { area: a }).unwrap();
        assert_eq!(w.tree.areas(), vec![a]);
    }

    #[test]
    fn lone_area_cannot_merge_or_absorb() {
        let (mut w, a) = ws();
        assert_eq!(apply(&mut w, AreaAction::MergeIntoNeighbour { area: a }), Err(OpError::LastArea));
        assert_eq!(apply(&mut w, AreaAction::AbsorbNeighbour { area: a }), Err(OpError::LastArea));
    }
}
