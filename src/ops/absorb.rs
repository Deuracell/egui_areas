use super::rebuild::{EPS, span, tree_from_rects};
use crate::layout::area_rects;
use crate::{Area, Axis, NodeId, OpError, UnitRect, AreaTree};

fn other(axis: Axis) -> Axis {
    match axis {
        Axis::Horizontal => Axis::Vertical,
        Axis::Vertical => Axis::Horizontal,
    }
}

fn with_span(r: UnitRect, axis: Axis, lo: f32, hi: f32) -> UnitRect {
    match axis {
        Axis::Horizontal => UnitRect { x: lo, w: hi - lo, ..r },
        Axis::Vertical => UnitRect { y: lo, h: hi - lo, ..r },
    }
}

/// The part of `r` outside the band `lo..hi` along `along`: at most one piece,
/// or `BandInterior` if the band leaves a piece on both sides.
fn remainder(r: UnitRect, along: Axis, lo: f32, hi: f32) -> Result<Option<UnitRect>, OpError> {
    let (start, end) = span(r, along);
    match (lo - start > EPS, end - hi > EPS) {
        (true, true) => Err(OpError::BandInterior),
        (true, false) => Ok(Some(with_span(r, along, start, lo))),
        (false, true) => Ok(Some(with_span(r, along, hi, end))),
        (false, false) => Ok(None),
    }
}

/// `x` is dragged into its neighbour `y`: in the stretch where they face each
/// other, `x` takes over `y`'s width (or height) as well as its own.
///
/// What lies outside that stretch stays: the part of `y` becomes a smaller `y`,
/// and the part of `x` stays as `x` while the stretch gets a new copy of it, as
/// an L-shaped area cannot exist. If `y` is entirely inside the stretch it is
/// removed. Two aligned siblings give a plain join.
///
/// Rebuilds the tree, so node ids change. Returns the id of the area now
/// covering the stretch. Refuses (tree unchanged) areas that do not touch, a
/// stretch strictly inside one of them, and results that straight cuts across
/// the whole window cannot produce.
pub fn absorb_across(tree: &mut AreaTree, x: NodeId, y: NodeId) -> Result<NodeId, OpError> {
    if x == y {
        return Err(OpError::SameArea);
    }
    for id in [x, y] {
        if !tree.contains(id) {
            return Err(OpError::NotFound(id));
        }
        if tree.area(id).is_none() {
            return Err(OpError::NotAnArea(id));
        }
    }
    let rects = area_rects(tree, UnitRect::UNIT);
    let rect_of = |id| rects.iter().find(|(a, _)| *a == id).map(|&(_, r)| r).expect("areas have rectangles");
    let (rx, ry) = (rect_of(x), rect_of(y));

    // Which way they touch, and the stretch (band) along the shared edge.
    let (across, band) = [Axis::Horizontal, Axis::Vertical]
        .into_iter()
        .find_map(|across| {
            let (xl, xh) = span(rx, across);
            let (yl, yh) = span(ry, across);
            if (xh - yl).abs() > EPS && (yh - xl).abs() > EPS {
                return None;
            }
            let along = other(across);
            let lo = span(rx, along).0.max(span(ry, along).0);
            let hi = span(rx, along).1.min(span(ry, along).1);
            (hi - lo > EPS).then_some((across, (lo, hi)))
        })
        .ok_or(OpError::NotAdjacent)?;
    let along = other(across);

    let x_rest = remainder(rx, along, band.0, band.1)?;
    let y_rest = remainder(ry, along, band.0, band.1)?;
    let (a_lo, a_hi) = (span(rx, across).0.min(span(ry, across).0), span(rx, across).1.max(span(ry, across).1));
    let band_rect = with_span(with_span(rx, across, a_lo, a_hi), along, band.0, band.1);

    let x_area = tree.area(x).expect("checked").clone();
    let y_area = tree.area(y).expect("checked").clone();
    let band_area = if x_rest.is_some() { Area { kind: x_area.kind.clone(), view: tree.new_view() } } else { x_area.clone() };
    let band_view = band_area.view;

    let mut items: Vec<(Area, UnitRect)> = rects
        .iter()
        .filter(|(id, _)| *id != x && *id != y)
        .map(|&(id, r)| (tree.area(id).expect("areas").clone(), r))
        .collect();
    items.push((band_area, band_rect));
    if let Some(r) = x_rest {
        items.push((x_area, r));
    }
    if let Some(r) = y_rest {
        items.push((y_area, r));
    }

    let rebuilt = tree_from_rects(items).ok_or(OpError::Unrepresentable)?;
    *tree = rebuilt;
    Ok(tree
        .areas()
        .into_iter()
        .find(|&id| tree.area(id).is_some_and(|a| a.view == band_view))
        .expect("the band area is in the rebuilt tree"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EditorKind, ops::split_area};

    fn k(s: &str) -> EditorKind {
        EditorKind::new(s)
    }

    /// A on the left, B above C on the right; B takes 40 % of the right column.
    fn layout() -> (AreaTree, NodeId, NodeId, NodeId) {
        let mut tree = AreaTree::single(k("a"));
        let a = tree.root();
        let b = split_area(&mut tree, a, Axis::Horizontal, 0.5, k("b")).unwrap();
        let c = split_area(&mut tree, b, Axis::Vertical, 0.4, k("c")).unwrap();
        // `b` keeps its id and is the top half; `c` is the new bottom area.
        (tree, a, b, c)
    }

    fn describe(tree: &AreaTree) -> Vec<(String, [i32; 4])> {
        let q = |v: f32| (v * 100.0).round() as i32;
        let mut out: Vec<_> = area_rects(tree, UnitRect::UNIT)
            .into_iter()
            .map(|(id, r)| (tree.area(id).unwrap().kind.id().to_string(), [q(r.x), q(r.y), q(r.w), q(r.h)]))
            .collect();
        out.sort();
        out
    }

    #[test]
    fn dragging_the_tall_area_into_the_short_one_splits_the_tall_one() {
        let (mut tree, a, b, _) = layout();
        let a_view = tree.area(a).unwrap().view;
        let survivor = absorb_across(&mut tree, a, b).unwrap();
        tree.assert_valid();
        assert_eq!(
            describe(&tree),
            [("a".to_string(), [0, 0, 100, 40]), ("a".to_string(), [0, 40, 50, 60]), ("c".to_string(), [50, 40, 50, 60])]
        );
        let top = tree.area(survivor).unwrap();
        assert_ne!(top.view, a_view, "the stretch gets a new instance");
        let lower = tree.areas().into_iter().find(|&id| tree.area(id).unwrap().view == a_view).unwrap();
        assert_eq!(area_rects(&tree, UnitRect::UNIT).into_iter().find(|(id, _)| *id == lower).unwrap().1.y, 0.4);
    }

    #[test]
    fn dragging_the_short_area_into_the_tall_one_shrinks_the_tall_one() {
        let (mut tree, a, b, _) = layout();
        let (a_view, b_view) = (tree.area(a).unwrap().view, tree.area(b).unwrap().view);
        let survivor = absorb_across(&mut tree, b, a).unwrap();
        tree.assert_valid();
        assert_eq!(
            describe(&tree),
            [("a".to_string(), [0, 40, 50, 60]), ("b".to_string(), [0, 0, 100, 40]), ("c".to_string(), [50, 40, 50, 60])]
        );
        assert_eq!(tree.area(survivor).unwrap().view, b_view, "the dragged area keeps itself");
        assert!(tree.view_ids().contains(&a_view));
    }

    #[test]
    fn aligned_siblings_give_a_plain_join() {
        let mut tree = AreaTree::single(k("a"));
        let a = tree.root();
        let b = split_area(&mut tree, a, Axis::Horizontal, 0.5, k("b")).unwrap();
        let view = tree.area(a).unwrap().view;
        let survivor = absorb_across(&mut tree, a, b).unwrap();
        assert_eq!(tree.areas(), vec![survivor]);
        assert_eq!(tree.area(survivor).unwrap().view, view);
    }

    #[test]
    fn refused_cases_leave_the_tree_unchanged() {
        let mut tree = AreaTree::single(k("a"));
        let a = tree.root();
        let b = split_area(&mut tree, a, Axis::Horizontal, 0.34, k("b")).unwrap();
        let c = split_area(&mut tree, b, Axis::Horizontal, 0.5, k("c")).unwrap();
        let before = describe(&tree);
        assert_eq!(absorb_across(&mut tree, a, c), Err(OpError::NotAdjacent));
        assert_eq!(absorb_across(&mut tree, a, a), Err(OpError::SameArea));
        assert_eq!(describe(&tree), before);

        // A tall area next to the middle of three stacked ones: the stretch is interior.
        let mut tree = AreaTree::single(k("a"));
        let a = tree.root();
        let right = split_area(&mut tree, a, Axis::Horizontal, 0.5, k("x")).unwrap();
        let mid = split_area(&mut tree, right, Axis::Vertical, 0.34, k("y")).unwrap();
        split_area(&mut tree, mid, Axis::Vertical, 0.5, k("z")).unwrap();
        let before = describe(&tree);
        assert_eq!(absorb_across(&mut tree, a, mid), Err(OpError::BandInterior));
        assert_eq!(describe(&tree), before);
    }
}
