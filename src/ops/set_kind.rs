use crate::{EditorKind, NodeId, OpError, Tree};

/// Switches what `area` shows. A different kind gets a fresh view, since view
/// state is specific to an editor type; the same kind changes nothing.
pub fn set_area_kind(tree: &mut Tree, area: NodeId, kind: EditorKind) -> Result<(), OpError> {
    if !tree.contains(area) {
        return Err(OpError::NotFound(area));
    }
    if tree.area(area).ok_or(OpError::NotAnArea(area))?.kind == kind {
        return Ok(());
    }
    let view = tree.new_view();
    let a = tree.area_mut(area).expect("checked");
    a.kind = kind;
    a.view = view;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_changes_and_non_areas_are_refused() {
        let mut tree = Tree::single(EditorKind::new("a"));
        let a = tree.root();
        set_area_kind(&mut tree, a, EditorKind::new("b")).unwrap();
        assert_eq!(tree.area(a).unwrap().kind.id(), "b");
        let view = tree.area(a).unwrap().view;
        set_area_kind(&mut tree, a, EditorKind::new("b")).unwrap();
        assert_eq!(tree.area(a).unwrap().view, view, "same kind keeps the view");
        set_area_kind(&mut tree, a, EditorKind::new("c")).unwrap();
        assert_ne!(tree.area(a).unwrap().view, view, "new kind gets a fresh view");
        crate::ops::split_area(&mut tree, a, crate::Axis::Vertical, 0.5, EditorKind::new("c")).unwrap();
        let root = tree.root();
        assert_eq!(set_area_kind(&mut tree, root, EditorKind::new("x")), Err(OpError::NotAnArea(root)));
    }
}
