//! The screens above the transcript: the bootstrap gate, the foot's standing,
//! the workspace roster and the conversation list. Pure presentation over the
//! model's snapshot — every wire crossing already happened on the model's
//! worker thread, and a tap is a command sent, never a call waited on.
//!
//! The transcript is the third depth and lives in `transcript.rs`: it is the
//! only screen with its own mechanics (the projected rows, the fold overrides,
//! the composer riding the keyboard) rather than a list of taps.

use eframe::egui;

use super::app::{COMPOSER, Shell};
use super::boot::Running;
use crate::seat::Snapshot;

impl Shell {
    /// Everything below the top inset: the component this launch is running,
    /// and then the screen its focus depth selects.
    ///
    /// **The outermost branch is the bootstrap gate** (yog bl-15bd), not a
    /// state check: a cold device paints the three offers and a foot paints
    /// what it is hosting, because a foot may not ask the world anything
    /// (REMOTE §4.2) and painting it a chat screen would be an app promising
    /// a surface its own certificate refuses.
    pub(crate) fn screens(&mut self, ui: &mut egui::Ui) {
        match &self.running {
            Running::Cold {
                offers,
                refusal,
                dir,
            } => {
                super::enrol::surface(ui, &offers.clone(), refusal.clone().as_ref(), &dir.clone());
                return;
            }
            Running::Foot { .. } => {
                self.foot(ui);
                return;
            }
            Running::Seat { .. } => {}
        }
        let Some(snap) = self.model_mut().map(crate::seat::Model::snapshot) else {
            return;
        };
        if let Some(error) = &snap.error {
            ui.colored_label(egui::Color32::LIGHT_RED, error);
        }
        match snap.focus.workspace.clone() {
            None => self.roster(ui, &snap),
            Some(workspace) => match snap.focus.agent.clone() {
                None => self.conversations(ui, &snap, &workspace),
                Some(agent) => self.transcript(ui, &snap, &workspace, &agent),
            },
        }
    }

    /// The foot's whole screen: what this machine offers and what it has run.
    /// There is nothing else to paint, and that is the component working —
    /// §4.2's *"a foot cannot ask about the world"* is the sentence, and an
    /// empty roster would be this app asking anyway and hiding the refusal.
    fn foot(&mut self, ui: &mut egui::Ui) {
        ui.heading("tool host");
        ui.weak(self.identity());
        ui.separator();
        self.hosting(ui);
        ui.add_space(8.0);
        ui.weak(
            "A foot advertises what this machine can run, waits for work \
             addressed to it, and hands back what happened. It says nothing \
             else about the world — mint this device an operator-grade leaf \
             to seat it instead.",
        );
    }

    fn roster(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        ui.heading("workspaces");
        ui.weak(self.identity());
        self.hosting(ui);
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for row in &snap.workspaces {
                let mark = if row.attention > 0 { " ●" } else { "" };
                let label = format!("{}{mark} · {} agents", row.workspace, row.agents);
                if ui.button(label).clicked() {
                    self.focus_workspace(Some(row.workspace.clone()));
                }
            }
        });
    }

    pub(super) fn conversations(&mut self, ui: &mut egui::Ui, snap: &Snapshot, workspace: &str) {
        if ui.button("< workspaces").clicked() {
            self.focus_workspace(None);
        }
        ui.heading(workspace);
        ui.separator();
        // The starter rides the BOTTOM of this screen, where the composer
        // sits on the next one: starting a conversation and speaking into one
        // are the same gesture to a thumb, so they are in the same place.
        let inset = self.inset.bottom;
        let ppp = ui.ctx().pixels_per_point();
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.add_space((inset as f32 / ppp).max(8.0));
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
                        if ui.button(label).clicked()
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
        let r = ui.add(
            egui::TextEdit::singleline(&mut self.composer)
                .id(egui::Id::new(COMPOSER.id))
                .desired_width(f32::INFINITY)
                .hint_text("start a conversation"),
        );
        if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            let goal = std::mem::take(&mut self.composer);
            if !goal.is_empty()
                && let Some(model) = self.model()
            {
                model.start_conversation(goal);
            }
            r.request_focus();
        }
    }

    /// What this device offers a session, one line (REMOTE §5). It rides the
    /// roster because that is the screen an operator lands on, and a tool host
    /// nobody can see is one nobody can tell has stopped.
    fn hosting(&mut self, ui: &mut egui::Ui) {
        let Some(host) = self.host_mut() else { return };
        let standing = host.standing();
        let line = match (&standing.stopped, &standing.last) {
            (Some(why), _) => format!("tools stopped: {why}"),
            (None, Some(last)) => format!(
                "tools: {} · served {} · {last}",
                standing.tools.join(", "),
                standing.served
            ),
            (None, None) if standing.advertised => {
                format!("tools: {} · waiting", standing.tools.join(", "))
            }
            (None, None) => "tools: presenting…".to_owned(),
        };
        if standing.stopped.is_some() {
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
