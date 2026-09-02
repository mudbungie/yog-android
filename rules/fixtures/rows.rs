// Fixture for rules/unbounded-label-in-row.yml: a bare label inside a
// horizontal row, which extends and widens the column. Deliberately
// violating; never compiled (nothing declares this as a module).
fn row(ui: &mut egui::Ui, text: &str) {
    ui.horizontal(|ui| {
        ui.label(text);
        ui.colored_label(egui::Color32::WHITE, text);
        ui.weak(text);
    });
}
