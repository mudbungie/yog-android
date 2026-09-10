//! **The anatomy** (`docs/STYLE.md` §5): the shapes every screen is built out
//! of — a list row, a line of more than one ink, a word with a fold mark, and
//! a band of chips. Split from `shell::theme` at the 300-line cap (bl-0691) on
//! the seam that file already read as: the language and its one installation
//! are there, and what is assembled from it is here.
//!
//! Nothing here decides anything about the world: a caller names the state and
//! the width, and this file paints them. The colours are the parent module's,
//! which is the one place a `Color32` is constructed
//! (`rules/no-literal-colour.yml`).

use eframe::egui;

use super::rgb;
use crate::theme::{self, space};

/// **A list row** (STYLE.md, *row*): full width, the touch floor tall, its
/// words at the left and centred in the band, a `RAISED` tint only while a
/// thumb is on it, and a hairline under it. No outline, and no centring — a
/// `Button` centres its text, which is how the roster read as two alignments.
pub(crate) fn row(ui: &mut egui::Ui, label: egui::WidgetText) -> egui::Response {
    let wide = ui.available_width();
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(wide, theme::TOUCH), egui::Sense::click());
    if !ui.is_rect_visible(rect) {
        return response;
    }
    if response.is_pointer_button_down_on() || response.hovered() {
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(theme::RADIUS),
            rgb(theme::RAISED),
        );
    }
    let galley = label.into_galley(
        ui,
        Some(egui::TextWrapMode::Truncate),
        wide - space::M * 2.0,
        egui::TextStyle::Body,
    );
    let at = egui::pos2(
        rect.left() + space::M,
        rect.center().y - galley.size().y / 2.0,
    );
    ui.painter().galley(at, galley, rgb(theme::INK));
    ui.painter().hline(
        rect.x_range(),
        rect.bottom(),
        egui::Stroke::new(1.0, rgb(theme::HAIRLINE)),
    );
    response
}

/// One line of text in more than one ink — a name in `INK`, its count in
/// `INK_WEAK`, its mark in an accent — laid as a single truncating row.
pub(crate) fn line(ui: &egui::Ui, parts: &[(String, egui::Color32)]) -> egui::text::LayoutJob {
    let font = egui::TextStyle::Body.resolve(ui.style());
    let mut job = egui::text::LayoutJob::default();
    for (text, color) in parts {
        job.append(
            text,
            0.0,
            egui::TextFormat {
                font_id: font.clone(),
                color: *color,
                ..Default::default()
            },
        );
    }
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    job
}

/// **A word and a fold mark, as one line.** The word takes the body face and
/// the MARK takes the monospace one, which is not a style choice: the
/// proportional face egui installs has no glyph for `▼` and paints a tofu box
/// where the fold's open state should be (measured on the emulator, bl-8c94),
/// while the monospace face the transcript's own toggle already uses has
/// both. One home for that fact, rather than each screen that wants a fold
/// mark discovering it again.
pub(crate) fn marked(
    ui: &egui::Ui,
    word: &str,
    mark: &str,
    ink: egui::Color32,
) -> egui::text::LayoutJob {
    let mut job = line(ui, &[(format!("{word} "), ink)]);
    job.append(
        mark,
        0.0,
        egui::TextFormat {
            font_id: egui::TextStyle::Monospace.resolve(ui.style()),
            color: ink,
            ..Default::default()
        },
    );
    job
}

/// **A chip** (STYLE.md, *a band of chips*): one control of a band that
/// divides its width equally, so the COUNT of controls is a fact the layout
/// cannot lose (bl-6e8a's argument) — laid out inside a child of exactly its
/// share, so a label too long for it elides against that share rather than
/// pushing its neighbours off the glass. `live` is whether it takes a tap;
/// a dark chip stays on the glass and says in its own words what would
/// light it (DESIGN §13.17).
pub(crate) fn chip(
    ui: &mut egui::Ui,
    label: egui::WidgetText,
    wide: f32,
    live: bool,
) -> egui::Response {
    let label = laid(ui, label, wide);
    ui.allocate_ui_with_layout(
        egui::vec2(wide, theme::TOUCH),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.add_enabled(
                live,
                egui::Button::new(label).min_size(egui::vec2(wide, theme::TOUCH)),
            )
        },
    )
    .inner
}

/// **A chip's words, laid to the width the chip was given** — here rather
/// than by the button, so the elision happens at exactly the number the row's
/// own width arithmetic spent (`shell::place::row`, bl-0691). A `Button`
/// truncates at the width it finds itself in, which is not always the width
/// it was promised, and a control that says `nu…` where `nudge` was measured
/// to fit is the layout disagreeing with itself.
fn laid(ui: &egui::Ui, label: egui::WidgetText, wide: f32) -> egui::WidgetText {
    let room = (wide - ui.spacing().button_padding.x * 2.0).max(0.0);
    label
        .into_galley(
            ui,
            Some(egui::TextWrapMode::Truncate),
            room,
            egui::TextStyle::Button,
        )
        .into()
}

/// **A dark chip that still takes a tap** (STYLE.md, *a band of chips*: a
/// dark chip says what would light it; bl-0691). Where the reason is a
/// sentence rather than two words, a chip's width cannot carry it and a phone
/// has no hover to say it in — so the control keeps its tap and the caller
/// spends it on the sentence, which is the only way this app can be ASKED
/// *why is this dark?*.
///
/// It is [`chip`]'s twin and not a parameter of it, because the two answer
/// different questions: `chip(.., live: false)` is a control that must not
/// FIRE — every caller of it acts on a click — and this is a control that
/// fires nothing but an explanation. Dark by ink rather than by egui's
/// disabled state, which is what makes it clickable at all.
pub(crate) fn dark_chip(ui: &mut egui::Ui, label: String, wide: f32) -> egui::Response {
    let faint = laid(
        ui,
        egui::RichText::new(label)
            .color(rgb(theme::INK_FAINT))
            .into(),
        wide,
    );
    ui.allocate_ui_with_layout(
        egui::vec2(wide, theme::TOUCH),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| ui.add(egui::Button::new(faint).min_size(egui::vec2(wide, theme::TOUCH))),
    )
    .inner
}

/// The share each of `count` chips gets of the width a band has, the gaps
/// between them taken off first.
pub(crate) fn share(ui: &egui::Ui, count: usize) -> f32 {
    let each = count as f32;
    let gap = ui.spacing().item_spacing.x;
    ((ui.available_width() - gap * (each - 1.0)) / each).max(0.0)
}
