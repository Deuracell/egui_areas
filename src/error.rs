use crate::NodeId;
use std::fmt;

/// Why a tree operation was refused. The tree is left unchanged on error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OpError {
    NotFound(NodeId),
    NotAnArea(NodeId),
    NotASplit(NodeId),
    SameArea,
    /// Join needs two areas that are the two halves of one split.
    NotSiblings,
    /// The only remaining area cannot be closed.
    LastArea,
    /// Ratio was NaN or infinite.
    BadRatio,
    /// The two areas do not share an edge.
    NotAdjacent,
    /// One area would be cut in two pieces by the other, which a tiling cannot show.
    BandInterior,
    /// The result would not be a layout of straight cuts across the whole window.
    Unrepresentable,
}

impl fmt::Display for OpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "no such node {id:?}"),
            Self::NotAnArea(id) => write!(f, "node {id:?} is not an area"),
            Self::NotASplit(id) => write!(f, "node {id:?} is not a split"),
            Self::SameArea => write!(f, "operation needs two different areas"),
            Self::NotSiblings => write!(f, "areas are not the two halves of one split"),
            Self::LastArea => write!(f, "cannot close the last area"),
            Self::BadRatio => write!(f, "split ratio must be a finite number"),
            Self::NotAdjacent => write!(f, "areas do not share an edge"),
            Self::BandInterior => write!(f, "the overlap lies strictly inside one of the areas"),
            Self::Unrepresentable => write!(f, "the result cannot be laid out with straight cuts"),
        }
    }
}

impl std::error::Error for OpError {}
