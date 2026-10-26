//! egui drawing of a [`Workspace`](crate::Workspace).

mod action;
mod active;
mod area;
mod dividers;
mod header;
mod host;
mod pixels;
mod show;

pub use active::ActiveArea;
pub use host::AreaHost;
pub use show::show_workspace;
