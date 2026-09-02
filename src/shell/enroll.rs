//! **The first-run surface**: the three bootstraps as branded, tappable
//! choices, and the screen behind each tap (bl-0d3c, amending bl-7714).
//!
//! bl-7714 shipped this screen with **no button, deliberately**, reasoning
//! from REMOTE §1.4. The reading was too strong and the operator has ruled:
//! §1.4 forbids the app *dialling unauthenticated* —
//!
//! > *"there is no pairing protocol in the wire, no token exchange a stranger
//! > on the network could initiate. Bootstrap is always an act performed
//! > through existing trust (a cable, an authenticated route, a screen), and
//! > the first thing the new device does with its material is an ordinary mTLS
//! > dial like any other client."*
//!
//! — and it never forbade a control. An operator who opens the app is entitled
//! to be told what each bootstrap is and taken to the screen that explains
//! it. **The buttons choose and inform; the material still arrives out of
//! channel**, and not one widget here opens a socket.
//!
//! **Nothing on this screen is stored.** [`Shell::chose`] is which screen is
//! open right now, no more durable than a scroll position; the component this
//! device runs is still derived from the leaf on disk every boot
//! ([`crate::bootstrap::standing`]). *Check for material* re-runs that
//! derivation — a read of this app's own storage, which is what makes a cable
//! push land without a restart.

use eframe::egui;

use super::app::Shell;
use super::boot::Running;
use crate::bootstrap::{Component, Offer};

mod material;
mod scan;

pub(crate) use scan::Scanner;

/// **What this device stands at, for the screen that would change it**: where
/// material goes, the sentence a half-provisioned store earns, and the
/// identity a landing would replace. One parameter rather than three, because
/// they are one question — what is here now — and because a screen function
/// that grew a fourth of them would be a signature nobody reads (bl-f12d).
///
/// Owned and rebuilt per frame: it is three small strings off state the frame
/// already holds, and the house style takes the clone over threading a
/// borrow through a paint stack.
pub(super) struct Landing {
    /// This app's material directory, painted so an operator can act on it.
    pub(super) dir: String,
    /// The half-provisioned sentence, when there is one.
    pub(super) refusal: Option<String>,
    /// The running identity a landing would replace, or `None` on a device
    /// with nothing to lose.
    pub(super) replacing: Option<String>,
}

/// What a tap on an opened screen asked for. `Stay` is every frame in which
/// nothing was pressed; the way back is the bar's (bl-7a57), not this
/// screen's.
pub(super) enum Act {
    Stay,
    /// Re-run the boot derivation. Both controls that act ask for this and
    /// neither knows what it will find — landing an envelope and re-reading
    /// after a cable push are the same question, asked after the material
    /// changed.
    Recheck,
}

impl Shell {
    /// The configuration surface: the chooser, or the screen a choice
    /// opened. A cold device's whole screen, and the screen the yog mark
    /// opens over a running component (bl-387f) — the same offers either
    /// way, because re-provisioning IS provisioning: the material lands in
    /// the same directory and the same recheck derives what it now names.
    pub(super) fn configuration(&mut self, ui: &mut egui::Ui) {
        let (offers, refusal, dir, standing) = match &self.running {
            Running::Cold {
                offers,
                refusal,
                dir,
            } => (
                offers.clone(),
                refusal.clone(),
                dir.clone(),
                "nothing is provisioned, so nothing is running.".to_owned(),
            ),
            // Opened over a running component nothing carries the triple
            // along, so it is re-derived: the offers are a constant and the
            // directory is the one fact the boot read. The standing line
            // says what is running — the cold sentence would be a lie here.
            _ => (
                crate::bootstrap::offers(),
                None,
                self.material_dir(),
                format!("running: {}", self.identity()),
            ),
        };
        let Some(open) = self.chose else {
            // The bar's back is the way out, where a way out exists
            // (bl-e192): a cold device has nothing behind this surface to
            // return to, so it gets no back control at all.
            let out = !matches!(self.running, Running::Cold { .. });
            if self.bar(ui, "configuration", out) {
                self.forget_envelope();
                self.settings = false;
                return;
            }
            self.chose = chooser(ui, &offers, refusal.as_ref(), &dir, &standing);
            return;
        };
        // An offer for every component exists by construction; a missing one
        // is a chooser that offered what `offers` does not carry, and the
        // honest answer is the chooser again rather than a blank screen.
        let Some(offer) = offers.iter().find(|o| o.component == open) else {
            self.chose = None;
            return;
        };
        // The scan screen is the whole screen while it is up — a camera
        // preview under a bar belonging to a screen it covers reads as two
        // screens at once — so the bar stays down while the scanner is live.
        // Which leaves the scanner the only thing that can take a platform
        // back press while it is up, and closing the camera is exactly one
        // depth up from it (bl-550e).
        if self.scanner.live() {
            if std::mem::take(&mut self.back) {
                self.scanner.shut();
            }
        } else if self.bar(ui, &offer.brand.clone(), true) {
            self.forget_envelope();
            return;
        }
        // §13.1 W3, said to the one operator it is for: opening Thrall while
        // a seat runs is the moment before enrolling a second name this
        // device does not need — the seat already hosts tools beside the
        // asker, one identity on two connections (REMOTE §5).
        if open == Component::Foot && matches!(self.running, Running::Seat { .. }) {
            ui.colored_label(
                egui::Color32::LIGHT_YELLOW,
                format!(
                    "this device already offers its tools — it runs {}, and a \
                     Lernie seat hosts tools beside the asker. Thrall is for a \
                     device that should offer ONLY tools.",
                    self.identity()
                ),
            );
        }
        // What an envelope landed here would REPLACE, when there is
        // something to lose (bl-f12d). `None` on a cold device: nothing is
        // running, so there is no consequence to state — and the identity
        // line is the one that already knows, so this derives from it rather
        // than reading the leaf a second time.
        let landing = Landing {
            dir,
            refusal,
            replacing: match &self.running {
                Running::Cold { .. } => None,
                _ => Some(self.identity()),
            },
        };
        let scanner = &mut self.scanner;
        let (text, said) = (&mut self.envelope, &mut self.envelope_said);
        match opened(ui, offer, &landing, text, said, scanner) {
            Act::Stay => {}
            Act::Recheck => {
                self.forget_envelope();
                self.reboot();
            }
        }
    }
}

/// The three choices. Returns the one that was tapped.
fn chooser(
    ui: &mut egui::Ui,
    offers: &[Offer],
    refusal: Option<&String>,
    dir: &str,
    standing: &str,
) -> Option<Component> {
    ui.weak(standing);
    // A half-provisioned store is not an empty one, and the two must not read
    // the same: this device HAS something, and the sentence names every file
    // that is missing at once.
    if let Some(why) = refusal {
        ui.add_space(4.0);
        ui.colored_label(egui::Color32::LIGHT_RED, why);
    }
    ui.separator();
    ui.add_space(4.0);
    ui.weak("choose what this device becomes:");
    let mut chosen = None;
    egui::ScrollArea::vertical().show(ui, |ui| {
        for offer in offers.iter().filter(|o| o.default) {
            if choice(ui, offer, true) {
                chosen = Some(offer.component);
            }
        }
        ui.add_space(12.0);
        ui.weak("and one more, which is not the usual choice:");
        ui.add_space(4.0);
        for offer in offers.iter().filter(|o| !o.default) {
            if choice(ui, offer, false) {
                chosen = Some(offer.component);
            }
        }
        ui.add_space(12.0);
        ui.separator();
        // The path, once more and on its own line: it is the fact an operator
        // with a cable in their hand is actually here for.
        ui.weak(format!("material goes at {dir}"));
    });
    chosen
}

/// One branded choice: the name on the control, the line under it. Tapping it
/// opens a screen — it does not store the choice, and it does not dial.
fn choice(ui: &mut egui::Ui, offer: &Offer, emphasised: bool) -> bool {
    ui.add_space(8.0);
    let size = if emphasised { 24.0 } else { 19.0 };
    let brand = egui::RichText::new(&offer.brand).size(size).strong();
    let button = egui::Button::new(brand).min_size(egui::vec2(ui.available_width(), 52.0));
    let hit = ui.add(button).clicked();
    ui.weak(&offer.tagline);
    hit
}

/// The screen a choice opened. For the two enrollments it is what material is
/// needed and where it goes; for the server it is the recorded blockers.
fn opened(
    ui: &mut egui::Ui,
    offer: &Offer,
    landing: &Landing,
    text: &mut String,
    said: &mut Option<String>,
    scanner: &mut Scanner,
) -> Act {
    // The scan screen is the whole screen while it is up: a camera preview
    // under a heading and a back control belonging to a screen it is covering
    // reads as two screens at once.
    if scanner.live() {
        if material::scanned(ui, &landing.dir, text, said, scanner) {
            return Act::Recheck;
        }
        return Act::Stay;
    }
    let mut act = Act::Stay;
    ui.weak(&offer.tagline);
    ui.separator();
    egui::ScrollArea::vertical().show(ui, |ui| {
        if offer.component == Component::Server {
            ui.label(&offer.how);
            ui.add_space(8.0);
            ui.weak("this bootstrap starts nothing.");
            return;
        }
        if material::screen(ui, offer, landing, text, said, scanner) {
            act = Act::Recheck;
        }
    });
    act
}
