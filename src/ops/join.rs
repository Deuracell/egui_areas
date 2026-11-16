use super::close_area;
use crate::{NodeId, OpError, AreaTree};

/// Merges `remove` into `keep`: `keep` absorbs the space and contents stay as
/// they were. Only the two halves of one split can be joined; areas that merely
/// touch across different splits cannot.
pub fn join_areas(tree: &mut AreaTree, keep: NodeId, remove: NodeId) -> Result<(), OpError> {
    if keep == remove {
        return Err(OpError::SameArea);
    }
    for id in [keep, remove] {
        if !tree.contains(id) {
            return Err(OpError::NotFound(id));
        }
        if tree.area(id).is_none() {
            return Err(OpError::NotAnArea(id));
        }
    }
    if tree.sibling(keep) != Some(remove) {
        return Err(OpError::NotSiblings);
    }
    close_area(tree, remove).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Axis, EditorKind, ops::split_area};

    fn k() -> EditorKind {
        EditorKind::new("x")
    }

    #[test]
    fn siblings_join() {
        let mut tree = AreaTree::single(k());
        let a = tree.root();
        let b = split_area(&mut tree, a, Axis::Horizontal, 0.5, k()).unwrap();
        join_areas(&mut tree, a, b).unwrap();
        tree.assert_valid();
        assert_eq!(tree.areas(), vec![a]);
    }

    #[test]
    fn non_siblings_are_refused_and_tree_unchanged() {
        let mut tree = AreaTree::single(k());
        let a = tree.root();
        let b = split_area(&mut tree, a, Axis::Horizontal, 0.5, k()).unwrap();
        let c = split_area(&mut tree, b, Axis::Vertical, 0.5, k()).unwrap();
        assert_eq!(join_areas(&mut tree, a, c), Err(OpError::NotSiblings));
        assert_eq!(join_areas(&mut tree, a, a), Err(OpError::SameArea));
        tree.assert_valid();
        assert_eq!(tree.areas(), vec![a, b, c]);
    }
}
