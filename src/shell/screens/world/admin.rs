//! **The admin screen** (DESIGN §13.17): the config files a world's policy is
//! written in, the task branch this workspace is marked with, its inbox flush,
//! and the unmaking of the workspace itself.
//!
//! **It holds the workspace it was opened on**, which is what this app does
//! with every aimed screen (§13.13) and matters most here: one of these acts
//! deletes that workspace, so a gesture composed against whatever the focus
//! happened to be at the moment of the tap would be a control that moved under
//! the operator.
//!
//! **Two of the three config destinations name no workspace at all**, and they
//! are here anyway: `config` is ONE op with a destination parameter, so
//! splitting its destinations across two screens would be two homes for one
//! op. What the screen holds is the workspace; what a destination names is the
//! destination's own business, and the label says which.
//!
//! **The composer is the editor, and a read is what seeds it.** Tapping a
//! destination reads that file and loads its bytes into the field, so the
//! ordinary flow is read, edit, write. It seeds **once per destination**: a
//! re-read of the file being edited keeps the draft (a tap that discarded work
//! would be a tap nobody dares make), and switching destinations loads the new
//! file, which is also how a bad draft is thrown away.
//!
//! **`delete-workspace` is an ENABLEMENT, not an arming, and the wire is what
//! says so** (lernie DESIGN §4.20). The engine refuses unless the typed name
//! matches the workspace's own, so the control is dark until the field holds
//! that name — the parameter is missing, not the subject, which is why it
//! stays on the glass saying what would fill it rather than vanishing.
//!
//! **It is the last control on the screen**, under everything else: an
//! unmaking gets the ORDER the fleet screen's stopping pair gets, at the one
//! site where the act really is an unmaking.

use eframe::egui;

mod acts;
mod minted;

use crate::codec::Destination;
use crate::seat::Snapshot;
use crate::shell::app::Shell;
use crate::shell::mark::{Back, TOUCH};

/// The screen's name and the harness's tap target (§15.2) — the op an operator
/// comes here for.
pub(in crate::shell) const SCREEN: &str = "config";

impl Shell {
    pub(in crate::shell) fn admin(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        // **The minted material covers this screen while it stands** (§13.18):
        // the whole act is look-at-it-now-and-close-it, and a surface legible
        // behind it would invite the one thing a private key on a display must
        // not have, which is a long life there.
        if let Some(envelope) = snap.minted.clone() {
            self.minted(ui, &envelope);
            return;
        }
        self.note_screen(SCREEN);
        let workspace = snap.focus.workspace.clone().unwrap_or_default();
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            self.admin_acts(ui, &workspace);
            ui.add_space(4.0);
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                if self.bar(ui, SCREEN, &Back::To("conversations")) {
                    self.close_world();
                }
                super::super::banner(ui, snap);
                ui.separator();
                egui::ScrollArea::vertical()
                    .min_scrolled_height(0.0)
                    .show(ui, |ui| self.destinations(ui, snap, &workspace));
            });
        });
    }

    /// The reads: which branch this workspace is marked with, the three
    /// destinations as controls, and the file the last one answered.
    fn destinations(&mut self, ui: &mut egui::Ui, snap: &Snapshot, workspace: &str) {
        match snap.marks.as_ref().filter(|marks| marks.about(workspace)) {
            Some(marks) => ui.weak(format!("marked at {}", marks.branch)),
            None => ui.weak("nothing read yet"),
        };
        ui.separator();
        for at in files(workspace) {
            let picked = self.destination.as_ref() == Some(&at);
            let mark = if picked { "▸ " } else { "" };
            let control = ui.add(
                egui::Button::new(format!("{mark}{}", at.file()))
                    .min_size(egui::vec2(ui.available_width(), TOUCH)),
            );
            crate::shell::act::act(ui, &control, SCREEN);
            if control.clicked() {
                self.destination = Some(at.clone());
                if let Some(model) = self.model() {
                    model.read_config(at);
                }
            }
            ui.add_space(4.0);
        }
        self.seed(snap);
        if let Some(config) = snap.config.as_ref() {
            ui.weak(format!("{} · read", config.at.file()));
        }
    }

    /// **The read seeds the editor, once per destination.** The answer carries
    /// the destination it was read at, so this compares against what was last
    /// loaded rather than remembering a second name for it — and a re-read of
    /// the file being edited keeps the draft.
    fn seed(&mut self, snap: &Snapshot) {
        let Some(config) = snap.config.as_ref() else {
            return;
        };
        let file = config.at.file().to_owned();
        if self.seeded.as_deref() == Some(file.as_str()) {
            return;
        }
        self.composer.clone_from(&config.text);
        self.seeded = Some(file);
    }
}

/// The three destinations this seat spells, in the order a thumb meets them:
/// the workspace's own first, the engine's two under it.
fn files(workspace: &str) -> [Destination; 3] {
    [
        Destination::Brazen {
            workspace: workspace.to_owned(),
        },
        Destination::LitanyModels,
        Destination::Cadence,
    ]
}
