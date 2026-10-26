//! Nested, id-free serde form of a [`Tree`]. Keeps the file format independent of
//! slotmap internals (and of TOML's inability to hold the null slots a raw
//! `SlotMap` serializes).

use crate::ops::clamp_ratio;
use crate::{Area, Axis, EditorKind, Node, NodeId, Tree};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum NodeDef {
    Split { axis: Axis, ratio: f32, first: Box<NodeDef>, second: Box<NodeDef> },
    Area { kind: EditorKind },
}

fn to_def(tree: &Tree, id: NodeId) -> NodeDef {
    match tree.node(id).expect("tree links are consistent") {
        Node::Split { axis, ratio, first, second } => NodeDef::Split {
            axis: *axis,
            ratio: *ratio,
            first: Box::new(to_def(tree, *first)),
            second: Box::new(to_def(tree, *second)),
        },
        Node::Area(a) => NodeDef::Area { kind: a.kind.clone() },
    }
}

fn build(tree: &mut Tree, def: NodeDef) -> NodeId {
    match def {
        NodeDef::Area { kind } => tree.insert(Node::Area(Area { kind }), None),
        NodeDef::Split { axis, ratio, first, second } => {
            let first = build(tree, *first);
            let second = build(tree, *second);
            let ratio = clamp_ratio(ratio).unwrap_or(0.5);
            let id = tree.insert(Node::Split { axis, ratio, first, second }, None);
            tree.set_parent(first, Some(id));
            tree.set_parent(second, Some(id));
            id
        }
    }
}

impl Serialize for Tree {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        to_def(self, self.root()).serialize(s)
    }
}

impl<'de> Deserialize<'de> for Tree {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let mut tree = Tree::unrooted();
        let root = build(&mut tree, NodeDef::deserialize(d)?);
        tree.set_root(root);
        Ok(tree)
    }
}

#[cfg(test)]
mod tests {
    use crate::ops::split_area;
    use crate::{Axis, EditorKind, Tree};

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Doc {
        tree: Tree,
    }

    #[test]
    fn toml_roundtrip_preserves_structure() {
        let k = |s| EditorKind::new(s);
        let mut tree = Tree::single(k("sketch_2d"));
        let root = tree.root();
        let right = split_area(&mut tree, root, Axis::Horizontal, 0.3, k("properties")).unwrap();
        split_area(&mut tree, right, Axis::Vertical, 0.6, k("status")).unwrap();

        let text = toml::to_string(&Doc { tree: tree.clone() }).unwrap();
        let back: Doc = toml::from_str(&text).unwrap();
        back.tree.assert_valid();

        let kinds = |t: &Tree| -> Vec<String> {
            t.areas().iter().map(|&a| t.area(a).unwrap().kind.id().to_string()).collect()
        };
        assert_eq!(kinds(&back.tree), kinds(&tree));
        assert_eq!(kinds(&tree), ["sketch_2d", "properties", "status"]);
    }

    #[test]
    fn out_of_range_ratio_in_file_is_clamped() {
        let text = r#"
            [tree]
            type = "split"
            axis = "horizontal"
            ratio = 7.0
            [tree.first]
            type = "area"
            kind = "a"
            [tree.second]
            type = "area"
            kind = "b"
        "#;
        let doc: Doc = toml::from_str(text).unwrap();
        doc.tree.assert_valid();
        let crate::Node::Split { ratio, .. } = doc.tree.node(doc.tree.root()).unwrap() else { panic!() };
        assert!(*ratio < 1.0);
    }
}
