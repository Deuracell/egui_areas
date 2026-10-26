slotmap::new_key_type! {
    /// Stable handle to a node in a [`Tree`](crate::Tree). Splitting an area keeps
    /// that area's id, so per-area view state can be keyed by it.
    pub struct NodeId;
}
