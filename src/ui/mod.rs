//! egui drawing of a [`Workspace`](crate::Workspace).

mod action;
mod active;
mod area;
mod corner;
mod corner_drag;
mod dividers;
mod gesture;
mod header;
mod hint;
mod host;
mod overlay;
mod pixels;
mod rounding;
mod shortcuts;
mod show;
mod snap;

pub use active::ActiveArea;
pub use hint::{DragMode, TilingHint};
pub use host::AreaHost;
pub use show::show_workspace;
