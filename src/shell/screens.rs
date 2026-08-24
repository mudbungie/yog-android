//! The three screens, by focus depth: workspace roster, conversation list,
//! transcript-with-composer. Pure presentation over the model's snapshot —
//! every wire crossing already happened on the model's worker thread, and a
//! tap is a command sent, never a call waited on.

use eframe::egui;

use super::app::{COMPOSER, Shell};
use crate::codec::{Block, Entry, EntryKind};
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
        if ui.button("< conversations").clicked() {
            self.focus_workspace(Some(workspace.to_owned()));
        }
        ui.heading(agent);
        ui.separator();
        // Bottom-up: the composer rides above the keyboard (or the gesture-
        // nav bar), then the transcript takes whatever height remains.
        let inset = self.inset.bottom;
        let ppp = ui.ctx().pixels_per_point();
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
                    for entry in &snap.transcript {
                        ui.label(line(entry));
                    }
                });
        });
    }

    fn focus_workspace(&self, workspace: Option<String>) {
        if let Ok(model) = &self.model {
            model.focus_workspace(workspace);
        }
    }
}

/// One transcript entry as the line the phone paints — the smallest honest
/// rendering; richer surfaces grow per consumer, never speculatively.
fn line(entry: &Entry) -> String {
    match &entry.kind {
        EntryKind::Delivered { sender, body, .. } => format!("{sender}: {body}"),
        EntryKind::Model { blocks, .. } => blocks
            .iter()
            .filter_map(|b| match b {
                Block::Text(text) => Some(text.clone()),
                Block::Thinking(_) | Block::ToolUse { .. } => None,
            })
            .collect::<Vec<_>>()
            .join("\n"),
        EntryKind::ToolResult { content, .. } => format!("[tool] {content}"),
        EntryKind::Streaming { thinking, text } => {
            if text.is_empty() {
                thinking.clone()
            } else {
                text.clone()
            }
        }
        EntryKind::Compacted { summary, .. } => format!("[compacted] {summary}"),
        EntryKind::Raw => entry.raw.clone(),
    }
}
