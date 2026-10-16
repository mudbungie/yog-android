//! **The controls that were egui's own** (DESIGN §23 item 10): a button, a
//! dark-or-live button, a pick off a list, a check and a slider — each
//! painted here and nowhere else (`rules/no-bare-control.yml`), each taking
//! its help sentence and telling it on a long press ([`super::told`]).

use eframe::egui;

use super::told::told;
use crate::theme;

/// **A button**: `label` on a control at least `wide` across and the touch
/// floor tall (`wide` of zero is the words' own width).
pub(crate) fn button(
    ui: &mut egui::Ui,
    label: impl Into<egui::WidgetText>,
    wide: f32,
    help: &str,
) -> egui::Response {
    button_if(ui, true, label, wide, help)
}

/// **A button whose words stand at its left edge** (the sign-in row's,
/// STYLE §5): the words then a growing gap, so a full-width control does
/// not centre them. A `String` rather than atoms at the signature, because
/// egui's atom trait wants a named lifetime and this crate takes none.
pub(crate) fn button_led(
    ui: &mut egui::Ui,
    words: String,
    wide: f32,
    help: &str,
) -> egui::Response {
    let control = egui::Button::new((words, egui::Atom::grow()))
        .truncate()
        .min_size(egui::vec2(wide, theme::TOUCH));
    let response = ui.add(control);
    told(ui, &response, help);
    response
}

/// **A button that is dark unless `live`**: disabled, it stays on the glass
/// and still tells — what would light it is often the sentence.
pub(crate) fn button_if(
    ui: &mut egui::Ui,
    live: bool,
    label: impl Into<egui::WidgetText>,
    wide: f32,
    help: &str,
) -> egui::Response {
    let control = egui::Button::new(label).min_size(egui::vec2(wide, theme::TOUCH));
    let response = ui.add_enabled(live, control);
    told(ui, &response, help);
    response
}

/// **A word that is a control** (§22 item 3's `send`): the label on bare
/// ground, no frame, at least `wide` across and the touch floor tall.
pub(crate) fn word(
    ui: &mut egui::Ui,
    label: impl Into<egui::WidgetText>,
    wide: f32,
    help: &str,
) -> egui::Response {
    let control = egui::Button::new(label)
        .frame(false)
        .min_size(egui::vec2(wide, theme::TOUCH));
    let response = ui.add(control);
    told(ui, &response, help);
    response
}

/// **One row of a list to pick from** (a dropup's rows): the words, the
/// `RAISED` wash under a thumb, never a checked state — a pick closes the
/// list.
pub(crate) fn pick(
    ui: &mut egui::Ui,
    label: impl Into<egui::WidgetText>,
    help: &str,
) -> egui::Response {
    let response = ui.selectable_label(false, label);
    told(ui, &response, help);
    response
}

/// **A check**: on or off, its word beside it, the touch floor tall.
pub(crate) fn check(ui: &mut egui::Ui, on: &mut bool, word: &str, help: &str) -> egui::Response {
    let response = ui.add_sized(
        egui::vec2(ui.available_width(), theme::TOUCH),
        egui::Checkbox::new(on, word),
    );
    told(ui, &response, help);
    response
}

/// **A slider**: `value` between `floor` and `ceiling`, moved by `step`,
/// said with `unit` after it.
pub(crate) fn slider(
    ui: &mut egui::Ui,
    value: &mut f32,
    (floor, ceiling, step): (f32, f32, f32),
    unit: &str,
    help: &str,
) -> egui::Response {
    let response = ui.add(
        egui::Slider::new(value, floor..=ceiling)
            .step_by(f64::from(step))
            .suffix(unit),
    );
    told(ui, &response, help);
    response
}
