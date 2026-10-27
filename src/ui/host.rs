use crate::{EditorKind, NodeId, ViewId};

/// What the layout needs from the application: names for headers and the
/// editors' contents. Keeps this crate ignorant of what editors do.
pub trait AreaHost {
    /// Display name of an editor kind, for area headers and dropdowns.
    fn editor_name(&self, kind: &EditorKind) -> String;

    /// Every kind an area can be switched to, in dropdown order.
    fn editor_kinds(&self) -> Vec<(EditorKind, String)>;

    /// Short badge for an area header, e.g. when `same_kind_count` areas show
    /// the same document. `None` for no badge.
    fn header_note(&self, kind: &EditorKind, same_kind_count: usize) -> Option<String>;

    /// Lets the editor add controls to its area's header, after the dropdown
    /// and badge. `ui` is a left-to-right row; content beyond the header is clipped.
    /// `view` keys the area's view state (it follows the editor through swaps).
    fn show_header_extras(&mut self, ui: &mut egui::Ui, area: NodeId, view: ViewId, kind: &EditorKind, active: bool);

    /// Draws the editor for `area` into `ui`, which spans exactly the area's
    /// body. `active` is true for the one area the user is working in.
    fn show_editor(&mut self, ui: &mut egui::Ui, area: NodeId, view: ViewId, kind: &EditorKind, active: bool);
}
