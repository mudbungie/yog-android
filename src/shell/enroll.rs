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

/// What a tap on an opened screen asked for. `Stay` is every frame in which
/// nothing was pressed.
enum Act {
    Stay,
    Back,
    Recheck,
}

impl Shell {
    /// The cold device's whole screen: the chooser, or the screen a choice
    /// opened.
    pub(super) fn cold(&mut self, ui: &mut egui::Ui) {
        let Running::Cold {
            offers,
            refusal,
            dir,
        } = &self.running
        else {
            return;
        };
        let (offers, refusal, dir) = (offers.clone(), refusal.clone(), dir.clone());
        let Some(open) = self.chose else {
            self.chose = chooser(ui, &offers, refusal.as_ref(), &dir);
            return;
        };
        // An offer for every component exists by construction; a missing one
        // is a chooser that offered what `offers` does not carry, and the
        // honest answer is the chooser again rather than a blank screen.
        let Some(offer) = offers.iter().find(|o| o.component == open) else {
            self.chose = None;
            return;
        };
        match opened(ui, offer, &dir, refusal.as_ref()) {
            Act::Stay => {}
            Act::Back => self.chose = None,
            Act::Recheck => self.reboot(),
        }
    }
}

/// The three choices. Returns the one that was tapped.
fn chooser(
    ui: &mut egui::Ui,
    offers: &[Offer],
    refusal: Option<&String>,
    dir: &str,
) -> Option<Component> {
    ui.heading("yog");
    ui.weak("nothing is provisioned, so nothing is running.");
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
fn opened(ui: &mut egui::Ui, offer: &Offer, dir: &str, refusal: Option<&String>) -> Act {
    let mut act = Act::Stay;
    if ui.button("< back").clicked() {
        act = Act::Back;
    }
    ui.heading(&offer.brand);
    ui.weak(&offer.tagline);
    ui.separator();
    egui::ScrollArea::vertical().show(ui, |ui| {
        if offer.component == Component::Server {
            ui.label(&offer.how);
            ui.add_space(8.0);
            ui.weak("this bootstrap starts nothing.");
            return;
        }
        if enrollment(ui, offer, dir, refusal) {
            act = Act::Recheck;
        }
    });
    act
}

/// The enrollment screen proper: the file list, the path, the delivery
/// channels, and the one control that re-reads this app's own storage.
/// Returns whether that control was pressed.
fn enrollment(ui: &mut egui::Ui, offer: &Offer, dir: &str, refusal: Option<&String>) -> bool {
    ui.strong("this device needs");
    for file in crate::material::WANTED {
        ui.label(format!("  · {file}"));
    }
    ui.add_space(4.0);
    ui.strong("put them at");
    ui.label(format!("  {dir}"));
    ui.add_space(8.0);
    ui.label(&offer.how);
    ui.add_space(8.0);
    ui.strong("how it gets here");
    for channel in crate::bootstrap::channels() {
        ui.label(format!("  · {channel}"));
    }
    ui.add_space(12.0);
    ui.separator();
    if let Some(why) = refusal {
        ui.colored_label(egui::Color32::LIGHT_RED, why);
    } else {
        ui.weak("nothing has arrived yet.");
    }
    ui.add_space(4.0);
    // A read of this app's own storage, not a dial: it is the act that makes
    // an `adb push` land without killing and relaunching the app.
    ui.button("check for material").clicked()
}
