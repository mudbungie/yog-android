//! **The yog mark**: the app's own name, a control at the top-left of every
//! screen, and the one way into the configuration surface (bl-387f).
//!
//! The operator's sighting: a device enrolled as a seat had no path back to
//! the first-run surface, so the second act — enrolling the tooling side —
//! could not be reached at the glass. Breadcrumbs were considered and
//! rejected: a breadcrumb trail requires every path worked out, and the
//! paths are not. One standing control that is always there asks nothing of
//! the screens beneath it.
//!
//! The mark TOGGLES. Into the configuration when the app is showing a
//! component, back out when the configuration is open. On a cold device the
//! configuration is forced open anyway (`screens.rs`), so the way back out
//! lands on the chooser — the mark doubles as "home" there, which costs
//! nothing and needs no second rule.

use eframe::egui;

use super::app::Shell;

impl Shell {
    /// The mark row. Painted before anything else on every screen, whatever
    /// the component — that unconditionality is the whole feature.
    pub(super) fn mark(&mut self, ui: &mut egui::Ui) {
        let word = egui::RichText::new("yog").size(19.0).strong();
        if !ui.button(word).clicked() {
            return;
        }
        if self.settings {
            // The way out drops whatever the configuration held: a pasted
            // envelope is a private key and an open camera is a running
            // capture session, and this tap is the last thing that knows the
            // screen is going away.
            self.forget_envelope();
            self.settings = false;
        } else {
            self.settings = true;
        }
    }
}
