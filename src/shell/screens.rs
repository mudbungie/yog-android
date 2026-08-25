//! The three screens, by focus depth: workspace roster, conversation list,
//! transcript-with-composer. Pure presentation over the model's snapshot —
//! every wire crossing already happened on the model's worker thread, and a
//! tap is a command sent, never a call waited on.

use eframe::egui;

use super::app::{COMPOSER, Shell};
use crate::rows::rows;
use crate::seat::Snapshot;

impl Shell {
    /// Everything below the top inset: the banner, then the screen the
    /// focus depth selects.
    pub(crate) fn screens(&mut self, ui: &mut egui::Ui) {
        let snap = match &mut self.model {
            Ok(model) => model.snapshot(),
            Err(why) => {
                // Unprovisioned or unopenable material: one sentence, and
                // provisioning is an operator act followed by a relaunch
                // (DESIGN §5) — nothing to retry from here.
                ui.label(why.clone());
                return;
            }
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

    fn roster(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        ui.heading("workspaces");
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

    fn conversations(&mut self, ui: &mut egui::Ui, snap: &Snapshot, workspace: &str) {
        if ui.button("< workspaces").clicked() {
            self.focus_workspace(None);
        }
        ui.heading(workspace);
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for row in &snap.conversations {
                let mark = if row.attention > 0 { " ●" } else { "" };
                let label = format!("{}{mark}\n{}", row.display, row.preview);
                if ui.button(label).clicked()
                    && let Ok(model) = &self.model
                {
                    model.focus_conversation(workspace.to_owned(), row.root_id.clone());
                }
            }
        });
    }

    fn transcript(&mut self, ui: &mut egui::Ui, snap: &Snapshot, workspace: &str, agent: &str) {
        ui.horizontal(|ui| {
            if ui.button("< conversations").clicked() {
                self.focus_workspace(Some(workspace.to_owned()));
            }
            // The two auto knobs, the desktop's own pair: which KINDS open by
            // default. They are policy; a hand-flipped row is the override
            // set below and dies with the screen.
            ui.checkbox(&mut self.auto.responses, "talk");
            ui.checkbox(&mut self.auto.others, "steps");
        });
        ui.heading(super::chat::speaker_of(snap, agent));
        ui.separator();
        let speaker = super::chat::speaker_of(snap, agent);
        let painted = rows(&snap.transcript, &speaker, self.auto, &self.folds);
        // Bottom-up: the composer rides above the keyboard (or the gesture-
        // nav bar), then the transcript takes whatever height remains.
        let inset = self.inset.bottom;
        let ppp = ui.ctx().pixels_per_point();
        let mut flipped = None;
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.add_space((inset as f32 / ppp).max(8.0));
            let r = ui.add(
                egui::TextEdit::singleline(&mut self.composer)
                    .id(egui::Id::new(COMPOSER.id))
                    .desired_width(f32::INFINITY)
                    .hint_text("message"),
            );
            if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                let taken = std::mem::take(&mut self.composer);
                if !taken.is_empty()
                    && let Ok(model) = &self.model
                {
                    model.deposit(taken);
                }
                r.request_focus();
            }
            ui.add_space(4.0);
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    // Top-down inside the scroller: the rows are in message
                    // order and the bottom-up layout above is about where the
                    // composer sits, not about which way a transcript reads.
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        for row in &painted {
                            if super::chat::row(ui, row) {
                                flipped = Some(row.key.clone());
                            }
                        }
                    });
                });
        });
        // Applied after the walk: flipping mid-iteration would re-project the
        // rows the loop is still reading.
        if let Some(key) = flipped
            && !self.folds.remove(&key)
        {
            self.folds.insert(key);
        }
    }

    /// What this device offers a session, one line (REMOTE §5). It rides the
    /// roster because that is the screen an operator lands on, and a tool host
    /// nobody can see is one nobody can tell has stopped.
    fn hosting(&mut self, ui: &mut egui::Ui) {
        let Some(host) = &mut self.host else { return };
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

    fn focus_workspace(&self, workspace: Option<String>) {
        if let Ok(model) = &self.model {
            model.focus_workspace(workspace);
        }
    }
}
