//! **The composer row**: the one editable field plus its send control, at
//! both depths (DESIGN §8). Split from `chat` on the seam between saying and
//! showing — that file paints what the engine wrote down, this one is the
//! control an operator writes with.
//!
//! **The field itself is not egui's** (bl-8bbb). It is a native Android
//! `EditText` overlaid at the rectangle this row lays out for it, synced
//! through `shell::field`, and the reason is the whole of the operator's
//! complaint: an egui `TextEdit` fed by the `GameTextInput` mirror adopts the
//! IME's committed buffer wholesale, so a letter typed with the cursor in the
//! middle of a draft landed at the end, nothing was selectable, and there was
//! no paste menu. What this file still owns is the ROW — where the field
//! stands, how tall it may grow, and the send beside it.

use eframe::egui;

use super::app::Shell;

/// **The field's own padding, and with it the field's own resting height**
/// (bl-01a6). A field at rest is one text row inside a two-point margin —
/// nineteen points of box, which at the bottom of a forty-eight point band
/// reads as a thin line pressed into a corner, and is not a target a thumb
/// can hit. So the padding is derived rather than chosen: half the difference
/// between the §13.2 touch floor and one line of body text, top and bottom,
/// which makes the resting field exactly the floor **and** centres the hint
/// in it rather than sitting it on a baseline. Derived and not a constant
/// because the line is the platform's — a device with larger text gets a
/// larger field, and the floor is never the thing that gives.
///
/// **Shared with the search field** (§13.6, bl-4c2b) and with the five
/// single-purpose fields that borrow the composer's widget id, which are
/// still egui's and have the same problem: a `TextEdit` at rest is a thin box
/// that a thumb aimed at its row misses. One home for *a field at rest is the
/// touch floor*, rather than a second derivation that would drift the first
/// time the floor moves. The native field is dressed from it too, in device
/// pixels.
pub(super) fn padding(ui: &egui::Ui) -> egui::Margin {
    let line = ui.text_style_height(&egui::TextStyle::Body);
    let pad = ((super::mark::TOUCH - line) / 2.0).max(SIDE_PAD).round() as i8;
    egui::Margin::symmetric(SIDE_PAD as i8, pad)
}

/// The breathing room either side of the text, and the floor under the
/// derived vertical padding — a hint hard against the frame reads as a
/// cramped box however tall it is.
const SIDE_PAD: f32 = 8.0;

impl Shell {
    /// The composer row: the one editable field plus a send control, shared
    /// by the transcript's composer and the conversation starter — the same
    /// gesture at two depths (DESIGN §8). Returns the taken text when a send
    /// happened.
    ///
    /// The button exists because Enter is not a control a phone can be
    /// promised (bl-9196): the IME's action key is the keyboard's to
    /// interpret, and a message that can be typed but not sent is a chat app
    /// that does not chat. On this field enter breaks the line, which is what
    /// every phone chat app does with it.
    ///
    /// **The row is allocated its own height, never the screen's remainder**
    /// (bl-193c). Both callers paint bottom-up, so a row that asks for what
    /// is left is handed the whole rest of the screen. The band is the
    /// field's own last measurement, floored at the touch target and capped
    /// (`crate::draft::band`) — a measurement rather than a guess, because
    /// the text is laid out by the platform now and only the platform knows
    /// how tall it came out.
    ///
    /// `acts` is what the send control fires, and it is the caller's because
    /// the two depths differ there and nowhere else: the transcript's
    /// composer posts `message`, while the starter stages and fires as one
    /// gesture (`prepare` then `prompt`, `seat::acts::started`) and so
    /// carries both tags on the one button an operator can see (PARITY §4).
    pub(super) fn compose(
        &mut self,
        ui: &mut egui::Ui,
        hint: &str,
        acts: &[&str],
    ) -> Option<String> {
        let band = egui::vec2(ui.available_width(), self.field.band());
        let mut taken = None;
        ui.allocate_ui_with_layout(
            band,
            egui::Layout::right_to_left(egui::Align::BOTTOM),
            |ui| {
                // Laid right-to-left so the button claims its seat first and
                // the field takes what remains, not the whole row;
                // bottom-aligned so the button sits on the band's floor
                // beside a field that fills the band.
                let control =
                    egui::Button::new("send").min_size(egui::vec2(0.0, super::mark::TOUCH));
                let send = ui.add(control);
                super::act::acts(ui, &send, acts);
                let pressed = send.clicked();
                // What is left of the band is the field's, and egui paints
                // nothing in it: the platform's own view stands there.
                let (rect, _) = ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());
                self.overlay(ui, hint, rect);
                if pressed && !self.composer.trim().is_empty() {
                    taken = Some(std::mem::take(&mut self.composer));
                    self.field.took(&self.android);
                }
            },
        );
        taken
    }

    /// Put the native field where the row put it, and take the draft back.
    ///
    /// **A popup takes the glass instead** (§13.2's opened-list rule). A
    /// platform view is drawn over the GL surface by the window compositor
    /// and knows nothing of egui's areas, so a list opened over the composer
    /// — the controls band's selectors open upward, directly across it —
    /// would paint UNDER the field. The field steps off the glass for that
    /// frame, which is the one arrangement in which the topmost thing on the
    /// screen is the thing the operator opened.
    fn overlay(&mut self, ui: &mut egui::Ui, hint: &str, rect: egui::Rect) {
        if egui::Popup::is_any_open(ui.ctx()) {
            return;
        }
        let ppp = ui.ctx().pixels_per_point();
        let px = |v: f32| (v * ppp).round() as i32;
        let at = [
            px(rect.left()),
            px(rect.top()),
            px(rect.width()),
            px(rect.height()),
        ];
        let skin = super::theme::skin(padding(ui), ppp);
        // **Where the harness types** (§15.2). The field carries no
        // accessibility node the walk can find by name any more than an egui
        // widget does — it is one view inside an app whose tree is a single
        // opaque surface — so the rectangle the app states is how a walk
        // reaches it: tap here, then `adb shell input text`.
        self.note_control("composer", ui, rect);
        if self
            .field
            .frame(&self.android, hint, at, skin, &mut self.composer)
        {
            // A native view's edits wake no egui frame, so the mirror is
            // read at the repaint cadence while the caret is in the field —
            // the same focus-gated poll DESIGN §3 rules for the IME bridge,
            // for the same reason.
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(16));
        }
    }
}
