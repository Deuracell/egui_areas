//! Tree editing operations as plain functions. Each refuses invalid input with
//! an [`OpError`](crate::OpError) and leaves the tree unchanged.

mod close;
mod join;
mod maximize;
mod ratio;
mod resize;
mod split;
mod swap;

pub use close::close_area;
pub use join::join_areas;
pub use maximize::{maximize_area, restore_area, toggle_maximize};
pub(crate) use ratio::clamp_ratio;
pub use ratio::{MAX_RATIO, MIN_RATIO};
pub use resize::resize_split;
pub use split::split_area;
pub use swap::swap_areas;
