//! Builds a tree of straight cuts from the rectangles of its areas.

use crate::ops::clamp_ratio;
use crate::{Area, Axis, Node, NodeId, Rect, Tree};

/// Tolerance when comparing edge positions in unit space.
pub(super) const EPS: f32 = 1e-4;

pub(super) fn span(r: Rect, axis: Axis) -> (f32, f32) {
    match axis {
        Axis::Horizontal => (r.x, r.x + r.w),
        Axis::Vertical => (r.y, r.y + r.h),
    }
}

/// A tree whose areas fill the unit square exactly as `items` say, or `None`
/// if no sequence of cuts across the whole window separates them (a pinwheel).
pub(super) fn tree_from_rects(items: Vec<(Area, Rect)>) -> Option<Tree> {
    let mut tree = Tree::unrooted();
    let root = node(&mut tree, items, Rect::UNIT)?;
    tree.set_root(root);
    Some(tree)
}

fn node(tree: &mut Tree, mut items: Vec<(Area, Rect)>, bounds: Rect) -> Option<NodeId> {
    if items.len() == 1 {
        let (area, _) = items.pop().expect("one item");
        return Some(tree.insert(Node::Area(area), None));
    }
    for axis in [Axis::Horizontal, Axis::Vertical] {
        let (lo, hi) = span(bounds, axis);
        let mut cuts: Vec<f32> = items.iter().map(|(_, r)| span(*r, axis).1).filter(|&c| c > lo + EPS && c < hi - EPS).collect();
        cuts.sort_by(f32::total_cmp);
        for cut in cuts {
            let before = |r: &Rect| span(*r, axis).1 <= cut + EPS;
            let after = |r: &Rect| span(*r, axis).0 >= cut - EPS;
            if !items.iter().all(|(_, r)| before(r) || after(r)) {
                continue;
            }
            let (first, second): (Vec<_>, Vec<_>) = items.iter().cloned().partition(|(_, r)| before(r));
            if first.is_empty() || second.is_empty() {
                continue;
            }
            let raw = (cut - lo) / (hi - lo);
            // A split cannot be thinner than the minimum ratio; refuse such a layout.
            let Some(ratio) = clamp_ratio(raw).ok().filter(|r| (r - raw).abs() < EPS) else { continue };
            let (b1, b2) = bounds.split(axis, ratio);
            let first = node(tree, first, b1)?;
            let second = node(tree, second, b2)?;
            let id = tree.insert(Node::Split { axis, ratio, first, second }, None);
            tree.set_parent(first, Some(id));
            tree.set_parent(second, Some(id));
            return Some(id);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EditorKind;

    fn area(name: &str) -> Area {
        let tree = Tree::single(EditorKind::new(name));
        tree.area(tree.root()).unwrap().clone()
    }

    fn r(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect { x, y, w, h }
    }

    #[test]
    fn pinwheel_has_no_straight_cut() {
        let items = vec![
            (area("a"), r(0.0, 0.0, 0.6, 0.4)),
            (area("b"), r(0.6, 0.0, 0.4, 0.6)),
            (area("c"), r(0.4, 0.6, 0.6, 0.4)),
            (area("d"), r(0.0, 0.4, 0.4, 0.6)),
            (area("e"), r(0.4, 0.4, 0.2, 0.2)),
        ];
        assert!(tree_from_rects(items).is_none());
    }

    #[test]
    fn a_sliver_thinner_than_the_minimum_ratio_is_refused() {
        let items = vec![(area("a"), r(0.0, 0.0, 0.02, 1.0)), (area("b"), r(0.02, 0.0, 0.98, 1.0))];
        assert!(tree_from_rects(items).is_none());
    }

    #[test]
    fn a_thin_first_cut_does_not_hide_a_valid_later_one() {
        // Full-height strips with edges at 0.04 and 0.5. Cutting at 0.04 first
        // would be a 4 % sliver; cutting at 0.5 first leaves a valid 8 % split inside.
        let items = vec![
            (area("a"), r(0.0, 0.0, 0.04, 1.0)),
            (area("b"), r(0.04, 0.0, 0.46, 1.0)),
            (area("d"), r(0.5, 0.0, 0.5, 1.0)),
        ];
        let tree = tree_from_rects(items).expect("a valid cut order exists");
        tree.assert_valid();
        assert_eq!(tree.areas().len(), 3);
    }

    #[test]
    fn grid_rebuilds_with_the_same_rectangles() {
        let items = vec![
            (area("a"), r(0.0, 0.0, 0.5, 0.5)),
            (area("b"), r(0.5, 0.0, 0.5, 0.5)),
            (area("c"), r(0.0, 0.5, 1.0, 0.5)),
        ];
        let tree = tree_from_rects(items).unwrap();
        tree.assert_valid();
        let rects = crate::layout::area_rects(&tree, Rect::UNIT);
        assert_eq!(rects.len(), 3);
        let total: f32 = rects.iter().map(|(_, r)| r.w * r.h).sum();
        assert!((total - 1.0).abs() < 1e-5);
    }
}
