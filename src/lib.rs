//! Area tiling: a headless tree of splits and editor areas with the operations
//! that edit it (split, join, close, resize, swap, maximize), plus an egui layer
//! in [`ui`] that draws it. Knows nothing about what the editors show.

pub mod editor_kind;
pub mod error;
pub mod ids;
pub mod layout;
pub mod node;
pub mod ops;
pub mod tree;
mod tree_def;
pub mod ui;
pub mod workspace;
pub mod workspaces;

pub use editor_kind::EditorKind;
pub use error::OpError;
pub use ids::{NodeId, ViewId};
pub use layout::{Rect, SplitRect};
pub use node::{Area, Axis, Node};
pub use tree::Tree;
pub use workspace::Workspace;
pub use workspaces::{Workspaces, WorkspacesError};
