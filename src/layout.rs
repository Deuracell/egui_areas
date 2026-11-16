use crate::{Axis, Node, NodeId, AreaTree};

/// Axis-aligned rectangle. The model works in unit space (`UnitRect::UNIT`); the
/// drawing layer scales it to pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnitRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl UnitRect {
    pub const UNIT: UnitRect = UnitRect { x: 0.0, y: 0.0, w: 1.0, h: 1.0 };

    /// Divides the rectangle at `ratio` along `axis`.
    pub fn split(self, axis: Axis, ratio: f32) -> (UnitRect, UnitRect) {
        match axis {
            Axis::Horizontal => {
                let w1 = self.w * ratio;
                (UnitRect { w: w1, ..self }, UnitRect { x: self.x + w1, w: self.w - w1, ..self })
            }
            Axis::Vertical => {
                let h1 = self.h * ratio;
                (UnitRect { h: h1, ..self }, UnitRect { y: self.y + h1, h: self.h - h1, ..self })
            }
        }
    }
}

/// Rectangle of every area within `bounds`, in depth-first order.
pub fn area_rects(tree: &AreaTree, bounds: UnitRect) -> Vec<(NodeId, UnitRect)> {
    let mut out = Vec::new();
    walk(tree, tree.root(), bounds, &mut out);
    out
}

fn walk(tree: &AreaTree, id: NodeId, rect: UnitRect, out: &mut Vec<(NodeId, UnitRect)>) {
    match tree.node(id) {
        Some(Node::Split { axis, ratio, first, second }) => {
            let (a, b) = rect.split(*axis, *ratio);
            walk(tree, *first, a, out);
            walk(tree, *second, b, out);
        }
        Some(Node::Area(_)) => out.push((id, rect)),
        None => {}
    }
}

/// A split node together with the rectangle it divides.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SplitRect {
    pub id: NodeId,
    pub axis: Axis,
    pub ratio: f32,
    pub rect: UnitRect,
}

/// Every split within `bounds`, parents before children.
pub fn split_rects(tree: &AreaTree, bounds: UnitRect) -> Vec<SplitRect> {
    let mut out = Vec::new();
    walk_splits(tree, tree.root(), bounds, &mut out);
    out
}

fn walk_splits(tree: &AreaTree, id: NodeId, rect: UnitRect, out: &mut Vec<SplitRect>) {
    if let Some(Node::Split { axis, ratio, first, second }) = tree.node(id) {
        out.push(SplitRect { id, axis: *axis, ratio: *ratio, rect });
        let (a, b) = rect.split(*axis, *ratio);
        walk_splits(tree, *first, a, out);
        walk_splits(tree, *second, b, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EditorKind, ops::split_area};

    #[test]
    fn rects_tile_the_bounds_without_gaps() {
        let k = || EditorKind::new("x");
        let mut tree = AreaTree::single(k());
        let root = tree.root();
        let right = split_area(&mut tree, root, Axis::Horizontal, 0.25, k()).unwrap();
        split_area(&mut tree, right, Axis::Vertical, 0.5, k()).unwrap();
        let rects = area_rects(&tree, UnitRect::UNIT);
        assert_eq!(rects.len(), 3);
        let area: f32 = rects.iter().map(|(_, r)| r.w * r.h).sum();
        assert!((area - 1.0).abs() < 1e-6);
        assert_eq!(rects[0].1, UnitRect { x: 0.0, y: 0.0, w: 0.25, h: 1.0 });
    }

    #[test]
    fn split_rects_report_the_rectangle_each_split_divides() {
        let k = || EditorKind::new("x");
        let mut tree = AreaTree::single(k());
        let root = tree.root();
        let right = split_area(&mut tree, root, Axis::Horizontal, 0.25, k()).unwrap();
        split_area(&mut tree, right, Axis::Vertical, 0.5, k()).unwrap();
        let splits = split_rects(&tree, UnitRect::UNIT);
        assert_eq!(splits.len(), 2);
        assert_eq!(splits[0].rect, UnitRect::UNIT);
        assert_eq!(splits[1].rect, UnitRect { x: 0.25, y: 0.0, w: 0.75, h: 1.0 });
    }
}
