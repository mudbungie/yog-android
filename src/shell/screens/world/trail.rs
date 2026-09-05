//! **The ops trail** (DESIGN §13.8): what the engine last did, and the two
//! acts an operator has over it.
//!
//! **A row paints and does not tap.** A trail line addresses nothing this
//! device could open — it is the record of an action that has already
//! happened — so it is a line rather than a control, which is the answer a
//! ball hit gets on the search screen for the same reason (§13.6).
//!
//! **Nothing here reads a verdict out of an exit number.** yog derives what a
//! failed action IS four ways and put all four on the wire (REMOTE §9.17:
//! `failed`, `exit_label`, `standing`) precisely so that no seat re-implements
//! them; the corpus this build is vendored against predates that bump, so the
//! row states the engine's own three facts and stops. Reading the words is
//! bl-8e3c's, with the re-vendor that brings them.
//!
//! **`clear-trail` is the first armed control in this app.** Every gesture
//! this seat had until now kept what it acted on; this one discards a durable
//! record — the record every other recovery sentence in this client points at
//! (REMOTE §9.8). The arm is two taps on one control, spelled in the control's
//! own label rather than behind a dialog: a phone's back gesture must dismiss
//! anything modal, so a confirmation a back press can answer is one nobody
//! read.

use eframe::egui;

use crate::codec::OpRow;
use crate::seat::Snapshot;
use crate::shell::app::Shell;
use crate::shell::mark::Back;

impl Shell {
    pub(super) fn trail(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        self.note_screen("trail");
        // The acts claim the floor first and the record takes what is left —
        // the conversation list's order (bl-192c), for its reason: what
        // claims the floor is what may never be pushed off it.
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            self.acts(ui);
            ui.add_space(4.0);
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                if self.bar(ui, "trail", &Back::To("workspaces")) {
                    self.close_world();
                }
                super::super::banner(ui, snap);
                ui.separator();
                egui::ScrollArea::vertical()
                    .min_scrolled_height(0.0)
                    .show(ui, |ui| {
                        if snap.trail.is_empty() {
                            ui.weak("the engine has said nothing here yet");
                        }
                        // Newest first: the engine answers its tail in the
                        // order it happened, and a thumb arrives at the top.
                        for row in snap.trail.iter().rev() {
                            line(ui, row);
                        }
                    });
            });
        });
    }

    /// The two acts. `ack` is a tap; the truncation takes two, and says so in
    /// its own label.
    fn acts(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let ack = ui.button("ack");
            crate::shell::act::act(ui, &ack, "ack");
            if ack.clicked() {
                self.armed = false;
                if let Some(model) = self.model() {
                    model.ack_trail();
                }
            }
            let clear = ui.button(if self.armed {
                "clear trail · tap again"
            } else {
                "clear trail"
            });
            crate::shell::act::act(ui, &clear, "clear-trail");
            if !clear.clicked() {
                return;
            }
            if std::mem::take(&mut self.armed) {
                if let Some(model) = self.model() {
                    model.clear_trail();
                }
            } else {
                self.armed = true;
            }
        });
    }
}

/// One line of the record: when, where it came from, what it exited, and what
/// it said.
/// One row: when, from where, and **the engine's own reading of what it
/// exited** (REMOTE §9.17) — never a number this seat interprets. A failed
/// row says so and says where it stands, so an operator can tell the alarm
/// (`live`) from one a newer clean run retired or an ack covered.
fn line(ui: &mut egui::Ui, row: &OpRow) {
    ui.weak(format!("{} · {} · {}", row.ts, row.origin, row.exit_label));
    ui.label(&row.argv);
    if row.failed {
        ui.colored_label(
            crate::shell::chat::tone_hue(ui, crate::codec::Tone::Bad),
            format!("failed · {}", row.standing.word()),
        );
    }
    if !row.stderr.is_empty() {
        ui.weak(&row.stderr);
    }
    ui.add_space(4.0);
}
