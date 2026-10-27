/// What the layout is offering the user right now, for the status bar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TilingHint {
    #[default]
    None,
    /// Pointer is on a divider.
    Divider,
    /// Pointer is on an area corner.
    Corner,
    /// A corner gesture is under way.
    Dragging(DragMode),
    /// The workspace shows one maximized area.
    Maximized,
}

/// What releasing a corner drag would do right now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragMode {
    /// Not moved far enough, or no valid target.
    Undecided,
    Split,
    Join,
    Swap,
    PopOut,
    /// Ctrl is held but the pointer is not over another area yet.
    SwapPending,
}
