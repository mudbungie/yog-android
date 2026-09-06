//! **The records screen** (DESIGN §13.11): the conversation's machinery, one
//! drill-down depth behind the transcript. What the conversation IS and may be
//! done to, what its steps were and what one of them recorded, the operable
//! spine and the config commit governing it, and the mail nothing has taken.
//!
//! **One screen and not six**, because the subject is one noun. Six covering
//! surfaces over one conversation would be six places to look for one thing,
//! and the reads stand or fall together anyway (`seat::asks::records`).
//!
//! **Opening is the ask**, the trail's rule and the ball pane's (§13.8,
//! §13.9): a screen nobody has opened costs this device no radio.
//!
//! **Each act is addressed at a picked row**, which is the ball pane's
//! arrangement exactly (§13.10) and is a departure from the desktop's — there
//! the control sits ON each row, because a desktop row has width for a second
//! rectangle and a phone's has not. So a steps row is a control that picks, a
//! fork point is another (§13.16), the foot carries both acts, and a dark one
//! says in its own label what would light it.
//!
//! **Two emptinesses and they are different sentences** (§13.9's pair, at a
//! second site): nobody has asked yet, and the engine answered.

use eframe::egui;

mod acts;
mod parts;
mod points;

use parts::{head, mail, orphan, spine, step_line};

use crate::codec::{Records, StepRow};
use crate::seat::Snapshot;
use crate::shell::app::Shell;
use crate::shell::mark::{Back, TOUCH};

impl Shell {
    /// The whole screen. The foot claims its band first and the listing takes
    /// what is left, which is the floor's order (§13.8, bl-192c).
    pub(super) fn records(
        &mut self,
        ui: &mut egui::Ui,
        snap: &Snapshot,
        workspace: &str,
        agent: &str,
    ) {
        self.note_screen("records");
        let held = snap
            .records
            .clone()
            .filter(|records| records.about(workspace, agent));
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            self.records_acts(ui, workspace, agent);
            ui.add_space(4.0);
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                if self.bar(ui, "records", &Back::To("transcript")) {
                    self.close_records();
                }
                super::banner(ui, snap);
                ui.separator();
                egui::ScrollArea::vertical()
                    .min_scrolled_height(0.0)
                    .show(ui, |ui| match held {
                        None => {
                            ui.weak("nothing read yet");
                        }
                        Some(records) => self.body(ui, &records),
                    });
            });
        });
    }

    /// The six halves, in the order they answer: what it is, what it is
    /// anchored to, what it did, and what is waiting for it.
    fn body(&mut self, ui: &mut egui::Ui, records: &Records) {
        head(ui, records);
        ui.separator();
        spine(ui, records);
        // **The fork points, between what governs the conversation and what it
        // did** (§13.16): they are what a child would start from, so they read
        // beside the policy rather than under the census.
        self.points(ui, records);
        ui.separator();
        orphan(ui, records);
        for row in &records.steps.rows {
            self.picking_step(ui, row);
            // **The answer says which row it is under**, so this asks it
            // rather than remembering a second name for the open step.
            if let Some(step) = records.drilled.as_ref().filter(|step| step.seq == row.seq) {
                parts::drilled(ui, step);
            }
        }
        if records.steps.rows.is_empty() {
            ui.weak("no steps here");
        }
        ui.separator();
        mail(ui, records);
    }

    /// **One step, as a control** (§13.10's row): tapping it makes it what the
    /// foot's act addresses, and tapping it again puts it down. A pick is
    /// navigation and carries no `act:` tag, because it fires no op.
    fn picking_step(&mut self, ui: &mut egui::Ui, row: &StepRow) {
        let picked = self.step.as_deref() == Some(row.seq.as_str());
        let control = ui.add(
            egui::Button::new(step_line(row, picked))
                .min_size(egui::vec2(ui.available_width(), TOUCH)),
        );
        if control.clicked() {
            self.step = (!picked).then(|| row.seq.clone());
        }
        ui.add_space(4.0);
    }

    /// **Open it** — and that is the ask (§13.11). The pick goes with the
    /// opening: a step picked on one visit is not picked on the next, because
    /// the act addresses a row and a row nobody can see is not one.
    pub(in crate::shell) fn open_records(&mut self) {
        self.records = true;
        self.step = None;
        self.from = None;
        self.armed = false;
        if let Some(model) = self.model() {
            model.open_records();
        }
    }

    /// Leave, back to the transcript.
    fn close_records(&mut self) {
        self.records = false;
        self.step = None;
        self.from = None;
        self.armed = false;
    }
}
