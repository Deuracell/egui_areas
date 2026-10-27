slotmap::new_key_type! {
    /// Stable handle to a node in a [`Tree`](crate::Tree). Splitting an area keeps
    /// that area's id, so per-area view state can be keyed by it.
    pub struct NodeId;
}

/// Key for per-area view state (camera, tool, ...) that the app stores outside
/// this crate. Travels with an area's editor through swaps and is never reused,
/// so the app can drop state for any id missing from [`Tree::view_ids`](crate::Tree::view_ids).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ViewId(pub(crate) u64);
