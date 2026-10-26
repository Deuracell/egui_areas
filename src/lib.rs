//! Headless area-tiling model: a tree of splits and editor areas, plus the
//! operations (split, join, close, resize, swap, maximize) that edit it.

pub mod editor_kind;
pub mod error;
pub mod ids;
pub mod layout;
pub mod node;
pub mod ops;
pub mod tree;
mod tree_def;
pub mod workspace;

pub use editor_kind::EditorKind;
pub use error::OpError;
pub use ids::NodeId;
pub use layout::Rect;
pub use node::{Area, Axis, Node};
pub use tree::Tree;
pub use workspace::Workspace;
