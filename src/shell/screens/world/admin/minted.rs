//! **The minted material, on the glass** (REMOTE §8.4, DESIGN §13.18): the
//! symbol the next device photographs, the three facts that are not secret,
//! and the control whose whole product is the forgetting.
//!
//! **It covers the admin screen while it stands**, which is the one place this
//! app allows a covering surface at all (lernie DESIGN §4.15's exception, and
//! for its reason): what it holds is a private key on a screen, the whole act
//! is *look at this now and close it*, and anything legible behind it would
//! invite the one thing the material must not have, which is a long life on a
//! display.
//!
//! **The key is never painted and never named.** What an operator needs to see
//! is that the mint happened and which device it was for; what the CAMERA
//! needs is the symbol. A field showing the PEM would be the material in two
//! places at once, and one of them would be a screenshot.
//!
//! **Leaving is forgetting.** The back control drops the material in the
//! worker's own memory — the only place it exists on this device — so there is
//! no state left to leak and nothing to write down.

use eframe::egui;

use crate::envelope::Envelope;
use crate::shell::app::Shell;
use crate::shell::mark::{Back, TOUCH};
use crate::symbol::{QUIET, Symbol, encode, pitch};

impl Shell {
    /// The covering surface. It answers whether it painted, so the screen
    /// under it knows to stand down.
    pub(super) fn minted(&mut self, ui: &mut egui::Ui, envelope: &Envelope) {
        self.note_screen(SCREEN);
        if self.bar(ui, SCREEN, &Back::To("config")) {
            self.forget();
        }
        ui.weak(format!(
            "{} · {} · {}",
            envelope.name,
            crate::codec::enroll::word(envelope.grade),
            envelope.address
        ));
        ui.weak(SAID);
        ui.separator();
        match encode(&crate::envelope::write(envelope)) {
            Ok(symbol) => paint(ui, &symbol),
            // A payload the format cannot carry is the engine's material and
            // this seat's problem to state, not to hide: the operator can
            // still mint again, and a blank square would say nothing.
            Err(why) => {
                ui.colored_label(egui::Color32::LIGHT_RED, why);
            }
        }
    }

    /// Drop the material — the act this surface exists to end with.
    fn forget(&mut self) {
        if let Some(model) = self.model() {
            model.forget();
        }
    }
}

/// The surface's name and the harness's tap target (§15.2).
pub(in crate::shell) const SCREEN: &str = "minted";

/// What an operator has to do while it is on the glass, said once.
const SAID: &str = "scan this from the new device now — leaving this screen forgets it, and \
                    the engine kept no key";

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
