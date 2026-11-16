use crate::layout::{UnitRect, area_rects};
use crate::{NodeId, AreaTree};
use serde::{Deserialize, Serialize};

/// A named layout: one tree of areas, optionally with one area maximized.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Workspace {
    pub name: String,
    /// The kind of work this layout suits (an open string such as `"sketch"`),
    /// if any. The crate does not know what modes exist.
    #[serde(default)]
    pub mode: Option<String>,
    /// The layout used for its `mode` when several are tagged with it.
    #[serde(default)]
    pub mode_default: bool,
    pub tree: AreaTree,
    /// Not persisted: a workspace always reopens un-maximized.
    #[serde(skip)]
    pub(crate) maximized: Option<NodeId>,
}

impl Workspace {
    pub fn new(name: impl Into<String>, tree: AreaTree) -> Self {
        Self { name: name.into(), mode: None, mode_default: false, tree, maximized: None }
    }

    /// Swaps in a whole new tree. Ends maximization: area ids of the old tree
    /// mean nothing in the new one, though they may look like valid ids.
    pub fn replace_tree(&mut self, tree: AreaTree) {
        self.tree = tree;
        self.maximized = None;
    }

    /// The maximized area, if it still exists. Closing or joining the area
    /// away silently ends maximization.
    pub fn maximized(&self) -> Option<NodeId> {
        self.maximized.filter(|&id| self.tree.area(id).is_some())
    }

    /// What should be drawn within `bounds`: every area, or just the maximized one.
    pub fn visible_areas(&self, bounds: UnitRect) -> Vec<(NodeId, UnitRect)> {
        match self.maximized() {
            Some(id) => vec![(id, bounds)],
            None => area_rects(&self.tree, bounds),
        }
    }
}
