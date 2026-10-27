use crate::{ViewId, Workspace};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Why a layout-set operation was refused. The set is left unchanged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkspacesError {
    OutOfRange(usize),
    EmptyName,
    NameTaken,
    /// There must always be at least one layout.
    LastLayout,
    /// Only a layout tagged with a mode can be that mode's default.
    NoMode,
}

impl fmt::Display for WorkspacesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfRange(i) => write!(f, "no layout at position {i}"),
            Self::EmptyName => write!(f, "a layout needs a name"),
            Self::NameTaken => write!(f, "another layout already has that name"),
            Self::LastLayout => write!(f, "cannot delete the last layout"),
            Self::NoMode => write!(f, "the layout has no mode"),
        }
    }
}

impl std::error::Error for WorkspacesError {}

/// The user's named layouts and which one is showing. Never empty, names are
/// unique. Knows nothing about windows, so a pop-out window can own one later.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "WorkspacesDef")]
pub struct Workspaces {
    workspaces: Vec<Workspace>,
    active: usize,
}

/// The stored form; turned into a valid [`Workspaces`] on load.
#[derive(Deserialize)]
struct WorkspacesDef {
    workspaces: Vec<Workspace>,
    #[serde(default)]
    active: usize,
}

impl TryFrom<WorkspacesDef> for Workspaces {
    type Error = String;

    /// Forgiving about a hand-edited file: duplicate names are renamed, a bad
    /// active index is clamped, repeated mode defaults are reduced to the first.
    fn try_from(def: WorkspacesDef) -> Result<Self, String> {
        let mut items = def.workspaces.into_iter();
        let first = items.next().ok_or("a layout file needs at least one layout")?;
        let mut set = Workspaces::new(first);
        for mut ws in items {
            ws.name = set.unique_name(&ws.name);
            ws.tree.refresh_views();
            set.workspaces.push(ws);
        }
        set.active = def.active.min(set.workspaces.len() - 1);
        set.normalize_defaults();
        Ok(set)
    }
}

impl Workspaces {
    pub fn new(first: Workspace) -> Self {
        let mut set = Self { workspaces: vec![first], active: 0 };
        set.normalize_defaults();
        set
    }

    pub fn len(&self) -> usize {
        self.workspaces.len()
    }

    pub fn is_empty(&self) -> bool {
        false
    }

    pub fn items(&self) -> &[Workspace] {
        &self.workspaces
    }

    pub fn active_index(&self) -> usize {
        self.active
    }

    pub fn active(&self) -> &Workspace {
        &self.workspaces[self.active]
    }

    pub fn active_mut(&mut self) -> &mut Workspace {
        &mut self.workspaces[self.active]
    }

    pub fn set_active(&mut self, index: usize) -> Result<(), WorkspacesError> {
        self.check(index)?;
        self.active = index;
        Ok(())
    }

    /// Adds `ws` at the end and returns its position. Its views are re-keyed so
    /// it can be a copy of a layout already in the set.
    pub fn add(&mut self, mut ws: Workspace) -> Result<usize, WorkspacesError> {
        self.check_new_name(&ws.name, None)?;
        ws.tree.refresh_views();
        ws.mode_default = false;
        self.workspaces.push(ws);
        Ok(self.workspaces.len() - 1)
    }

    /// Copies the layout at `index` under a fresh name and returns the copy's position.
    pub fn duplicate(&mut self, index: usize) -> Result<usize, WorkspacesError> {
        self.check(index)?;
        let mut copy = self.workspaces[index].clone();
        copy.name = self.unique_name(&format!("{} copy", copy.name));
        self.add(copy)
    }

    pub fn rename(&mut self, index: usize, name: &str) -> Result<(), WorkspacesError> {
        self.check(index)?;
        let name = name.trim();
        self.check_new_name(name, Some(index))?;
        self.workspaces[index].name = name.to_string();
        Ok(())
    }

    /// Deletes the layout at `index`. The one showing stays showing unless it
    /// was the one deleted, in which case its neighbour takes over.
    pub fn remove(&mut self, index: usize) -> Result<(), WorkspacesError> {
        self.check(index)?;
        if self.workspaces.len() == 1 {
            return Err(WorkspacesError::LastLayout);
        }
        self.workspaces.remove(index);
        if index < self.active || self.active == self.workspaces.len() {
            self.active -= 1;
        }
        self.normalize_defaults();
        Ok(())
    }

    /// Moves the layout at `from` so it ends up at `to`; the active one stays active.
    pub fn move_to(&mut self, from: usize, to: usize) -> Result<(), WorkspacesError> {
        self.check(from)?;
        self.check(to)?;
        let showing = self.active;
        let ws = self.workspaces.remove(from);
        self.workspaces.insert(to, ws);
        self.active = match showing {
            s if s == from => to,
            s if from < s && s <= to => s - 1,
            s if to <= s && s < from => s + 1,
            s => s,
        };
        Ok(())
    }

    /// Tags the layout with the kind of work it suits, or clears the tag.
    pub fn set_mode(&mut self, index: usize, mode: Option<String>) -> Result<(), WorkspacesError> {
        self.check(index)?;
        self.workspaces[index].mode = mode;
        self.normalize_defaults();
        Ok(())
    }

    /// Makes the layout the default for its mode, replacing any other.
    pub fn set_mode_default(&mut self, index: usize) -> Result<(), WorkspacesError> {
        self.check(index)?;
        let mode = self.workspaces[index].mode.clone().ok_or(WorkspacesError::NoMode)?;
        for ws in self.workspaces.iter_mut().filter(|w| w.mode.as_ref() == Some(&mode)) {
            ws.mode_default = false;
        }
        self.workspaces[index].mode_default = true;
        Ok(())
    }

    /// The layout to use for `mode`: its marked default, else the first one tagged with it.
    pub fn for_mode(&self, mode: &str) -> Option<usize> {
        let tagged = |w: &&Workspace| w.mode.as_deref() == Some(mode);
        let marked = self.workspaces.iter().position(|w| tagged(&w) && w.mode_default);
        marked.or_else(|| self.workspaces.iter().position(|w| tagged(&w)))
    }

    /// View keys of every area in every layout, so state of layouts that are
    /// not showing is kept.
    pub fn view_ids(&self) -> Vec<ViewId> {
        self.workspaces.iter().flat_map(|w| w.tree.view_ids()).collect()
    }

    /// `base`, or `base 2`, `base 3`, ... if taken.
    pub fn unique_name(&self, base: &str) -> String {
        let base = match base.trim() {
            "" => "Layout",
            b => b,
        };
        let taken = |n: &str| self.workspaces.iter().any(|w| w.name == n);
        if !taken(base) {
            return base.to_string();
        }
        (2..).map(|i| format!("{base} {i}")).find(|n| !taken(n)).expect("unbounded range")
    }

    fn check(&self, index: usize) -> Result<(), WorkspacesError> {
        if index < self.workspaces.len() { Ok(()) } else { Err(WorkspacesError::OutOfRange(index)) }
    }

    fn check_new_name(&self, name: &str, ignoring: Option<usize>) -> Result<(), WorkspacesError> {
        if name.trim().is_empty() {
            return Err(WorkspacesError::EmptyName);
        }
        let clash = self.workspaces.iter().enumerate().any(|(i, w)| Some(i) != ignoring && w.name == name);
        if clash { Err(WorkspacesError::NameTaken) } else { Ok(()) }
    }

    /// At most one default per mode, and none on an untagged layout.
    fn normalize_defaults(&mut self) {
        let mut seen: Vec<String> = Vec::new();
        for ws in &mut self.workspaces {
            match (&ws.mode, ws.mode_default) {
                (Some(m), true) if !seen.contains(m) => seen.push(m.clone()),
                _ => ws.mode_default = false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EditorKind, Tree};

    fn ws(name: &str) -> Workspace {
        Workspace::new(name, Tree::single(EditorKind::new("a")))
    }

    fn names(set: &Workspaces) -> Vec<&str> {
        set.items().iter().map(|w| w.name.as_str()).collect()
    }

    fn three() -> Workspaces {
        let mut set = Workspaces::new(ws("A"));
        set.add(ws("B")).unwrap();
        set.add(ws("C")).unwrap();
        set
    }

    #[test]
    fn names_must_be_unique_and_non_empty() {
        let mut set = three();
        assert_eq!(set.add(ws("B")), Err(WorkspacesError::NameTaken));
        assert_eq!(set.add(ws("  ")), Err(WorkspacesError::EmptyName));
        assert_eq!(set.rename(0, "B"), Err(WorkspacesError::NameTaken));
        set.rename(0, "A").unwrap(); // keeping its own name is fine
        set.rename(0, " Z ").unwrap();
        assert_eq!(names(&set), ["Z", "B", "C"]);
        assert_eq!(set.len(), 3, "failed operations change nothing");
    }

    #[test]
    fn views_are_unique_across_layouts() {
        let mut set = three();
        let copy = set.duplicate(0).unwrap();
        assert_eq!(set.items()[copy].name, "A copy");
        let mut views = set.view_ids();
        let total = views.len();
        views.sort();
        views.dedup();
        assert_eq!(views.len(), total, "a duplicate must not share views with its original");
    }

    #[test]
    fn remove_keeps_the_showing_layout_and_refuses_the_last() {
        let mut set = three();
        set.set_active(1).unwrap();
        set.remove(0).unwrap();
        assert_eq!(set.active().name, "B");
        set.remove(1).unwrap(); // C, after the showing one
        assert_eq!(set.active().name, "B");
        assert_eq!(set.remove(0), Err(WorkspacesError::LastLayout));
        let mut set = three();
        set.set_active(2).unwrap();
        set.remove(2).unwrap();
        assert_eq!(set.active().name, "B", "deleting the showing last layout falls back to its neighbour");
    }

    #[test]
    fn moving_keeps_the_same_layout_showing() {
        for from in 0..3 {
            for to in 0..3 {
                for showing in 0..3 {
                    let mut set = three();
                    set.set_active(showing).unwrap();
                    let name = set.active().name.clone();
                    set.move_to(from, to).unwrap();
                    assert_eq!(set.active().name, name, "move {from}->{to} with {showing} showing");
                }
            }
        }
        let mut set = three();
        set.move_to(0, 2).unwrap();
        assert_eq!(names(&set), ["B", "C", "A"]);
    }

    #[test]
    fn mode_default_is_unique_per_mode() {
        let mut set = three();
        assert_eq!(set.set_mode_default(0), Err(WorkspacesError::NoMode));
        set.set_mode(0, Some("sketch".into())).unwrap();
        set.set_mode(1, Some("sketch".into())).unwrap();
        set.set_mode(2, Some("part".into())).unwrap();
        assert_eq!(set.for_mode("sketch"), Some(0), "first tagged when none is marked");
        set.set_mode_default(1).unwrap();
        assert_eq!(set.for_mode("sketch"), Some(1));
        set.set_mode_default(2).unwrap();
        assert_eq!(set.for_mode("sketch"), Some(1), "another mode's default is separate");
        assert_eq!(set.for_mode("assembly"), None);
        set.set_mode(1, None).unwrap();
        assert_eq!(set.for_mode("sketch"), Some(0), "an untagged layout loses the default");
    }

    #[test]
    fn toml_roundtrip_and_forgiving_load() {
        let mut set = three();
        set.set_active(2).unwrap();
        let text = toml::to_string(&set).unwrap();
        let back: Workspaces = toml::from_str(&text).unwrap();
        assert_eq!(names(&back), ["A", "B", "C"]);
        assert_eq!(back.active_index(), 2);

        let messy = r#"
            active = 9
            [[workspaces]]
            name = "X"
            [workspaces.tree]
            type = "area"
            kind = "a"
            [[workspaces]]
            name = "X"
            [workspaces.tree]
            type = "area"
            kind = "a"
        "#;
        let loaded: Workspaces = toml::from_str(messy).unwrap();
        assert_eq!(names(&loaded), ["X", "X 2"]);
        assert_eq!(loaded.active_index(), 1);
        assert!(toml::from_str::<Workspaces>("workspaces = []").is_err());
    }
}
