//! **The op table** (DESIGN §13.14): every gesture the engine speaks, what it
//! is called, which surface owes it a control, and what it does.
//!
//! **It costs no wire read.** The table is `crate::help::TABLE`, vendored into
//! this repository and compiled in — and for any engine this build can talk
//! to it IS that engine's table, because §2 rules that the corpus and the
//! spoken version move together and a peer of another version is refused at
//! the §3 preface. So this screen works with nothing dialled, which is what
//! makes it the right shape for the surface an operator opens when they do not
//! know what a control does.
//!
//! **It sits at the top depth**, beside the queue and the trail, for their
//! reason exactly: what it is about is not a workspace and not a conversation,
//! and the roster is the screen where the whole world is already on the glass.
//!
//! **It is a listing and carries no control.** The `act:help` tag rides the
//! entry that OPENS it — PARITY §2's *"the owed interactable for a read is the
//! affordance that reaches the view it populates"* — and there is nothing to
//! fire from inside, because reading is all this surface does.

use eframe::egui;

use crate::seat::Snapshot;
use crate::shell::app::Shell;
use crate::shell::mark::Back;

/// The screen's name and the harness's tap target (§15.2), which is also the
/// op token the entry is tagged with.
pub(in crate::shell) const SCREEN: &str = "help";

impl Shell {
    pub(in crate::shell) fn help(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        self.note_screen(SCREEN);
        if self.bar(ui, SCREEN, &Back::To("workspaces")) {
            self.close_world();
        }
        super::super::banner(ui, snap);
        ui.separator();
        // **The table is read at paint rather than held**, which is what a
        // compiled-in constant makes affordable: there is no answer to keep in
        // step with, and a copy on the model would be a second home for bytes
        // the binary already carries. A read that cannot fail on any device
        // this build runs on still says so if it ever does.
        let rows = match crate::help::rows(crate::help::TABLE) {
            Ok(rows) => rows,
            Err(why) => {
                ui.colored_label(egui::Color32::LIGHT_RED, why);
                return;
            }
        };
        egui::ScrollArea::vertical()
            .min_scrolled_height(0.0)
            .show(ui, |ui| {
                for row in rows {
                    ui.label(format!("{} · {}", row.verb, row.surface));
                    for said in [&row.usage, &row.summary, &row.detail] {
                        if !said.is_empty() {
                            ui.weak(said);
                        }
                    }
                    ui.add_space(4.0);
                }
            });
    }
}
