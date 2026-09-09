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
//! them, and the row paints those words rather than the number they came from.
//!
//! **And it names the client that acted** (REMOTE §9.20, PROTOCOL 16). A
//! shared workspace is several seats depositing into one conversation, and
//! until this field the record could not say which of them made an act; the
//! engine's own acts say `local`. It rides the first line beside the origin,
//! because who acted and which surface owes the row a reading are the same
//! kind of provenance and are read in one glance.
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
                            crate::shell::clip::weak(ui, "the engine has said nothing here yet");
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
/// One row: when, from where, **who made it** (REMOTE §9.20 — a leaf's common
/// name, or `local` for the engine's own acts, which is the fact a shared
/// workspace reads this trail to learn) and **the engine's own reading of what
/// it exited** (REMOTE §9.17) — never a number this seat interprets. A failed
/// row says so and says where it stands, so an operator can tell the alarm
/// (`live`) from one a newer clean run retired or an ack covered.
fn line(ui: &mut egui::Ui, row: &OpRow) {
    crate::shell::clip::weak(
        ui,
        format!(
            "{} · {} · {} · {}",
            row.ts, row.origin, row.client, row.exit_label
        ),
    );
    crate::shell::clip::said(ui, &row.argv);
    if row.failed {
        crate::shell::clip::tinted(
            ui,
            crate::shell::chat::tone_hue(ui, &crate::codec::Tone::Bad),
            format!("failed · {}", row.standing.word()),
        );
    }
    if !row.stderr.is_empty() {
        crate::shell::clip::weak(ui, &row.stderr);
    }
    ui.add_space(4.0);
}
