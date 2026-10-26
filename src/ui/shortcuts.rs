use super::action::AreaAction;
use super::active::ActiveArea;

/// Layout-level keyboard shortcuts, acting on the active area.
pub(super) fn collect(ui: &egui::Ui, active: &ActiveArea, actions: &mut Vec<AreaAction>) {
    if ui.ctx().wants_keyboard_input() {
        return;
    }
    let maximize = ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Space));
    if let (true, Some(area)) = (maximize, active.get()) {
        actions.push(AreaAction::ToggleMaximize { area });
    }
}
