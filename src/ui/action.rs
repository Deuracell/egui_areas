use crate::ops::{absorb_across, set_area_kind, split_area, swap_areas, toggle_maximize};
use crate::{Axis, EditorKind, NodeId, OpError, Workspace};

/// Something the user asked for through an area header. Collected while
/// drawing and applied afterwards, so the tree is never edited mid-draw.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum AreaAction {
    SetKind { area: NodeId, kind: EditorKind },
    /// Split in two at `ratio`; the new area shows the same editor as `area`.
    Split { area: NodeId, axis: Axis, ratio: f32 },
    /// Exchange what two areas show.
    SwapWith { area: NodeId, other: NodeId },
    /// `area` takes over the neighbour `target`'s space where they face each
    /// other (see [`absorb_across`]).
    JoinAcross { area: NodeId, target: NodeId },
    ToggleMaximize { area: NodeId },
    /// Show a copy of `area`'s editor in a new window. Changes no tree: the
    /// host opens the window.
    PopOut { area: NodeId },
}

/// Applies `action`. On success returns the area that should take focus, if the
/// action changed which areas exist.
pub(super) fn apply(ws: &mut Workspace, action: AreaAction) -> Result<Option<NodeId>, OpError> {
    match action {
        AreaAction::SetKind { area, kind } => set_area_kind(&mut ws.tree, area, kind).map(|_| None),
        AreaAction::Split { area, axis, ratio } => {
            let kind = ws.tree.area(area).ok_or(OpError::NotAnArea(area))?.kind.clone();
            split_area(&mut ws.tree, area, axis, ratio, kind).map(|_| None)
        }
        AreaAction::SwapWith { area, other } => swap_areas(&mut ws.tree, area, other).map(|_| None),
        AreaAction::JoinAcross { area, target } => absorb_across(&mut ws.tree, area, target).map(Some),
        AreaAction::ToggleMaximize { area } => toggle_maximize(ws, area).map(|_| None),
        AreaAction::PopOut { .. } => Ok(None),
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
        apply(&mut w, AreaAction::Split { area: a, axis: Axis::Horizontal, ratio: 0.5 }).unwrap();
        let kinds: Vec<_> = w.tree.areas().iter().map(|&i| w.tree.area(i).unwrap().kind.id().to_string()).collect();
        assert_eq!(kinds, ["sketch", "sketch"]);
        let b = w.tree.areas()[1];
        let survivor = apply(&mut w, AreaAction::JoinAcross { area: a, target: b }).unwrap().unwrap();
        assert_eq!(w.tree.areas(), vec![survivor]);
    }

    #[test]
    fn swap_exchanges_editors() {
        let (mut w, a) = ws();
        apply(&mut w, AreaAction::Split { area: a, axis: Axis::Vertical, ratio: 0.3 }).unwrap();
        let b = w.tree.areas()[1];
        crate::ops::set_area_kind(&mut w.tree, b, EditorKind::new("props")).unwrap();
        apply(&mut w, AreaAction::SwapWith { area: a, other: b }).unwrap();
        assert_eq!(w.tree.area(a).unwrap().kind.id(), "props");
    }

    #[test]
    fn an_area_cannot_join_itself() {
        let (mut w, a) = ws();
        assert_eq!(apply(&mut w, AreaAction::JoinAcross { area: a, target: a }), Err(OpError::SameArea));
    }
}
