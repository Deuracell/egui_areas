use super::clamp_ratio;
use crate::{Area, Axis, EditorKind, Node, NodeId, OpError, AreaTree};

/// Splits `area` in two along `axis`. `area` keeps its id and becomes the
/// first (left/top) half, taking `ratio` of the space; the new area, showing
/// `new_kind` with a fresh view, is returned.
pub fn split_area(
    tree: &mut AreaTree,
    area: NodeId,
    axis: Axis,
    ratio: f32,
    new_kind: EditorKind,
) -> Result<NodeId, OpError> {
    if !tree.contains(area) {
        return Err(OpError::NotFound(area));
    }
    if tree.area(area).is_none() {
        return Err(OpError::NotAnArea(area));
    }
    let ratio = clamp_ratio(ratio)?;

    let old_parent = tree.parent(area);
    let view = tree.new_view();
    let new_leaf = tree.insert(Node::Area(Area { kind: new_kind, view }), None);
    let split = tree.insert(Node::Split { axis, ratio, first: area, second: new_leaf }, old_parent);
    tree.set_parent(area, Some(split));
    tree.set_parent(new_leaf, Some(split));
    match old_parent {
        Some(p) => tree.replace_child(p, area, split),
        None => tree.set_root(split),
    }
    Ok(new_leaf)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k(s: &str) -> EditorKind {
        EditorKind::new(s)
    }

    #[test]
    fn splitting_root_keeps_original_id() {
        let mut tree = AreaTree::single(k("a"));
        let a = tree.root();
        let b = split_area(&mut tree, a, Axis::Horizontal, 0.5, k("b")).unwrap();
        tree.assert_valid();
        assert_eq!(tree.areas(), vec![a, b]);
        assert_ne!(tree.root(), a);
        assert_eq!(tree.sibling(a), Some(b));
    }

    #[test]
    fn split_gives_new_area_a_fresh_view() {
        let mut tree = AreaTree::single(k("a"));
        let a = tree.root();
        let before = tree.area(a).unwrap().view;
        let b = split_area(&mut tree, a, Axis::Horizontal, 0.5, k("b")).unwrap();
        assert_eq!(tree.area(a).unwrap().view, before);
        assert_ne!(tree.area(b).unwrap().view, before);
    }

    #[test]
    fn splitting_nested_area_rewires_parent() {
        let mut tree = AreaTree::single(k("a"));
        let a = tree.root();
        let b = split_area(&mut tree, a, Axis::Horizontal, 0.5, k("b")).unwrap();
        let c = split_area(&mut tree, b, Axis::Vertical, 0.5, k("c")).unwrap();
        tree.assert_valid();
        assert_eq!(tree.areas(), vec![a, b, c]);
    }

    #[test]
    fn ratio_is_clamped_and_nan_rejected() {
        let mut tree = AreaTree::single(k("a"));
        let a = tree.root();
        assert_eq!(split_area(&mut tree, a, Axis::Vertical, f32::NAN, k("b")), Err(OpError::BadRatio));
        assert_eq!(tree.areas().len(), 1, "failed split must not change the tree");
        split_area(&mut tree, a, Axis::Vertical, 0.0, k("b")).unwrap();
        let Some(Node::Split { ratio, .. }) = tree.node(tree.root()) else { panic!() };
        assert_eq!(*ratio, crate::ops::MIN_RATIO);
    }

    #[test]
    fn splitting_a_split_or_stale_id_is_refused() {
        let mut tree = AreaTree::single(k("a"));
        let a = tree.root();
        split_area(&mut tree, a, Axis::Horizontal, 0.5, k("b")).unwrap();
        let root = tree.root();
        assert_eq!(split_area(&mut tree, root, Axis::Vertical, 0.5, k("c")), Err(OpError::NotAnArea(root)));
        tree.assert_valid();
    }
}
