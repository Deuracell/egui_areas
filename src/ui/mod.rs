//! egui drawing of a [`Workspace`](crate::Workspace).

mod area;
mod dividers;
mod header;
mod host;
mod pixels;
mod show;

pub use host::AreaHost;
pub use show::show_workspace;
