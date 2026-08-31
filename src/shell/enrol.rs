//! **The first-run surface**: the three bootstraps, painted (yog bl-15bd).
//!
//! It is a *surface* rather than a picker, and that is the ruling: the
//! components are gated, so an app that picked one would have made the
//! operator's choice for them and an app that started all three would have
//! made it three times. What is on screen is what each bootstrap *is* and the
//! act that takes it — every one of those acts happens off this device, which
//! is REMOTE §1.4 rather than a limitation of the app:
//!
//! > *"there is no pairing protocol in the wire, no token exchange a stranger
//! > on the network could initiate. Bootstrap is always an act performed
//! > through existing trust (a cable, an authenticated route, a screen), and
//! > the first thing the new device does with its material is an ordinary mTLS
//! > dial like any other client."*
//!
//! So **there is no button here**, deliberately. Every widget on this screen
//! reads; none of them acts. A tap that "started enrolment" would be the
//! unauthenticated connection §1.4 forbids, dressed as a convenience.
//!
//! The content is [`crate::bootstrap::offers`]' and the emphasis is its
//! `default` flag — the two enrolments carry it and the server does not.

use eframe::egui;

use crate::bootstrap::Offer;

/// Paint the cold device's whole screen.
pub(crate) fn surface(ui: &mut egui::Ui, offers: &[Offer], refusal: Option<&String>, dir: &str) {
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
    egui::ScrollArea::vertical().show(ui, |ui| {
        for offer in offers.iter().filter(|o| o.default) {
            entry(ui, offer, true);
        }
        ui.add_space(12.0);
        ui.weak("and one more, which is not the usual choice:");
        ui.add_space(4.0);
        for offer in offers.iter().filter(|o| !o.default) {
            entry(ui, offer, false);
        }
        ui.add_space(12.0);
        ui.separator();
        // The path, once more and on its own line: it is the fact an operator
        // with a cable in their hand is actually here for.
        ui.weak(format!("material goes at {dir}"));
    });
}

/// One offer: what it makes this device, and the act that takes it.
fn entry(ui: &mut egui::Ui, offer: &Offer, emphasised: bool) {
    ui.add_space(8.0);
    if emphasised {
        ui.strong(&offer.title);
    } else {
        ui.weak(&offer.title);
    }
    ui.label(&offer.how);
}
