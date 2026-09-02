//! **The composer row**: the one editable field plus its send control, at
//! both depths (DESIGN §8). Split from `chat` on the seam between saying and
//! showing — that file paints what the engine wrote down, this one is the
//! control an operator writes with.

use eframe::egui;

/// How tall the field may grow before it scrolls inside itself, in points —
/// and, with the touch floor, the whole of what the row's own height may be.
/// The cap has one home: the band below is what enforces it, so the scroller
/// states no second maximum of its own.
const FIELD_CAP: f32 = 132.0;

/// **The field's own padding, and with it the field's own resting height**
/// (bl-01a6). A `TextEdit` at rest is one text row inside a two-point margin
/// — nineteen points of box, which at the bottom of a forty-four point band
/// reads as a thin line pressed into a corner, and is not a target a thumb
/// can hit. So the padding is derived rather than chosen: half the difference
/// between the §13.2 touch floor and one line of body text, top and bottom,
/// which makes the resting field exactly the floor **and** centres the hint
/// in it rather than sitting it on a baseline. Derived and not a constant
/// because the line is the platform's — a device with larger text gets a
/// larger field, and the floor is never the thing that gives.
///
/// The text is already the transcript's size and nothing here sets it: a
/// `TextEdit`'s default font selection resolves to `TextStyle::Body`, which
/// is what `chat::row` labels a body with.
fn padding(ui: &egui::Ui) -> egui::Margin {
    let line = ui.text_style_height(&egui::TextStyle::Body);
    let pad = ((super::mark::TOUCH - line) / 2.0).max(SIDE_PAD).round() as i8;
    egui::Margin::symmetric(SIDE_PAD as i8, pad)
}

/// The breathing room either side of the text, and the floor under the
/// derived vertical padding — a hint hard against the frame reads as a
/// cramped box however tall it is.
const SIDE_PAD: f32 = 8.0;

/// The composer row: the one editable field plus a send control, shared by
/// the transcript's composer and the conversation starter — the same gesture
/// at two depths, already sharing a widget id (DESIGN §8). Returns the taken
/// text when a send happened, and refocuses the field either way.
///
/// The button exists because Enter is not a control a phone can be promised
/// (bl-9196): the IME's action key is the keyboard's to interpret, and a
/// message that can be typed but not sent is a chat app that does not chat.
/// Enter stays as the second path where a keyboard offers it.
///
/// **The row is allocated its own height, never the screen's remainder**
/// (bl-193c). Both callers paint bottom-up, so a row that asks for what is
/// left is handed the whole rest of the screen — and the two children then
/// resolve that rect at opposite extremes: a `ScrollArea` anchors to the top
/// of what it is given (it never reads the cross alignment) while the
/// bottom-aligned button anchors to the floor. On device that is a field
/// under the header and a send button a screen below it. The band fixes the
/// rect before either child sees it.
pub(super) fn composer(ui: &mut egui::Ui, text: &mut String, hint: &str) -> Option<String> {
    // The band is the field's own last-painted height, floored at a touch
    // target and capped at the growth limit. Last frame's measurement,
    // because a widget's height is not knowable before it is laid out — and
    // it is the CONTENT height that is cached, which depends on the text and
    // the width but never on the band, so the loop converges instead of
    // pinning the field at whatever it was first given.
    let measured = egui::Id::new(super::app::COMPOSER.id).with("row-height");
    let grown = ui.data(|d| d.get_temp::<f32>(measured)).unwrap_or_default();
    let band = egui::vec2(
        ui.available_width(),
        grown.clamp(super::mark::TOUCH, FIELD_CAP),
    );
    let mut taken = None;
    ui.allocate_ui_with_layout(
        band,
        egui::Layout::right_to_left(egui::Align::BOTTOM),
        |ui| {
            // Laid right-to-left so the button claims its seat first and the
            // field's infinite width takes what remains, not the whole row;
            // bottom-aligned so the button sits on the band's floor beside a
            // field that fills the band.
            let control = egui::Button::new("send").min_size(egui::vec2(0.0, super::mark::TOUCH));
            let pressed = ui.add(control).clicked();
            // Multiline (bl-56d6): the IME's enter is a newline on this stack
            // (DESIGN §3 residual) and that is also simply what a phone chat
            // composer does with enter — so the field grows with its text to a
            // cap and scrolls inside it, and the button is the one send. A field
            // that grew unbounded would push the transcript off the glass.
            // `auto_shrink` off: the scroller fills the band it was sized
            // from, so the field's ink and the row are the same rectangle.
            // `min_scrolled_height` is the reason the field used to paint
            // under the gesture-nav bar (bl-9cfd): a vertical `ScrollArea`
            // refuses to be shorter than 64 points by default, so a 44-point
            // band was overflowed by 20 — and the overflow inherited this
            // row's BOTTOM alignment, which put the text at the very bottom
            // of it, exactly where the nav bar is. The band is the cap and
            // the floor both; a scroller that will not fit inside what it is
            // given is a scroller that decides the layout.
            let shown = egui::ScrollArea::vertical()
                .id_salt(super::app::COMPOSER.id)
                .min_scrolled_height(0.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(text)
                            .id(egui::Id::new(super::app::COMPOSER.id))
                            .desired_width(f32::INFINITY)
                            .desired_rows(1)
                            .margin(padding(ui))
                            .hint_text(hint),
                    )
                });
            ui.data_mut(|d| d.insert_temp(measured, shown.content_size.y));
            if pressed {
                if !text.trim().is_empty() {
                    taken = Some(std::mem::take(text));
                }
                shown.inner.request_focus();
            }
        },
    );
    taken
}
