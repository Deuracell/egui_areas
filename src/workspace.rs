use crate::layout::{Rect, area_rects};
use crate::{NodeId, Tree};
use serde::{Deserialize, Serialize};

/// A named layout: one tree of areas, optionally with one area maximized.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Workspace {
    pub name: String,
    pub tree: Tree,
    /// Not persisted: a workspace always reopens un-maximized.
    #[serde(skip)]
    pub(crate) maximized: Option<NodeId>,
}

impl Workspace {
    pub fn new(name: impl Into<String>, tree: Tree) -> Self {
        Self { name: name.into(), tree, maximized: None }
    }

    /// The maximized area, if it still exists. Closing or joining the area
    /// away silently ends maximization.
    pub fn maximized(&self) -> Option<NodeId> {
        self.maximized.filter(|&id| self.tree.area(id).is_some())
    }

    /// What should be drawn within `bounds`: every area, or just the maximized one.
    pub fn visible_areas(&self, bounds: Rect) -> Vec<(NodeId, Rect)> {
        match self.maximized() {
            Some(id) => vec![(id, bounds)],
            None => area_rects(&self.tree, bounds),
        }
    }
}
