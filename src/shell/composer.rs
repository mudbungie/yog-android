//! **The composer row**: the one editable field plus its send control, at
//! both depths (DESIGN §8). Split from `chat` on the seam between saying and
//! showing — that file paints what the server wrote down, this one is the
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
use super::theme::rgb;

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
        snap: &crate::seat::Snapshot,
        hint: &str,
        acts: &[&str],
    ) -> Option<String> {
        let band = egui::vec2(ui.available_width(), self.field.band());
        let mut taken = None;
        ui.allocate_ui_with_layout(
            band,
            // **Centred, and that is where the send's own word comes from**
            // (bl-0691): a `Button` positions its text by the ENCLOSING ui's
            // layout, so under a `BOTTOM` cross-align the word sat on the
            // floor of its box while the field's hint beside it was centred.
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                // Laid right-to-left so `send` claims its seat first, the
                // prompt door its square at the far left, and the field what
                // is left between them. **`send` is a word in the brand on
                // bare ground, painted only while the draft holds text**
                // (§22 item 3): with nothing to send there is no control.
                let pressed = !self.composer.is_empty() && {
                    let word = egui::RichText::new("send").color(rgb(crate::theme::BRAND));
                    let send = super::theme::word(
                        ui,
                        word,
                        super::mark::TOUCH,
                        "sends the line: prose to the conversation, a slash line as its verb",
                    );
                    super::act::acts(ui, &send, acts);
                    send.clicked()
                };
                // The door's square AND the gap before it: a right-to-left
                // row that forgets the gap lays the door that far past the
                // left edge, and the bottom-up block above widens to it —
                // the whole chat lost its left gutter that way.
                let gap = ui.spacing().item_spacing.x;
                let wide = (ui.available_width() - super::mark::TOUCH - gap).max(0.0);
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(wide, ui.available_height()),
                    egui::Sense::hover(),
                );
                self.door(ui);
                // What is left of the band is the field's, and egui paints
                // nothing in it: the platform's own view stands there.
                self.overlay(ui, hint, rect);
                if pressed && crate::verbs::slashing(&self.composer) {
                    self.send_line(snap);
                } else if pressed && !self.composer.trim().is_empty() {
                    taken = Some(std::mem::take(&mut self.composer));
                    self.field.took(&self.android);
                }
            },
        );
        taken
    }

    /// **`›`, the verb door** (§22 items 3 and 5): the prompt glyph in the
    /// brand, a touch-floor square, and a tap puts `/` at the head of the
    /// draft — or takes it back off — which is what opens the sheet.
    fn door(&mut self, ui: &mut egui::Ui) {
        let square = egui::vec2(super::mark::TOUCH, super::mark::TOUCH);
        let (rect, hit) = ui.allocate_exact_size(square, egui::Sense::click());
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            crate::theme::glyph::PROMPT,
            egui::FontId::proportional(crate::theme::type_scale::HEADING),
            rgb(crate::theme::BRAND),
        );
        self.note_control("prompt", ui, rect);
        if hit.clicked() {
            self.composer = crate::verbs::door(&self.composer);
        }
    }

    /// Put the native field where the row put it, and take the draft back.
    ///
    /// **A popup takes the glass instead** (§13.2's opened-list rule). A
    /// platform view is drawn over the GL surface by the window compositor
    /// and knows nothing of egui's areas, so a list opened over the composer
    /// — the rule's knobs stand under it and open upward, directly across it
    /// (§23 item 14) — would paint UNDER the field. The field steps off the
    /// glass for that frame, which is the one arrangement in which the
    /// topmost thing on the screen is the thing the operator opened. A told
    /// sentence is the same case: its box stands above the control, and a
    /// knob's is over the field. **The drawer is the same case** (§24 item
    /// 2): it stands on the foreground layer over the chat, and the field
    /// would stand over it.
    fn overlay(&mut self, ui: &mut egui::Ui, hint: &str, rect: egui::Rect) {
        let told =
            super::theme::told::standing(ui.ctx()) && self.knobs.on(crate::settings::Toggle::Help);
        if egui::Popup::is_any_open(ui.ctx()) || told || self.drawer.standing {
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
        let caret = self
            .field
            .frame(&self.android, hint, at, skin, &mut self.composer);
        if std::mem::take(&mut self.summon) {
            super::field::Native::summon(&self.android);
        }
        if caret {
            // A native view's edits wake no egui frame, so the mirror is
            // read at the repaint cadence while the caret is in the field —
            // the same focus-gated poll DESIGN §3 rules for the IME bridge,
            // for the same reason.
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(16));
        }
    }
}
