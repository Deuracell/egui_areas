use super::clamp_ratio;
use crate::{Node, NodeId, OpError, Tree};

/// Moves the divider of `split` so its first child gets `ratio` of the space
/// (clamped to `MIN_RATIO..=MAX_RATIO`). Returns the ratio actually applied.
pub fn resize_split(tree: &mut Tree, split: NodeId, ratio: f32) -> Result<f32, OpError> {
    let ratio = clamp_ratio(ratio)?;
    match tree.node_mut(split) {
        Some(Node::Split { ratio: r, .. }) => {
            *r = ratio;
            Ok(ratio)
        }
        Some(Node::Area(_)) => Err(OpError::NotASplit(split)),
        None => Err(OpError::NotFound(split)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Axis, EditorKind, ops::split_area};

    #[test]
    fn resize_clamps_and_rejects_areas() {
        let mut tree = Tree::single(EditorKind::new("x"));
        let a = tree.root();
        split_area(&mut tree, a, Axis::Horizontal, 0.5, EditorKind::new("y")).unwrap();
        let split = tree.root();
        assert_eq!(resize_split(&mut tree, split, 0.7), Ok(0.7));
        assert_eq!(resize_split(&mut tree, split, 5.0), Ok(crate::ops::MAX_RATIO));
        assert_eq!(resize_split(&mut tree, a, 0.5), Err(OpError::NotASplit(a)));
        assert_eq!(resize_split(&mut tree, split, f32::INFINITY), Err(OpError::BadRatio));
    }
}
