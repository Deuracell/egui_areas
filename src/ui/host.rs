use crate::{EditorKind, NodeId};

/// What the layout needs from the application: names for headers and the
/// editors' contents. Keeps this crate ignorant of what editors do.
pub trait AreaHost {
    /// Display name of an editor kind, for area headers and dropdowns.
    fn editor_name(&self, kind: &EditorKind) -> String;

    /// Draws the editor for `area` into `ui`, which spans exactly the area's body.
    fn show_editor(&mut self, ui: &mut egui::Ui, area: NodeId, kind: &EditorKind);
}
