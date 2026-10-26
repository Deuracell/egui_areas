use serde::{Deserialize, Serialize};

/// Identifies which editor an area shows, e.g. `"sketch_2d"`.
///
/// An open string rather than an enum so the crate that defines editors
/// (`radcad-ui`) owns the list; this crate only stores and compares them.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EditorKind(String);

impl EditorKind {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn id(&self) -> &str {
        &self.0
    }
}
