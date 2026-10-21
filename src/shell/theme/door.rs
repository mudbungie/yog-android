//! **A door** (DESIGN §23 item 14; lernie §4.44 item 5's `door`): a run of
//! words that is a control — the rule's knobs and its name. Bare ground at
//! rest and `RAISED` while a thumb is down on it (the phone's lit, §23 item
//! 8), the touch floor tall, its words at the left in their ink and, where
//! it opens something, its mark inside the same seat in its own ink: one
//! tap target, the mark the affordance and never the name.
//!
//! It replaced the rule's filled chips (`anatomy::chip`), whose fill said
//! *button* on every knob at rest and whose caret was a shape painted over
//! the words after the fact.

use eframe::egui;

use super::rgb;
use crate::theme;

/// **The air either side of a door's words**, inside its seat — the `XS`
/// step: a run on the rule is a run of text, and its room is the target's.
pub(crate) const PAD: f32 = theme::space::XS;

/// **One door**: `words` in `ink`, then `mark` in its own ink where there
/// is one, laid in a seat `size` across and tall, the words eliding where
/// the seat cannot hold them and the mark never.
pub(crate) fn door(
    ui: &mut egui::Ui,
    (words, ink): (&str, egui::Color32),
    mark: Option<(&str, egui::Color32)>,
    size: egui::Vec2,
    help: &str,
) -> egui::Response {
    let (rect, seat) = ui.allocate_exact_size(size, egui::Sense::click());
    super::told::told(ui, &seat, help);
    if !ui.is_rect_visible(rect) {
        return seat;
    }
    if seat.is_pointer_button_down_on() {
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(theme::RADIUS),
            rgb(theme::RAISED),
        );
    }
    let marked = mark.map(|(glyph, tint)| laid(ui, &format!(" {glyph}"), tint, None));
    let room = size.x - PAD * 2.0 - marked.as_ref().map_or(0.0, |m| m.size().x);
    let said = laid(ui, words, ink, Some(room.max(0.0)));
    let left = rect.left() + PAD;
    let at = |galley: &egui::Galley, x: f32| egui::pos2(x, rect.center().y - galley.size().y / 2.0);
    let after = left + said.size().x;
    ui.painter().galley(at(&said, left), said, ink);
    if let Some(marked) = marked {
        ui.painter().galley(at(&marked, after), marked, ink);
    }
    seat
}

/// **A run that is not a door** — a diagnostic, a separator: `words` in
/// `ink`, `inset` in from the left of a seat `size` across and tall, eliding
/// where the seat cannot hold them. It takes no tap.
pub(crate) fn run(
    ui: &mut egui::Ui,
    (words, ink): (&str, egui::Color32),
    size: egui::Vec2,
    inset: f32,
) {
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let said = laid(ui, words, ink, Some((size.x - inset * 2.0).max(0.0)));
    let at = egui::pos2(rect.left() + inset, rect.center().y - said.size().y / 2.0);
    ui.painter().galley(at, said, ink);
}

/// **How wide `words` are** at the rule's type, with nothing around them.
pub(crate) fn wide(ui: &egui::Ui, words: &str) -> f32 {
    laid(ui, words, rgb(theme::INK), None).size().x
}

/// **How wide a door would like to be**: its words, its air and its mark.
pub(crate) fn wants(ui: &egui::Ui, words: &str, mark: Option<&str>) -> f32 {
    let mark = mark.map_or(0.0, |glyph| wide(ui, &format!(" {glyph}")));
    wide(ui, words) + PAD * 2.0 + mark
}

/// `text` at the rule's type in `ink`, cut to `room` where one is given.
fn laid(
    ui: &egui::Ui,
    text: &str,
    ink: egui::Color32,
    room: Option<f32>,
) -> std::sync::Arc<egui::Galley> {
    let (wrap, room) = room.map_or((egui::TextWrapMode::Extend, f32::INFINITY), |room| {
        (egui::TextWrapMode::Truncate, room)
    });
    egui::WidgetText::from(egui::RichText::new(text).color(ink)).into_galley(
        ui,
        Some(wrap),
        room,
        egui::TextStyle::Button,
    )
}
