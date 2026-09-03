//! The screens above the transcript: the bootstrap gate, the foot's standing,
//! the workspace roster and the conversation list. Pure presentation over the
//! model's snapshot — every wire crossing already happened on the model's
//! worker thread, and a tap is a command sent, never a call waited on.
//!
//! The transcript is the third depth and lives in `transcript.rs`: it is the
//! only screen with its own mechanics (the projected rows, the fold overrides,
//! the composer riding the keyboard) rather than a list of taps.

use eframe::egui;

use super::app::Shell;
use super::boot::Running;
use super::mark::Back;
use crate::host::Health;
use crate::seat::Snapshot;

mod rows;

impl Shell {
    /// Everything below the top inset: the yog mark, then the component this
    /// launch is running, and then the screen its focus depth selects.
    ///
    /// **The outermost branch is the configuration surface** (bl-387f), and
    /// the bootstrap gate (yog bl-15bd) is its forced-open case: a cold
    /// device paints the three offers because nothing else exists, and a
    /// provisioned one paints them because the mark was tapped — one
    /// surface, two ways in. A foot otherwise paints what it is hosting,
    /// because a foot may not ask the world anything (REMOTE §4.2) and
    /// painting it a chat screen would be an app promising a surface its own
    /// certificate refuses.
    pub(crate) fn screens(&mut self, ui: &mut egui::Ui) {
        if self.settings || matches!(self.running, Running::Cold { .. }) {
            self.configuration(ui);
            return;
        }
        if matches!(self.running, Running::Foot { .. }) {
            self.note_screen("foot");
            self.bar(ui, &crate::bootstrap::Component::Foot.brand(), &Back::None);
            self.foot(ui);
            return;
        }
        let Some(snap) = self.model_mut().map(crate::seat::Model::snapshot) else {
            return;
        };
        // The outbox settles once per frame, whatever screen is up: an echo
        // whose conversation the operator left is not an echo any more
        // (bl-66fb).
        self.settle_echo(&snap);
        // The bar first, then the error banner under it (§13.2), then the
        // depth's own body. Back walks exactly one focus depth — the bar
        // returns the tap and this match is the one place a depth is spelled.
        match snap.focus.workspace.clone() {
            None => {
                self.note_screen("roster");
                self.bar(ui, &crate::bootstrap::Component::Seat.brand(), &Back::None);
                banner(ui, &snap);
                self.roster(ui, &snap);
            }
            // **The two screens that anchor controls to the floor paint their
            // own bar** (bl-192c). Everywhere else the bar goes first because
            // nothing below it can be pushed anywhere; here the floor's order
            // is the opposite — the acts and the composer claim from the
            // floor and the chrome takes what is left above them — so the bar
            // is painted inside that remainder rather than before it. It is
            // still the first thing in the screen's body, which is what §13.2
            // says.
            Some(workspace) => match snap.focus.agent.clone() {
                None => {
                    self.note_screen("conversations");
                    self.conversations(ui, &snap, &workspace);
                }
                Some(agent) => {
                    self.note_screen("transcript");
                    self.transcript(ui, &snap, &workspace, &agent);
                }
            },
        }
    }

    /// The foot's whole screen: what this machine offers and what it has run.
    /// There is nothing else to paint, and that is the component working —
    /// §4.2's *"a foot cannot ask about the world"* is the sentence, and an
    /// empty roster would be this app asking anyway and hiding the refusal.
    fn foot(&mut self, ui: &mut egui::Ui) {
        ui.weak(self.identity());
        ui.separator();
        Self::hosting(ui);
        ui.add_space(8.0);
        ui.weak(
            "Thrall advertises what this machine can run, waits for work \
             addressed to it, and hands back what happened. It says nothing \
             else about the world — mint this device a Lernie (operator-grade) \
             leaf to seat it instead.",
        );
    }

    fn roster(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        ui.weak("workspaces");
        ui.weak(self.identity());
        Self::hosting(ui);
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for row in &snap.workspaces {
                let mark = if row.attention > 0 { " ●" } else { "" };
                let label = format!("{}{mark} · {} agents", row.workspace, row.agents);
                // Tapping a workspace focuses it, and the focus is what the
                // worker asks `conversations` at.
                if tap(ui, label.into(), "conversations").clicked() {
                    self.focus_workspace(Some(row.workspace.clone()));
                }
            }
        });
    }

    /// What this device offers a session, one line (REMOTE §5). It rides the
    /// roster because that is the screen an operator lands on, and a tool host
    /// nobody can see is one nobody can tell has stopped.
    ///
    /// It takes no `self`: since bl-8bd0 the host belongs to the PROCESS and
    /// this reads `state::standing()`, so a receiver here would be a claim
    /// that the frame owns the fact.
    fn hosting(ui: &mut egui::Ui) {
        let Some(standing) = crate::state::standing() else {
            return;
        };
        // Health first: a host that is climbing back says so with the
        // sentence that broke the channel, rather than showing the last tool
        // it ran as though it were still there (bl-8641).
        let line = match (&standing.health, &standing.last) {
            (Health::Stopped(why), _) => format!("tools stopped: {why}"),
            (Health::Redialling(why), _) => format!("tools: reconnecting… · {why}"),
            (Health::Serving, Some(last)) => format!(
                "tools: {} · served {} · {last}",
                standing.tools.join(", "),
                standing.served
            ),
            (Health::Serving, None) if standing.advertised => {
                format!("tools: {} · waiting", standing.tools.join(", "))
            }
            (Health::Serving, None) => "tools: presenting…".to_owned(),
        };
        // Red is for the one that will not mend itself. A redial is ordinary
        // on a phone and reads as ordinary; the word carries it.
        if matches!(standing.health, Health::Stopped(_)) {
            ui.colored_label(egui::Color32::LIGHT_RED, line);
        } else {
            ui.weak(line);
        }
        // **A disarming that healed itself is still worth a sentence**
        // (REMOTE §5.1, bl-cc54): the set this device offers was replaced
        // while it was running a tool, and the host put it back. Yellow and
        // not red, by the rule above — it HAS mended itself — but not weak
        // either, because two processes claiming one device's name is
        // something only an operator can end. The words are
        // `host::RESTORED`'s; this line only says how many times.
        if standing.restored > 0 {
            ui.colored_label(
                egui::Color32::LIGHT_YELLOW,
                format!("{} (×{})", crate::host::RESTORED, standing.restored),
            );
        }
    }

    pub(super) fn focus_workspace(&self, workspace: Option<String>) {
        if let Some(model) = self.model() {
            model.focus_workspace(workspace);
        }
    }
}

/// The connection banner: the worker's standing error, under the bar and
/// above whatever screen it interrupted (§13.2).
pub(super) fn banner(ui: &mut egui::Ui, snap: &Snapshot) {
    if let Some(error) = &snap.error {
        ui.colored_label(egui::Color32::LIGHT_RED, error);
    }
}

/// One full-width list row at the §13.2 touch floor. Every navigation list
/// paints its rows through this, so the floor is a fact of the helper rather
/// than a discipline at each site — and so is the parity tag: `op` is the read
/// this row's tap reaches (PARITY §2, *"the owed interactable for a read is
/// the affordance that reaches the view it populates"*), which is the one
/// thing that differs between the two lists.
pub(super) fn tap(ui: &mut egui::Ui, label: egui::RichText, op: &str) -> egui::Response {
    let control =
        egui::Button::new(label).min_size(egui::vec2(ui.available_width(), super::mark::TOUCH));
    let response = ui.add(control);
    super::act::act(ui, &response, op);
    response
}
