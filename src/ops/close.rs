use crate::{NodeId, OpError, Tree};

/// Removes `area`; its sibling (a whole subtree if the sibling is itself split)
/// takes over the freed space. Returns the sibling's id.
pub fn close_area(tree: &mut Tree, area: NodeId) -> Result<NodeId, OpError> {
    if !tree.contains(area) {
        return Err(OpError::NotFound(area));
    }
    if tree.area(area).is_none() {
        return Err(OpError::NotAnArea(area));
    }
    let parent = tree.parent(area).ok_or(OpError::LastArea)?;
    let sibling = tree.sibling(area).expect("a child of a split has a sibling");
    let grandparent = tree.parent(parent);

    tree.set_parent(sibling, grandparent);
    match grandparent {
        Some(g) => tree.replace_child(g, parent, sibling),
        None => tree.set_root(sibling),
    }
    tree.remove(area);
    tree.remove(parent);
    Ok(sibling)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Axis, EditorKind, ops::split_area};

    fn k() -> EditorKind {
        EditorKind::new("x")
    }

    #[test]
    fn closing_gives_space_to_sibling() {
        let mut tree = Tree::single(k());
        let a = tree.root();
        let b = split_area(&mut tree, a, Axis::Horizontal, 0.5, k()).unwrap();
        assert_eq!(close_area(&mut tree, b), Ok(a));
        tree.assert_valid();
        assert_eq!(tree.root(), a);
    }

    #[test]
    fn closing_drops_the_view() {
        let mut tree = Tree::single(k());
        let a = tree.root();
        let b = split_area(&mut tree, a, Axis::Horizontal, 0.5, k()).unwrap();
        let (view_a, view_b) = (tree.area(a).unwrap().view, tree.area(b).unwrap().view);
        close_area(&mut tree, b).unwrap();
        assert_eq!(tree.view_ids(), vec![view_a]);
        assert!(!tree.view_ids().contains(&view_b));
    }

    #[test]
    fn sibling_subtree_expands() {
        let mut tree = Tree::single(k());
        let a = tree.root();
        let b = split_area(&mut tree, a, Axis::Horizontal, 0.5, k()).unwrap();
        let c = split_area(&mut tree, b, Axis::Vertical, 0.5, k()).unwrap();
        close_area(&mut tree, a).unwrap();
        tree.assert_valid();
        assert_eq!(tree.areas(), vec![b, c]);
    }

    #[test]
    fn first_area_descends_to_the_left_top_leaf() {
        let mut tree = Tree::single(k());
        let a = tree.root();
        let b = split_area(&mut tree, a, Axis::Horizontal, 0.5, k()).unwrap();
        split_area(&mut tree, b, Axis::Vertical, 0.5, k()).unwrap();
        assert_eq!(tree.first_area(tree.root()), Some(a));
        assert_eq!(tree.first_area(tree.parent(b).unwrap()), Some(b));
        assert_eq!(tree.first_area(b), Some(b));
    }

    #[test]
    fn last_area_cannot_be_closed() {
        let mut tree = Tree::single(k());
        let a = tree.root();
        assert_eq!(close_area(&mut tree, a), Err(OpError::LastArea));
        tree.assert_valid();
    }

    #[test]
    fn closing_a_split_node_is_refused() {
        let mut tree = Tree::single(k());
        let a = tree.root();
        split_area(&mut tree, a, Axis::Horizontal, 0.5, k()).unwrap();
        let root = tree.root();
        assert_eq!(close_area(&mut tree, root), Err(OpError::NotAnArea(root)));
    }
}
