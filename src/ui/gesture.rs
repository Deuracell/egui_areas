//! Decides what a corner drag means. Pure geometry, no egui context, so it is
//! unit-tested without a window.

use super::corner::Corner;
use super::snap::nearest;
use crate::{Axis, NodeId};

/// Movement below this many pixels is still "undecided".
const DEAD_ZONE: f32 = 10.0;
/// Smallest extent either half of a split may get.
pub(super) const MIN_SPLIT_PX: f32 = 80.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Side {
    Left,
    Right,
    Top,
    Bottom,
}

/// The neighbour an area can absorb: its sibling in the tree, when that is a single area.
#[derive(Clone, Copy, Debug)]
pub(super) struct Neighbour {
    pub id: NodeId,
    pub rect: egui::Rect,
    pub side: Side,
}

pub(super) struct Input<'a> {
    pub area: egui::Rect,
    pub corner: Corner,
    /// Pointer position minus where the drag started.
    pub delta: egui::Vec2,
    pub pointer: egui::Pos2,
    pub ctrl: bool,
    pub neighbour: Option<Neighbour>,
    /// The other area under the pointer, for swapping.
    pub hovered_other: Option<(NodeId, egui::Rect)>,
    /// x positions of vertical dividers a left|right split can snap to.
    pub snap_x: &'a [f32],
    /// y positions of horizontal dividers a top/bottom split can snap to.
    pub snap_y: &'a [f32],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum Gesture {
    Idle,
    /// `guide` is the coordinate snapped to, for drawing a guide line.
    Split { axis: Axis, ratio: f32, guide: Option<f32> },
    /// The dragged area grows over `target`, which is removed.
    Join { target: NodeId, target_rect: egui::Rect },
    Swap { target: NodeId, target_rect: egui::Rect },
}

pub(super) fn classify(i: &Input) -> Gesture {
    if i.ctrl {
        return match i.hovered_other {
            Some((target, target_rect)) => Gesture::Swap { target, target_rect },
            None => Gesture::Idle,
        };
    }
    if i.delta.length() < DEAD_ZONE {
        return Gesture::Idle;
    }
    let horizontal = i.delta.x.abs() >= i.delta.y.abs();
    let (along, inward_sign) = if horizontal { (i.delta.x, i.corner.inward_x()) } else { (i.delta.y, i.corner.inward_y()) };

    if along * inward_sign > 0.0 {
        split(i, horizontal)
    } else {
        join(i, horizontal)
    }
}

fn split(i: &Input, horizontal: bool) -> Gesture {
    let (axis, pointer, origin, extent, snaps) = if horizontal {
        (Axis::Horizontal, i.pointer.x, i.area.min.x, i.area.width(), i.snap_x)
    } else {
        (Axis::Vertical, i.pointer.y, i.area.min.y, i.area.height(), i.snap_y)
    };
    if extent < 2.0 * MIN_SPLIT_PX {
        return Gesture::Idle;
    }
    let snapped = nearest(pointer, snaps.iter().copied().chain([origin + extent / 2.0]));
    let raw = (snapped.unwrap_or(pointer) - origin) / extent;
    let ratio = raw.clamp(MIN_SPLIT_PX / extent, 1.0 - MIN_SPLIT_PX / extent);
    let guide = snapped.filter(|_| ratio == raw);
    Gesture::Split { axis, ratio, guide }
}

fn join(i: &Input, horizontal: bool) -> Gesture {
    let outward = if horizontal {
        if i.corner.inward_x() > 0.0 { Side::Left } else { Side::Right }
    } else if i.corner.inward_y() > 0.0 {
        Side::Top
    } else {
        Side::Bottom
    };
    match i.neighbour {
        Some(n) if n.side == outward => Gesture::Join { target: n.id, target_rect: n.rect },
        _ => Gesture::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EditorKind, Tree};

    fn ids() -> (NodeId, NodeId) {
        let mut t = Tree::single(EditorKind::new("a"));
        let a = t.root();
        let b = crate::ops::split_area(&mut t, a, Axis::Horizontal, 0.5, EditorKind::new("b")).unwrap();
        (a, b)
    }

    fn area() -> egui::Rect {
        egui::Rect::from_min_size(egui::pos2(100.0, 100.0), egui::vec2(400.0, 300.0))
    }

    fn input(corner: Corner, delta: (f32, f32)) -> Input<'static> {
        let start = match corner {
            Corner::TopLeft => area().left_top(),
            Corner::TopRight => area().right_top(),
            Corner::BottomLeft => area().left_bottom(),
            Corner::BottomRight => area().right_bottom(),
        };
        Input {
            area: area(),
            corner,
            delta: egui::vec2(delta.0, delta.1),
            pointer: start + egui::vec2(delta.0, delta.1),
            ctrl: false,
            neighbour: None,
            hovered_other: None,
            snap_x: &[],
            snap_y: &[],
        }
    }

    #[test]
    fn small_movement_is_undecided() {
        assert_eq!(classify(&input(Corner::TopLeft, (4.0, 3.0))), Gesture::Idle);
    }

    #[test]
    fn inward_horizontal_drag_splits_left_right_at_the_pointer() {
        let Gesture::Split { axis, ratio, .. } = classify(&input(Corner::TopLeft, (100.0, 10.0))) else { panic!() };
        assert_eq!(axis, Axis::Horizontal);
        assert!((ratio - 0.25).abs() < 1e-6);
    }

    #[test]
    fn inward_vertical_drag_from_bottom_right_splits_top_bottom() {
        let Gesture::Split { axis, ratio, .. } = classify(&input(Corner::BottomRight, (-5.0, -150.0))) else { panic!() };
        assert_eq!(axis, Axis::Vertical);
        assert!((ratio - 0.5).abs() < 1e-6);
    }

    #[test]
    fn split_snaps_to_a_nearby_divider_and_to_the_middle() {
        let mut i = input(Corner::TopLeft, (105.0, 0.0)); // pointer x = 205
        i.snap_x = &[200.0];
        let Gesture::Split { ratio, guide, .. } = classify(&i) else { panic!() };
        assert_eq!(guide, Some(200.0));
        assert!((ratio - 0.25).abs() < 1e-6);

        i.snap_x = &[];
        i.pointer.x = 296.0; // middle of x 100..500 is 300
        i.delta.x = 196.0;
        let Gesture::Split { ratio, guide, .. } = classify(&i) else { panic!() };
        assert_eq!(guide, Some(300.0));
        assert!((ratio - 0.5).abs() < 1e-6);
    }

    #[test]
    fn split_ratio_keeps_both_halves_usable() {
        let Gesture::Split { ratio, .. } = classify(&input(Corner::TopLeft, (15.0, 0.0))) else { panic!() };
        assert!((ratio - MIN_SPLIT_PX / 400.0).abs() < 1e-6);
    }

    #[test]
    fn outward_drag_joins_only_toward_the_neighbour_side() {
        let (_, b) = ids();
        let mut i = input(Corner::TopRight, (60.0, 0.0));
        i.neighbour = Some(Neighbour { id: b, rect: area(), side: Side::Right });
        assert!(matches!(classify(&i), Gesture::Join { target, .. } if target == b));

        i.neighbour = Some(Neighbour { id: b, rect: area(), side: Side::Left });
        assert_eq!(classify(&i), Gesture::Idle, "neighbour is on the other side");
        i.neighbour = None;
        assert_eq!(classify(&i), Gesture::Idle, "nothing to join");
    }

    #[test]
    fn ctrl_swaps_with_the_area_under_the_pointer() {
        let (_, b) = ids();
        let mut i = input(Corner::TopLeft, (0.0, 0.0));
        i.ctrl = true;
        assert_eq!(classify(&i), Gesture::Idle);
        i.hovered_other = Some((b, area()));
        assert!(matches!(classify(&i), Gesture::Swap { target, .. } if target == b));
    }
}
