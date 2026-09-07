// Fixture for rules/no-literal-colour.yml: a paint site spelling a colour of
// its own instead of reading the state's accent through `shell::theme`.
// Deliberately violating; never compiled (nothing declares this as a module).
fn banner(ui: &mut egui::Ui, error: &str) {
    ui.colored_label(egui::Color32::LIGHT_RED, error);
}

fn rule(ui: &mut egui::Ui, rect: egui::Rect) {
    ui.painter().rect_filled(rect, 0.0, egui::Color32::from_rgb(160, 112, 240));
}
