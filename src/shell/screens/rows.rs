//! **The chat in start mode** (DESIGN §22 items 3 and 7, bl-3edc): a
//! workspace focused and no conversation — the prompt line in start mode,
//! and nothing above it but the bar. The list this file used to paint stands
//! in the switcher now (`shell/switcher.rs`), which keeps this module's two
//! halves of a conversation row: the long press it carries (`rows/menu.rs`,
//! §13.5) and the threading it is painted with (`rows/thread.rs`, §13.20).

use eframe::egui;

use crate::overlay::Overlay;
use crate::seat::Snapshot;
use crate::shell::app::Shell;

mod menu;
mod thread;

pub(in crate::shell) use thread::threaded;

impl Shell {
    pub(crate) fn start(&mut self, ui: &mut egui::Ui, snap: &Snapshot, workspace: &str) {
        // The starter rides the BOTTOM of this screen, where the composer sits
        // on a conversation: starting one and speaking into one are the same
        // gesture to a thumb. **What claims the floor first is what may never
        // be pushed off it** (bl-192c).
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            // The prompt line in start mode, and the rule above it (§22
            // items 1 and 2) — no bar: the rule carries the mark.
            self.starter(ui, snap);
            self.rule(ui, snap);
            ui.add_space(4.0);
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                // The verb sheet, while the draft is a slash line (§22 item
                // 5) — the start shares the prompt line and its door.
                if self.sheet(ui, snap) {
                    return;
                }
                ui.weak(format!(
                    "begin a conversation on {workspace} below — back for the others"
                ));
            });
        });
        self.back_to_switcher();
    }

    /// **Back from the chat opens the switcher** (§22 item 7), once nothing
    /// painted on it — the sheet, an open list — took the press first.
    pub(in crate::shell) fn back_to_switcher(&mut self) {
        if std::mem::take(&mut self.back) {
            self.open(Overlay::Switcher);
        }
    }

    /// The one field that starts a conversation. It shares the composer's
    /// widget id with the chat screen's, and deliberately: only one of the two
    /// is ever on screen, they are the same gesture at two depths, and the IME
    /// bridge addresses exactly one field by that id (bl-014e).
    fn starter(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        if let Some(goal) = self.compose(ui, snap, "start a conversation", &["prepare", "prompt"])
            && let Some(model) = self.model()
        {
            model.start_conversation(goal);
        }
    }
}
