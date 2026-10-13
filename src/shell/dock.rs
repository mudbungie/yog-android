//! **The live dock** (DESIGN §22 item 9, bl-365c): one tap line above the
//! rule while anything under the focused workspace runs, unfolding into one
//! touch-tall row per running conversation. Which rows and what each says are
//! `crate::dock`, pure and host-tested; this file paints them and performs
//! the taps.
//!
//! Added to the transcript's bottom-up floor, so it is painted from the rule
//! upward: the rows first, the line last — which puts the line on top and the
//! rows between it and the rule, reading downward as the desktop's do.

use eframe::egui;

use super::app::Shell;
use super::theme::{self, ink, rgb};
use crate::seat::Snapshot;
use crate::theme::State;

/// Where the fold is kept: egui's temporary memory, viewport state that dies
/// with the process (§22 item 9).
fn fold() -> egui::Id {
    egui::Id::new("yog.dock.unfolded")
}

impl Shell {
    /// **The dock**, added to a bottom-up block; nothing while nothing runs.
    pub(super) fn dock(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        let rows = crate::dock::running(&snap.conversations);
        let open = ui
            .ctx()
            .data(|d| d.get_temp::<bool>(fold()))
            .unwrap_or(false);
        let Some(words) = crate::dock::line(&rows, open) else {
            return;
        };
        let mut chosen = None;
        if open {
            let now = crate::roster::now_unix();
            for row in rows.iter().rev() {
                let parts: Vec<(String, egui::Color32)> = crate::dock::row_words(row, now)
                    .into_iter()
                    .map(|(text, hue)| (text, rgb(hue)))
                    .collect();
                if theme::row(ui, theme::line(ui, &parts).into()).clicked() {
                    chosen = Some(row.root_id.clone());
                }
            }
        }
        let head = theme::row(
            ui,
            egui::RichText::new(words).color(ink(State::Working)).into(),
        );
        self.note_control("dock", ui, head.rect);
        if head.clicked() {
            ui.ctx().data_mut(|d| d.insert_temp(fold(), !open));
        }
        let (Some(agent), Some(workspace)) = (chosen, snap.focus.workspace.clone()) else {
            return;
        };
        let Some(model) = self.model() else { return };
        model.focus_conversation(workspace, agent);
    }
}
