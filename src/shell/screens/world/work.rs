//! **The work screen** (DESIGN §13.15): what this workspace's attempts
//! actually changed, and the bytes of any one changed file.
//!
//! **It is the candidates screen's other half, off the same row** (§13.12). A
//! science row's `diff` IS a work-diff row — upstream encodes both with one
//! encoder — so the two screens read one shape (`codec::workdiff`) and differ
//! in the question: what an attempt COST is over there, and what it CHANGED
//! is here. That is why the churn and the refs ride through unread on the
//! candidates screen and are painted on this one.
//!
//! **It is aimed at the workspace it was opened on**, like the three entries
//! beside it: `work-diff` names a workspace and nothing else, so it is offered
//! where that place is what the operator is standing in.
//!
//! **Opening is the ask, and a changed file is the same ask one depth down.**
//! A work diff is derived when it is asked and nothing behind it is stored, so
//! the same row a minute later is a statement about the world a minute later.
//! The answer to a file's ask carries the whole listing back with it, so
//! nothing here merges a patch into rows it was not read beside.
//!
//! **Every row's churn paints.** The churn IS the answer — a listing that hid
//! it behind a pick would be a work diff that says nothing about work — so
//! there is no picked row on this screen and no state for one.

use eframe::egui;

use crate::codec::{Churn, Diff, Work, WorkFile};
use crate::seat::Snapshot;
use crate::shell::app::Shell;
use crate::shell::mark::{Back, TOUCH};

/// The screen's name, the op it asks and the harness's tap target.
pub(in crate::shell) const SCREEN: &str = "work-diff";

impl Shell {
    /// The whole screen. No foot: every act here is a row.
    pub(in crate::shell) fn work(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        self.note_screen(SCREEN);
        let held = snap
            .work
            .clone()
            .filter(|work| work.about(snap.focus.workspace.as_deref().unwrap_or_default()));
        if self.bar(ui, SCREEN, &Back::To("conversations")) {
            self.close_world();
        }
        super::super::banner(ui, snap);
        ui.separator();
        egui::ScrollArea::vertical()
            .min_scrolled_height(0.0)
            .show(ui, |ui| match held {
                None => {
                    ui.weak("nothing read yet");
                }
                Some(work) => self.churned(ui, &work),
            });
    }

    /// The rows, or the sentence saying the engine answered with none.
    fn churned(&mut self, ui: &mut egui::Ui, work: &Work) {
        if work.rows.is_empty() {
            ui.weak("nothing changed here");
        }
        for row in &work.rows {
            ui.weak(head(row));
            for file in &row.files {
                self.changed(ui, work, row, file);
            }
            if row.truncated {
                ui.weak("the churn was cut short");
            }
            ui.separator();
        }
    }

    /// One changed file, as a control: tapping it asks for its patch, and the
    /// patch paints under the row it was asked for.
    fn changed(&mut self, ui: &mut egui::Ui, work: &Work, row: &Diff, file: &Churn) {
        let asked = WorkFile {
            ball: row.ball.clone(),
            path: file.path.clone(),
            handle: row.handle.clone(),
        };
        let control = ui
            .add(egui::Button::new(churn(file)).min_size(egui::vec2(ui.available_width(), TOUCH)));
        crate::shell::act::act(ui, &control, SCREEN);
        if control.clicked()
            && let Some(model) = self.model()
        {
            model.open_work(Some(asked.clone()));
        }
        if work.opened.as_ref() == Some(&asked)
            && let Some(patch) = work.patch.as_ref()
        {
            crate::shell::screens::preview::bytes(ui, patch);
        }
        ui.add_space(4.0);
    }
}

/// One attempt's heading: whose work it is, which attempt, and what the state
/// token could say. An `unreadable` project states no refs and an `absent` one
/// names what is missing — the state decides, so the line does too.
fn head(row: &Diff) -> String {
    let which = if row.handle.is_empty() {
        "the claim".to_owned()
    } else {
        row.handle.clone()
    };
    let refs = match row.state.as_str() {
        "absent" => format!("{} — missing {}", row.state, row.missing.join(", ")),
        "diff" => format!("{} · {}..{}", row.state, row.target, row.source),
        _ => row.state.clone(),
    };
    let delivered = if row.delivered.is_empty() {
        String::new()
    } else {
        format!(" · delivered {}", row.delivered)
    };
    format!(
        "{} · {} · {which}\n{refs}{delivered}",
        row.ball, row.project
    )
}

/// One file's churn. Binary says so instead of counts, because upstream
/// counted no lines in it.
fn churn(file: &Churn) -> String {
    if file.binary {
        format!("{} · binary", file.path)
    } else {
        format!("{} · +{} −{}", file.path, file.added, file.removed)
    }
}
