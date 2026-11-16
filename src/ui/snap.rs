//! Snapping of dividers and split positions to each other, so panes line up.

use super::pixels::to_pixels;
use crate::layout::split_rects;
use crate::{Axis, NodeId, UnitRect, AreaTree};

/// How close (in pixels) a position must be to a target to snap onto it.
pub(super) const SNAP_PX: f32 = 8.0;

/// The candidate closest to `value`, if within [`SNAP_PX`].
pub(super) fn nearest(value: f32, candidates: impl IntoIterator<Item = f32>) -> Option<f32> {
    candidates
        .into_iter()
        .filter(|c| (c - value).abs() <= SNAP_PX)
        .min_by(|a, b| (a - value).abs().total_cmp(&(b - value).abs()))
}

/// On-screen coordinates of every divider laid out along `axis` (x for
/// `Horizontal` splits, y for `Vertical`), skipping `exclude` and anything
/// inside it: those move together with the divider being dragged.
pub(super) fn divider_coords(tree: &AreaTree, bounds: egui::Rect, axis: Axis, exclude: Option<NodeId>) -> Vec<f32> {
    split_rects(tree, UnitRect::UNIT)
        .into_iter()
        .filter(|s| s.axis == axis && !exclude.is_some_and(|e| is_within(tree, s.id, e)))
        .map(|s| {
            let r = to_pixels(s.rect, bounds);
            match axis {
                Axis::Horizontal => r.min.x + r.width() * s.ratio,
                Axis::Vertical => r.min.y + r.height() * s.ratio,
            }
        })
        .collect()
}

/// True if `node` is `ancestor` or sits below it.
fn is_within(tree: &AreaTree, node: NodeId, ancestor: NodeId) -> bool {
    let mut cur = Some(node);
    while let Some(n) = cur {
        if n == ancestor {
            return true;
        }
        cur = tree.parent(n);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::split_area;
    use crate::EditorKind;

    #[test]
    fn nearest_picks_the_closest_within_threshold() {
        assert_eq!(nearest(100.0, [90.0, 103.0, 150.0]), Some(103.0));
        assert_eq!(nearest(100.0, [80.0, 120.0]), None);
        assert_eq!(nearest(100.0, []), None);
    }

    #[test]
    fn dragged_divider_and_its_descendants_are_not_snap_targets() {
        let k = || EditorKind::new("x");
        let mut tree = AreaTree::single(k());
        let a = tree.root();
        let b = split_area(&mut tree, a, Axis::Horizontal, 0.25, k()).unwrap(); // x = 100 of 400
        split_area(&mut tree, b, Axis::Horizontal, 0.5, k()).unwrap(); // x = 100 + 150 = 250
        let bounds = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(400.0, 300.0));
        let outer = tree.parent(a).unwrap();

        let all = divider_coords(&tree, bounds, Axis::Horizontal, None);
        assert_eq!(all.len(), 2);
        let others = divider_coords(&tree, bounds, Axis::Horizontal, Some(outer));
        assert!(others.is_empty(), "inner divider lives inside the dragged one");
        assert!(divider_coords(&tree, bounds, Axis::Vertical, None).is_empty());
    }
}
