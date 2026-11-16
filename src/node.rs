use crate::{EditorKind, NodeId, ViewId};
use serde::{Deserialize, Serialize};

/// Direction in which a split lays out its two children.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Axis {
    /// `first` on the left, `second` on the right.
    Horizontal,
    /// `first` on top, `second` below.
    Vertical,
}

/// A leaf of the tree: one editor and the key of its view state.
#[derive(Clone, Debug, PartialEq)]
pub struct Area {
    pub kind: EditorKind,
    pub view: ViewId,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    /// `ratio` is the fraction of the split's extent given to `first`.
    Split { axis: Axis, ratio: f32, first: NodeId, second: NodeId },
    Area(Area),
}
