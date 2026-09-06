//! **The sign-in screen** (REMOTE §8.3, DESIGN §13.19): the workspace's
//! provider rows, the act that signs one in, and the held tail of whatever
//! that run is saying.
//!
//! **It is a depth, not a control on the composer's row.** The provider
//! selector under the composer states each row's credential fact and stays
//! tappable *because* an operator may be about to sign it in (§13.2) — and
//! what a sign-in produces is a URL to copy, a device code to type and, when
//! it fails, a command to run by hand. None of that fits in a popup over a
//! composer, and a band under it would push the transcript off the glass for
//! the minutes a browser flow takes. So it is an aimed entry beside the admin
//! screen: one workspace, its providers, and room to read.
//!
//! **The rows are the same fact painted twice, and that is not two homes.**
//! Both surfaces read `snap.providers` — the engine's own rows with the
//! engine's own sentences on them — and neither derives anything. Opening
//! this screen IS the `providers` ask, exactly as opening the records screen
//! is its five.
//!
//! **Tapping a row opens its tail; the button beside it starts a run.** The
//! two are different gestures and the wire says so: `login-tail` is a held
//! READ and `login` is the act. A row that cannot be signed in shows the
//! engine's reason where the verb would be, and its button is dark — the
//! greying discipline the selector already keeps (§13.2), which never derives
//! the reason and never hides the row.
//!
//! **There is one tail, under the row it belongs to.** The worker holds which
//! provider is watched (`seat::pass::login`) and the snapshot carries it back
//! with the lines, so which row is open has ONE home and this screen paints
//! rather than remembers it — no local pick to drift from the lane's subject.

use eframe::egui;

use crate::codec::{LoginView, ProviderRow};
use crate::seat::Snapshot;
use crate::shell::app::Shell;
use crate::shell::mark::{Back, TOUCH};

/// The screen's name, the harness's tap target (§15.2), and the act an
/// operator comes here for.
pub(in crate::shell) const SCREEN: &str = "login";

impl Shell {
    pub(in crate::shell) fn signin(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        self.note_screen(SCREEN);
        if self.bar(ui, SCREEN, &Back::To("conversations")) {
            self.close_world();
        }
        super::super::banner(ui, snap);
        ui.separator();
        egui::ScrollArea::vertical()
            .min_scrolled_height(0.0)
            .show(ui, |ui| self.rows(ui, snap));
    }

    /// The provider rows, each with its own tail under it when it is the one
    /// being followed.
    fn rows(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        if snap.providers.is_empty() {
            ui.weak("nothing read yet");
        }
        for row in &snap.providers {
            self.provider(ui, row);
            if let Some(signing) = snap.login.as_ref().filter(|held| held.about(&row.name)) {
                tail(ui, &signing.view);
            }
            ui.add_space(4.0);
        }
    }

    /// One row: what the engine says about it, the tap that follows its run,
    /// and the act that starts one.
    fn provider(&mut self, ui: &mut egui::Ui, row: &ProviderRow) {
        let name = row.name.clone();
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), TOUCH),
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                let start = ui.add_enabled(row.blocked.is_none(), egui::Button::new("sign in"));
                crate::shell::act::act(ui, &start, SCREEN);
                if start.clicked()
                    && let Some(model) = self.model()
                {
                    model.sign_in(name.clone());
                }
                let stated = match &row.blocked {
                    Some(why) => format!("{} · {} · {why}", row.name, row.fact),
                    None => format!("{} · {}", row.name, row.fact),
                };
                let follow = ui.add(
                    egui::Button::new(stated).min_size(egui::vec2(ui.available_width(), TOUCH)),
                );
                crate::shell::act::act(ui, &follow, "login-tail");
                if follow.clicked()
                    && let Some(model) = self.model()
                {
                    model.watch_login(Some(name.clone()));
                }
            },
        );
    }
}

/// **What one run has said.** The `err` flag is which stream a line came down
/// and never a verdict on it — bz writes the whole human-facing flow, the
/// authorize URL included, to stderr — so both are painted the same and
/// neither is coloured. The outcome and the fallback are painted only once
/// they exist: a run still going has no exit, and saying it exited zero would
/// be the one misreading this screen must not make.
fn tail(ui: &mut egui::Ui, view: &LoginView) {
    if view.lines.is_empty() && view.outcome.is_none() {
        ui.weak("nothing said yet");
    }
    for line in &view.lines {
        ui.weak(&line.text);
    }
    if let Some(outcome) = view.outcome {
        let said = if outcome == 0 {
            "signed in".to_owned()
        } else {
            format!("the sign-in ended at {outcome}")
        };
        ui.label(said);
    }
    if let Some(fallback) = &view.fallback {
        ui.weak(format!("run this by hand: {fallback}"));
    }
}
