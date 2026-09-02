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
use crate::host::Health;
use crate::seat::Snapshot;

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
            self.bar(ui, &crate::bootstrap::Component::Foot.brand(), false);
            self.foot(ui);
            return;
        }
        let Some(snap) = self.model_mut().map(crate::seat::Model::snapshot) else {
            return;
        };
        // The bar first, then the error banner under it (§13.2), then the
        // depth's own body. Back walks exactly one focus depth — the bar
        // returns the tap and this match is the one place a depth is spelled.
        match snap.focus.workspace.clone() {
            None => {
                self.bar(ui, &crate::bootstrap::Component::Seat.brand(), false);
                banner(ui, &snap);
                self.roster(ui, &snap);
            }
            Some(workspace) => match snap.focus.agent.clone() {
                None => {
                    if self.bar(ui, &workspace, true) {
                        self.focus_workspace(None);
                    }
                    banner(ui, &snap);
                    self.conversations(ui, &snap, &workspace);
                }
                Some(agent) => {
                    if self.bar(ui, &super::chat::speaker_of(&snap, &agent), true) {
                        self.focus_workspace(Some(workspace.clone()));
                    }
                    banner(ui, &snap);
                    self.transcript(ui, &snap, &agent);
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
        self.hosting(ui);
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
        self.hosting(ui);
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for row in &snap.workspaces {
                let mark = if row.attention > 0 { " ●" } else { "" };
                let label = format!("{}{mark} · {} agents", row.workspace, row.agents);
                if tap(ui, label.into()) {
                    self.focus_workspace(Some(row.workspace.clone()));
                }
            }
        });
    }

    pub(super) fn conversations(&mut self, ui: &mut egui::Ui, snap: &Snapshot, workspace: &str) {
        ui.separator();
        // The starter rides the BOTTOM of this screen, where the composer
        // sits on the next one: starting a conversation and speaking into one
        // are the same gesture to a thumb, so they are in the same place. The
        // bottom of this layout is already the platform's floor — `app::pass`
        // spends the inset once, for every screen (bl-9cfd).
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            // The same controls row as the transcript's, under the same
            // composer (§13.2, bl-0267): a model is picked for the WORKSPACE,
            // so it is picked from the screen that lists it as readily as
            // from a conversation inside it.
            self.controls(ui, snap);
            self.starter(ui);
            ui.add_space(4.0);
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    if snap.conversations.is_empty() {
                        ui.weak("nothing here yet — say what to start below");
                    }
                    for row in &snap.conversations {
                        let mark = if row.attention > 0 { " ●" } else { "" };
                        let label = format!("{}{mark}\n{}", row.display, row.preview);
                        // The row's ink is the ENGINE's reading of it, not a
                        // second one taken here (bl-ef9a). `Tone::Bad` is the
                        // one passive sighting of a conversation refused at
                        // the provider rung: the badge set is frozen at four,
                        // so such a conversation comes to rest `stopped` — the
                        // word `/stop` owns — and a list where the two read
                        // identically is a list that cannot be scanned.
                        // `Tone::Weak` is a start whose driver has written no
                        // branch yet. `chat::tone_hue` is the same map the
                        // transcript spends, so this app has one colour
                        // vocabulary and it is the desktop's.
                        let ink = super::chat::tone_hue(ui, row.tone);
                        let label = egui::RichText::new(label).color(ink);
                        if tap(ui, label)
                            && let Some(model) = self.model()
                        {
                            model.focus_conversation(workspace.to_owned(), row.root_id.clone());
                        }
                    }
                });
            });
        });
    }

    /// The one field that starts a conversation. It shares the composer's
    /// widget id with the chat screen's, and deliberately: only one of the
    /// two is ever on screen, they are the same gesture at two depths, and
    /// the IME bridge addresses exactly one field by that id (bl-014e).
    fn starter(&mut self, ui: &mut egui::Ui) {
        if let Some(goal) = super::chat::composer(ui, &mut self.composer, "start a conversation")
            && let Some(model) = self.model()
        {
            model.start_conversation(goal);
        }
    }

    /// What this device offers a session, one line (REMOTE §5). It rides the
    /// roster because that is the screen an operator lands on, and a tool host
    /// nobody can see is one nobody can tell has stopped.
    fn hosting(&mut self, ui: &mut egui::Ui) {
        let Some(host) = self.host_mut() else { return };
        let standing = host.standing();
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
    }

    pub(super) fn focus_workspace(&self, workspace: Option<String>) {
        if let Some(model) = self.model() {
            model.focus_workspace(workspace);
        }
    }
}

/// The connection banner: the worker's standing error, under the bar and
/// above whatever screen it interrupted (§13.2).
fn banner(ui: &mut egui::Ui, snap: &Snapshot) {
    if let Some(error) = &snap.error {
        ui.colored_label(egui::Color32::LIGHT_RED, error);
    }
}

/// One full-width list row at the §13.2 touch floor. Every navigation list
/// paints its rows through this, so the floor is a fact of the helper rather
/// than a discipline at each site.
fn tap(ui: &mut egui::Ui, label: egui::RichText) -> bool {
    let control =
        egui::Button::new(label).min_size(egui::vec2(ui.available_width(), super::mark::TOUCH));
    ui.add(control).clicked()
}
