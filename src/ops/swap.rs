use crate::{NodeId, OpError, Tree};

/// Exchanges what two areas show. Positions and ids stay put; only the editors move.
pub fn swap_areas(tree: &mut Tree, a: NodeId, b: NodeId) -> Result<(), OpError> {
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
    let kind_a = tree.area(a).expect("checked").kind.clone();
    let kind_b = std::mem::replace(&mut tree.area_mut(b).expect("checked").kind, kind_a);
    tree.area_mut(a).expect("checked").kind = kind_b;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Axis, EditorKind, ops::split_area};

    #[test]
    fn swap_exchanges_kinds_not_ids() {
        let mut tree = Tree::single(EditorKind::new("sketch"));
        let a = tree.root();
        let b = split_area(&mut tree, a, Axis::Horizontal, 0.5, EditorKind::new("props")).unwrap();
        swap_areas(&mut tree, a, b).unwrap();
        assert_eq!(tree.area(a).unwrap().kind.id(), "props");
        assert_eq!(tree.area(b).unwrap().kind.id(), "sketch");
        assert_eq!(tree.areas(), vec![a, b]);
        assert_eq!(swap_areas(&mut tree, a, a), Err(OpError::SameArea));
    }
}
