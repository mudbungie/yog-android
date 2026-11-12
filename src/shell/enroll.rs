//! **The first-run surface**: the three bootstraps as branded, tappable
//! choices, and the screen behind each tap (bl-0d3c, amending bl-7714).
//!
//! **The two enrollment screens are the registration door** (DESIGN §5; yog
//! REMOTE §15.1): a field for the one-time registration URL an operator's
//! offer showed, the camera beside it, and — once the offer is claimed — the
//! confirmation word and the wait for approval. Which grade this device takes
//! is the offer's, so Lernie's screen and Thrall's differ in what they say and
//! not in what they do.
//!
//! **Nothing on this screen is stored.** [`Shell::chose`] is which screen is
//! open right now, no more durable than a scroll position; the component this
//! device runs is still derived from the leaf on disk every boot
//! ([`crate::bootstrap::standing`]). An approval, and *check for material*,
//! re-run that derivation — a read of this app's own storage.

use eframe::egui;

use super::app::Shell;
use super::boot::Running;
use super::mark::Back;
use crate::bootstrap::{Component, Offer};

mod door;
mod scan;

pub(crate) use door::Door;
pub(crate) use scan::Scanner;

/// **What this device stands at, for the screen that would change it**: where
/// material goes, the sentence a half-provisioned store earns, and the
/// identity a registration would replace. One parameter rather than three, because
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
    /// Re-run the boot derivation: the server stopped refusing a device
    /// that registered here, or the operator asked for a look at what a cable
    /// carried. Neither knows what it will find — both are the same question,
    /// asked after the material changed.
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
            self.note_screen("configuration");
            // The bar's back is the way out, where a way out exists
            // (bl-e192): a cold device has nothing behind this surface to
            // return to, so it gets no back control at all.
            let out = if matches!(self.running, Running::Cold { .. }) {
                Back::None
            } else {
                Back::Plain
            };
            if self.bar(ui, "configuration", &out) {
                self.forget_door();
                self.settings = false;
                return;
            }
            let word = self.word.clone();
            let (chosen, at) = chooser(ui, &offers, refusal.as_ref(), &dir, &standing, word);
            // Where each way in is, for the harness that has to tap one
            // (`app/probe.rs`): named by the brand an operator reads.
            for (component, rect) in at {
                let name = match component {
                    Component::Seat => "lernie",
                    Component::Foot => "thrall",
                    Component::Server => "yog",
                };
                self.note_control(name, ui, rect);
            }
            self.chose = chosen;
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
        // The three screens behind the chooser, named for the harness that
        // walks them (`app/probe.rs`): the two enrollment screens differ only
        // in the leaf they are asking for, and a walk that cannot tell them
        // apart cannot say which one it reached.
        self.note_screen(match open {
            Component::Seat => "enroll-seat",
            Component::Foot => "enroll-foot",
            Component::Server => "server",
        });
        if self.scanner.live() {
            self.note_screen("scan");
            if std::mem::take(&mut self.back) {
                self.scanner.shut();
            }
        } else if self.bar(ui, &offer.brand.clone(), &Back::Plain) {
            self.forget_door();
            return;
        }
        // §13.1 W3, said to the one operator it is for: opening Thrall while
        // a seat runs is the moment before enrolling a second name this
        // device does not need — the seat already hosts tools beside the
        // asker, one identity on two connections (REMOTE §5).
        if open == Component::Foot && matches!(self.running, Running::Seat { .. }) {
            ui.colored_label(
                super::theme::ink(crate::theme::State::Annotation),
                format!(
                    "this device already offers its tools — it runs {}, and a \
                     Lernie seat hosts tools beside the asker. Thrall is for a \
                     device that should offer ONLY tools.",
                    self.identity()
                ),
            );
        }
        // What a registration here would REPLACE, when there is something to
        // lose (bl-f12d). `None` on a cold device: nothing is running, so
        // there is no consequence to state — and the identity line is the one
        // that already knows, so this derives from it rather than reading the
        // leaf a second time.
        let landing = Landing {
            dir,
            refusal,
            replacing: match &self.running {
                Running::Cold { .. } => None,
                _ => Some(self.identity()),
            },
        };
        match opened(ui, offer, &landing, &mut self.door, &mut self.scanner) {
            Act::Stay => {}
            Act::Recheck => {
                self.door = Door::new();
                self.scanner.shut();
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
    word: Option<String>,
) -> (Option<Component>, Vec<(Component, egui::Rect)>) {
    ui.weak(standing);
    // The word of a device whose key this app made (DESIGN §5): read off the
    // files, so it is here for the operator who is asked for it after the
    // screen that first said it has gone.
    if let Some(word) = word {
        door::word(ui, &word);
        ui.weak("an operator approves this device by the offer whose row shows this word.");
    }
    // A half-provisioned store is not an empty one, and the two must not read
    // the same: this device HAS something, and the sentence names every file
    // that is missing at once.
    if let Some(why) = refusal {
        ui.add_space(4.0);
        ui.colored_label(super::theme::ink(crate::theme::State::Error), why);
    }
    ui.separator();
    ui.add_space(4.0);
    ui.weak("choose what this device becomes:");
    let mut chosen = None;
    let mut at = Vec::new();
    egui::ScrollArea::vertical().show(ui, |ui| {
        for offer in offers.iter().filter(|o| o.default) {
            let hit = choice(ui, offer, true);
            at.push((offer.component, hit.rect));
            if hit.clicked() {
                chosen = Some(offer.component);
            }
        }
        ui.add_space(12.0);
        ui.weak("and one more, which is not the usual choice:");
        ui.add_space(4.0);
        for offer in offers.iter().filter(|o| !o.default) {
            let hit = choice(ui, offer, false);
            at.push((offer.component, hit.rect));
            if hit.clicked() {
                chosen = Some(offer.component);
            }
        }
        ui.add_space(12.0);
        ui.separator();
        // The path, once more and on its own line: it is the fact an operator
        // with a cable in their hand is actually here for.
        ui.weak(format!("material goes at {dir}"));
    });
    (chosen, at)
}

/// One branded choice: the name on the control, the line under it. Tapping it
/// opens a screen — it does not store the choice, and it does not dial.
fn choice(ui: &mut egui::Ui, offer: &Offer, emphasised: bool) -> egui::Response {
    ui.add_space(8.0);
    let size = if emphasised { 24.0 } else { 19.0 };
    let brand = egui::RichText::new(&offer.brand).size(size).strong();
    let hit = super::theme::button(
        ui,
        brand,
        ui.available_width(),
        "opens this way in; nothing is stored or dialled yet",
    );
    ui.weak(&offer.tagline);
    hit
}

/// The screen a choice opened. For the two enrollments it is the
/// registration door; for the server it is the recorded blockers.
fn opened(
    ui: &mut egui::Ui,
    offer: &Offer,
    landing: &Landing,
    door: &mut Door,
    scanner: &mut Scanner,
) -> Act {
    // The scan screen is the whole screen while it is up: a camera preview
    // under a heading and a back control belonging to a screen it is covering
    // reads as two screens at once.
    if scanner.live() {
        door::scanned(ui, &landing.dir, door, scanner);
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
        crate::shell::clip::said(ui, &offer.how);
        ui.add_space(8.0);
        if door::screen(ui, landing, door, scanner) {
            act = Act::Recheck;
        }
    });
    act
}
