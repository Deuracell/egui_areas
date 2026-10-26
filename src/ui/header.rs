use super::action::AreaAction;
use super::host::AreaHost;
use crate::{Axis, EditorKind, NodeId};

pub(super) const HEADER_HEIGHT: f32 = 24.0;

/// What a header needs to know about its area to offer the right choices.
pub(super) struct HeaderInfo {
    pub area: NodeId,
    pub same_kind_count: usize,
    /// The neighbour is a single area, so it can be absorbed.
    pub can_absorb: bool,
    /// The area has a neighbour to give its space to.
    pub can_merge: bool,
    pub maximized: bool,
}

/// Draws the strip above an area's body: editor dropdown, badge, context menu.
pub(super) fn draw_header(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    kind: &EditorKind,
    info: &HeaderInfo,
    active: bool,
    host: &mut dyn AreaHost,
    actions: &mut Vec<AreaAction>,
) {
    ui.painter().rect_filled(rect, 0.0, ui.visuals().faint_bg_color);
    let area = info.area;

    // Registered before the dropdown so the dropdown wins clicks on top of it.
    let background = ui.interact(rect, ui.id().with(("header", area)), egui::Sense::click());
    background.context_menu(|ui| context_menu(ui, info, actions));

    // Controls are one interact-height tall and centred, so this is also the
    // gap above and below them: the left margin matches.
    let margin = ((rect.height() - ui.spacing().interact_size.y) / 2.0).max(0.0);
    let mut strip = ui.new_child(
        egui::UiBuilder::new()
            .id_salt(("header_strip", area))
            .max_rect(rect.shrink2(egui::vec2(margin, 0.0)))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    egui::ComboBox::from_id_salt(("editor_kind", area))
        .selected_text(egui::RichText::new(host.editor_name(kind)).small().strong())
        .show_ui(&mut strip, |ui| {
            for (k, name) in host.editor_kinds() {
                if ui.selectable_label(&k == kind, name).clicked() {
                    actions.push(AreaAction::SetKind { area, kind: k });
                }
            }
        });
    if let Some(note) = host.header_note(kind, info.same_kind_count) {
        strip.label(egui::RichText::new(note).small().weak());
    }
    strip.set_clip_rect(rect.intersect(ui.clip_rect()));
    host.show_header_extras(&mut strip, area, kind, active);
}

fn context_menu(ui: &mut egui::Ui, info: &HeaderInfo, actions: &mut Vec<AreaAction>) {
    let area = info.area;
    let mut choose = |ui: &mut egui::Ui, enabled: bool, label: &str, action: AreaAction| {
        if ui.add_enabled(enabled, egui::Button::new(label)).clicked() {
            actions.push(action);
            ui.close();
        }
    };
    choose(ui, true, "Split into Left | Right", AreaAction::Split { area, axis: Axis::Horizontal });
    choose(ui, true, "Split into Top / Bottom", AreaAction::Split { area, axis: Axis::Vertical });
    ui.separator();
    choose(ui, info.can_absorb, "Absorb neighbour", AreaAction::AbsorbNeighbour { area });
    choose(ui, info.can_merge, "Merge into neighbour", AreaAction::MergeIntoNeighbour { area });
    ui.separator();
    let label = if info.maximized { "Restore" } else { "Maximize" };
    choose(ui, true, label, AreaAction::ToggleMaximize { area });
}
