//! **The chat in start mode** (DESIGN §22 items 3 and 7, bl-3edc; §24 item
//! 3): a workspace focused and no conversation — the top line saying `new
//! conversation on <workspace>` between its two doors, and the prompt line
//! in start mode on the rule. The list this file used to paint stands in the
//! drawer now (`shell/drawer.rs`), which keeps this module's two halves of a
//! conversation row: the long press it carries (`rows/menu.rs`, §13.5) and
//! the threading it is painted with (`rows/thread.rs`, §13.20).

use eframe::egui;

use crate::seat::Snapshot;
use crate::shell::app::Shell;

mod menu;
mod thread;

pub(in crate::shell) use thread::connectors;

impl Shell {
    pub(crate) fn start(&mut self, ui: &mut egui::Ui, snap: &Snapshot, workspace: &str) {
        // The starter rides the floor of this screen, where the composer sits
        // on a conversation: starting one and speaking into one are the same
        // gesture to a thumb. **What claims the floor first is what may never
        // be pushed off it** (bl-192c).
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            // The rule at the bottom edge and the prompt line in start mode
            // on it (§23 item 4, the desktop's order).
            self.rule(ui, snap);
            self.starter(ui, snap, workspace);
            ui.add_space(4.0);
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                // **The top line** (§24 item 3): `≡`, the start's words in the
                // strip's place, `+`.
                self.tabs(ui, snap, Some(workspace));
                // The verb sheet, while the draft is a slash line (§22 item
                // 5) — the start shares the prompt line and its door.
                self.sheet(ui, snap);
            });
        });
        // **Back on the chat with nothing standing leaves the app** (§24
        // item 2): nothing here takes the press, so it falls through to
        // `shell::back`'s own act at the end of the pass.
    }

    /// The one field that starts a conversation. It shares the composer's
    /// widget id with the chat screen's, and deliberately: only one of the two
    /// is ever on screen, they are the same gesture at two depths, and the IME
    /// bridge addresses exactly one field by that id (bl-014e).
    fn starter(&mut self, ui: &mut egui::Ui, snap: &Snapshot, workspace: &str) {
        let hint = crate::draft::hint(Some(workspace), false);
        if let Some(goal) = self.compose(ui, snap, &hint, &["prepare", "prompt"])
            && let Some(model) = self.model()
        {
            model.start_conversation(goal);
        }
    }
}
