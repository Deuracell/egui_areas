slotmap::new_key_type! {
    /// Stable handle to a node in a [`AreaTree`](crate::AreaTree). Splitting an area keeps
    /// that area's id, so per-area view state can be keyed by it.
    pub struct NodeId;
}

/// Key for per-area view state (camera, tool, ...) that the app stores outside
/// this crate. Travels with an area's editor through swaps and is never reused,
/// so the app can drop state for any id missing from [`AreaTree::view_ids`](crate::AreaTree::view_ids).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// The default (0) is never handed out; use it only as a placeholder.
pub struct ViewId(pub(crate) u64);
