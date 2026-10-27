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

/// An area next to the dragged corner that the dragged area could take over.
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
    pub shift: bool,
    /// The areas just outside this corner, one per side at most.
    pub neighbours: &'a [Neighbour],
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
    /// The dragged area takes over `target`'s space where they face each other.
    /// `result` is where the area covering that stretch will be; the drawing
    /// layer replaces the first guess made here with the real outcome.
    Join { target: NodeId, target_rect: egui::Rect, result: egui::Rect },
    Swap { target: NodeId, target_rect: egui::Rect },
    /// Open a copy of the dragged area in a new window.
    PopOut,
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
    if i.shift {
        return Gesture::PopOut;
    }
    let horizontal = i.delta.x.abs() >= i.delta.y.abs();
    let first = decide(i, horizontal);
    if first != Gesture::Idle {
        return first;
    }
    // Near the diagonal there may be something to join along the other axis only.
    let other = if horizontal { i.delta.y } else { i.delta.x };
    if other.abs() >= DEAD_ZONE / 2.0 {
        if let join @ Gesture::Join { .. } = decide(i, !horizontal) {
            return join;
        }
    }
    Gesture::Idle
}

/// What dragging along one axis means: split if heading into the area, join if out of it.
fn decide(i: &Input, horizontal: bool) -> Gesture {
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
    match i.neighbours.iter().find(|n| n.side == outward) {
        Some(n) => Gesture::Join { target: n.id, target_rect: n.rect, result: n.rect },
        None => Gesture::Idle,
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

    fn input<'a>(corner: Corner, delta: (f32, f32)) -> Input<'a> {
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
            shift: false,
            neighbours: &[],
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
        let right = [Neighbour { id: b, rect: area(), side: Side::Right }];
        let mut i = input(Corner::TopRight, (60.0, 0.0));
        i.neighbours = &right;
        assert!(matches!(classify(&i), Gesture::Join { target, .. } if target == b));

        let left = [Neighbour { id: b, rect: area(), side: Side::Left }];
        i.neighbours = &left;
        assert_eq!(classify(&i), Gesture::Idle, "neighbour is on the other side");
        i.neighbours = &[];
        assert_eq!(classify(&i), Gesture::Idle, "nothing to join");
    }

    #[test]
    fn shift_drag_pops_the_area_out_once_moved() {
        let mut i = input(Corner::TopLeft, (4.0, 3.0));
        i.shift = true;
        assert_eq!(classify(&i), Gesture::Idle, "still inside the dead zone");
        let mut i = input(Corner::TopLeft, (100.0, 10.0));
        i.shift = true;
        assert_eq!(classify(&i), Gesture::PopOut, "shift wins over split, whichever way it is dragged");
        i.delta = egui::vec2(-60.0, 0.0);
        assert_eq!(classify(&i), Gesture::PopOut);
    }

    #[test]
    fn a_diagonal_drag_joins_along_the_other_axis_when_the_dominant_one_has_nothing() {
        let (_, b) = ids();
        // Neighbour above the top-left corner; the drag is mostly leftward but also upward.
        let above = [Neighbour { id: b, rect: area(), side: Side::Top }];
        let mut i = input(Corner::TopLeft, (-30.0, -20.0));
        i.neighbours = &above;
        assert!(matches!(classify(&i), Gesture::Join { target, .. } if target == b));
        // Too little movement on the other axis to count.
        let mut i = input(Corner::TopLeft, (-40.0, -2.0));
        i.neighbours = &above;
        assert_eq!(classify(&i), Gesture::Idle);
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
