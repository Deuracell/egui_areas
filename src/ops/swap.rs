use crate::{NodeId, OpError, AreaTree};

/// Exchanges what two areas show, view state included. Positions and ids stay
/// put; only the editors (with their views) move.
pub fn swap_areas(tree: &mut AreaTree, a: NodeId, b: NodeId) -> Result<(), OpError> {
    if a == b {
        return Err(OpError::SameArea);
    }
    for id in [a, b] {
        if !tree.contains(id) {
            return Err(OpError::NotFound(id));
        }
        if tree.area(id).is_none() {
            return Err(OpError::NotAnArea(id));
        }
    }
    let area_a = tree.area(a).expect("checked").clone();
    let area_b = std::mem::replace(tree.area_mut(b).expect("checked"), area_a);
    *tree.area_mut(a).expect("checked") = area_b;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Axis, EditorKind, ops::split_area};

    #[test]
    fn swap_exchanges_kinds_not_ids() {
        let mut tree = AreaTree::single(EditorKind::new("sketch"));
        let a = tree.root();
        let b = split_area(&mut tree, a, Axis::Horizontal, 0.5, EditorKind::new("props")).unwrap();
        let (view_a, view_b) = (tree.area(a).unwrap().view, tree.area(b).unwrap().view);
        swap_areas(&mut tree, a, b).unwrap();
        assert_eq!(tree.area(a).unwrap().view, view_b, "views travel with their editor");
        assert_eq!(tree.area(b).unwrap().view, view_a);
        assert_eq!(tree.area(a).unwrap().kind.id(), "props");
        assert_eq!(tree.area(b).unwrap().kind.id(), "sketch");
        assert_eq!(tree.areas(), vec![a, b]);
        assert_eq!(swap_areas(&mut tree, a, a), Err(OpError::SameArea));
    }
}
