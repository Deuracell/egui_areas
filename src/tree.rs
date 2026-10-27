use crate::{Area, EditorKind, Node, NodeId, ViewId};
use slotmap::SlotMap;

#[derive(Clone, Debug)]
struct Slot {
    parent: Option<NodeId>,
    node: Node,
}

/// Binary tree of splits and areas. Owns storage and read queries; the editing
/// operations live in [`crate::ops`].
#[derive(Clone, Debug)]
pub struct Tree {
    slots: SlotMap<NodeId, Slot>,
    root: NodeId,
    next_view: u64,
}

impl Tree {
    /// A tree with a single area filling everything.
    pub fn single(kind: EditorKind) -> Self {
        let mut tree = Self::unrooted();
        let view = tree.new_view();
        let root = tree.insert(Node::Area(Area { kind, view }), None);
        tree.set_root(root);
        tree
    }

    pub fn root(&self) -> NodeId {
        self.root
    }

    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.slots.get(id).map(|s| &s.node)
    }

    pub fn contains(&self, id: NodeId) -> bool {
        self.slots.contains_key(id)
    }

    pub fn area(&self, id: NodeId) -> Option<&Area> {
        match self.node(id)? {
            Node::Area(a) => Some(a),
            Node::Split { .. } => None,
        }
    }

    pub fn area_mut(&mut self, id: NodeId) -> Option<&mut Area> {
        match &mut self.slots.get_mut(id)?.node {
            Node::Area(a) => Some(a),
            Node::Split { .. } => None,
        }
    }

    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.slots.get(id)?.parent
    }

    /// The other child of `id`'s parent split.
    pub fn sibling(&self, id: NodeId) -> Option<NodeId> {
        match self.node(self.parent(id)?)? {
            Node::Split { first, second, .. } => Some(if *first == id { *second } else { *first }),
            Node::Area(_) => None,
        }
    }

    /// Area ids in depth-first (left/top first) order.
    pub fn areas(&self) -> Vec<NodeId> {
        let mut out = Vec::new();
        self.collect_areas(self.root, &mut out);
        out
    }

    fn collect_areas(&self, id: NodeId, out: &mut Vec<NodeId>) {
        match self.node(id) {
            Some(Node::Split { first, second, .. }) => {
                self.collect_areas(*first, out);
                self.collect_areas(*second, out);
            }
            Some(Node::Area(_)) => out.push(id),
            None => {}
        }
    }

    /// The view keys of all areas, in the same order as [`Tree::areas`].
    pub fn view_ids(&self) -> Vec<ViewId> {
        self.areas().iter().filter_map(|&a| self.area(a)).map(|a| a.view).collect()
    }

    // ---- mutation primitives for ops and deserialization ----

    pub(crate) fn unrooted() -> Self {
        Self { slots: SlotMap::with_key(), root: NodeId::default(), next_view: 0 }
    }

    /// A view key never handed out before by this tree.
    pub(crate) fn new_view(&mut self) -> ViewId {
        self.next_view += 1;
        ViewId(self.next_view)
    }

    pub(crate) fn insert(&mut self, node: Node, parent: Option<NodeId>) -> NodeId {
        self.slots.insert(Slot { parent, node })
    }

    pub(crate) fn remove(&mut self, id: NodeId) {
        self.slots.remove(id);
    }

    pub(crate) fn set_root(&mut self, id: NodeId) {
        self.root = id;
    }

    pub(crate) fn set_parent(&mut self, id: NodeId, parent: Option<NodeId>) {
        if let Some(slot) = self.slots.get_mut(id) {
            slot.parent = parent;
        }
    }

    pub(crate) fn node_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.slots.get_mut(id).map(|s| &mut s.node)
    }

    /// Points `parent`'s child link `old` at `new`. Does not touch `new`'s parent.
    pub(crate) fn replace_child(&mut self, parent: NodeId, old: NodeId, new: NodeId) {
        if let Some(Node::Split { first, second, .. }) = self.node_mut(parent) {
            if *first == old {
                *first = new;
            } else if *second == old {
                *second = new;
            }
        }
    }

    /// Panics unless links are consistent and every node is reachable from the root.
    #[cfg(test)]
    pub(crate) fn assert_valid(&self) {
        let mut seen = 0;
        let mut stack = vec![(self.root, None)];
        while let Some((id, parent)) = stack.pop() {
            seen += 1;
            assert_eq!(self.parent(id), parent, "parent link of {id:?}");
            if let Some(Node::Split { first, second, ratio, .. }) = self.node(id) {
                assert!(ratio.is_finite());
                stack.push((*first, Some(id)));
                stack.push((*second, Some(id)));
            }
        }
        assert_eq!(seen, self.slots.len(), "unreachable nodes left in storage");
    }
}
