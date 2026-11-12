//! **The offers surface** (yog REMOTE §15.1, DESIGN §13.18): the registration
//! URL this seat just minted — as text and as the symbol the next device
//! scans — and the workspace's pending offers, each with its state, its
//! confirmation word, and the two acts that settle it.
//!
//! **It covers the admin screen while it stands**, under its own name, and
//! its back control closes it. What it replaced held a private key on a
//! display and had to be forgotten for safety's sake; this holds a URL that
//! carries no key and works once, so closing is only closing.
//!
//! **Approve is dark until a row is proved** (`OfferRow::approvable`): the
//! server refuses an approval of anything else, and the operator's whole job
//! is to read the word off the new device and approve the row that matches.
//! The rows are re-read every few seconds while the surface stands, because a
//! row's state moves on the new device's clock, not this one's.
//!
//! Android-only glue, excluded from coverage: what a row says and enables is
//! `codec::offer`, what each act sends and the read behind it is
//! `seat::acts::offer` and `worker::after`, and the symbol's modules and pitch
//! are `crate::symbol` — all pure and asserted by host tests.

use eframe::egui;

use crate::codec::{OfferRow, Offered, Pending, Settle};
use crate::shell::app::Shell;
use crate::shell::mark::TOUCH;
use crate::symbol::{QUIET, Symbol, encode, pitch};

/// The surface's name and the harness's tap target (§15.2).
pub(in crate::shell) const SCREEN: &str = crate::overlay::OFFERS;

/// How often the rows are re-read while the surface stands, in seconds.
const EVERY: f64 = 4.0;

/// What an operator does with a minted URL, said once.
const SAID: &str = "scan this from the new device, or paste the URL there — it works once, \
                    and for ten minutes while nobody has claimed it. It carries no key.";

impl Shell {
    /// The covering surface.
    pub(super) fn offers(&mut self, ui: &mut egui::Ui, pending: &Pending) {
        self.note_screen(SCREEN);
        self.reread_offers(ui);
        egui::ScrollArea::vertical().show(ui, |ui| {
            if let Some((name, offered)) = &pending.minted {
                minted(ui, name, offered);
                ui.separator();
            }
            ui.weak("pending offers — approve the row whose word the new device shows");
            if pending.rows.is_empty() {
                ui.weak("nothing is pending.");
            }
            for row in &pending.rows {
                self.offer_row(ui, row);
            }
        });
    }

    /// Ask for the rows again when the last asking is old enough.
    fn reread_offers(&self, ui: &egui::Ui) {
        let now = ui.input(|i| i.time);
        let slot = egui::Id::new("offers-asked");
        let asked = ui.ctx().data(|data| data.get_temp::<f64>(slot));
        if asked.is_some_and(|asked| (0.0..EVERY).contains(&(now - asked))) {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_secs(1));
            return;
        }
        ui.ctx().data_mut(|data| data.insert_temp(slot, now));
        if let (Some(_), Some(model)) = (asked, self.model()) {
            model.offers();
        }
    }

    /// One pending offer: its line, and the two acts under it.
    fn offer_row(&self, ui: &mut egui::Ui, row: &OfferRow) {
        ui.add_space(4.0);
        crate::shell::clip::said(ui, row.said());
        let band = egui::vec2(ui.available_width(), TOUCH);
        ui.allocate_ui_with_layout(
            band,
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                let wide = crate::shell::theme::share(ui, 2);
                for (settle, live, help) in [
                    (
                        Settle::Approve,
                        row.approvable(),
                        "seats this device in the workspace — dark until the device has dialled",
                    ),
                    (
                        Settle::Decline,
                        true,
                        "voids this offer; a claimed one's certificate is refused for good",
                    ),
                ] {
                    let op = settle.op();
                    let control =
                        crate::shell::theme::chip(ui, op.clone().into(), wide, live, help);
                    crate::shell::act::act(ui, &control, &op);
                    if control.clicked()
                        && let Some(model) = self.model()
                    {
                        model.settle(settle, row.name.clone());
                    }
                }
            },
        );
    }
}

/// The URL this seat just minted: who it is for, the text, and the symbol.
fn minted(ui: &mut egui::Ui, name: &str, offered: &Offered) {
    ui.weak(format!("offer for {name}"));
    crate::shell::clip::said(ui, &offered.url);
    ui.weak(SAID);
    match encode(&offered.url) {
        Ok(symbol) => paint(ui, &symbol),
        // A URL the format cannot carry is still on the glass as text, and
        // the sentence says why there is no symbol under it.
        Err(why) => {
            ui.colored_label(crate::shell::theme::ink(crate::theme::State::Error), why);
        }
    }
}

/// **The symbol as one mesh at a whole-pixel pitch** (`crate::symbol`, and
/// lernie bl-5e0e's ruling): egui feathers every fill by a device pixel, so a
/// rectangle per module at a fractional origin spends the contrast a decoder
/// needs on anti-aliasing. One mesh, and the rule that sizes it is pure.
fn paint(ui: &mut egui::Ui, symbol: &Symbol) {
    let side = ui.available_width().min(ui.available_height()).max(TOUCH);
    let pitch = pitch(side, ui.ctx().pixels_per_point(), symbol.modules);
    let across = symbol.modules + QUIET * 2;
    let span = pitch * across as f32;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(span, span), egui::Sense::hover());
    // The quiet zone is light, so the whole square is painted light first and
    // the dark modules are laid on it — which is also what a decoder expects
    // to find around the symbol.
    ui.painter().rect_filled(rect, 0.0, egui::Color32::WHITE);
    let mut mesh = egui::Mesh::default();
    for y in 0..symbol.modules {
        for x in 0..symbol.modules {
            if !symbol.dark(x, y) {
                continue;
            }
            let at = rect.min + egui::vec2((x + QUIET) as f32 * pitch, (y + QUIET) as f32 * pitch);
            mesh.add_colored_rect(
                egui::Rect::from_min_size(at, egui::vec2(pitch, pitch)),
                egui::Color32::BLACK,
            );
        }
    }
    ui.painter().add(mesh);
}
