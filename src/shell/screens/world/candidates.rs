//! **The candidates screen** (DESIGN §13.12): what each attempt on this
//! workspace's obligations cost, and the three acts that spread, accept and
//! release them.
//!
//! **It is the ball pane's aimed view one noun along.** `science` names a
//! workspace and nothing else, so it is offered where that place is what the
//! operator is standing in — the workspace's own conversation list, beside
//! `workspace-balls` (§13.9's placement rule, at a second site).
//!
//! **Opening is the ask**, and here that is sharper than the trail's version
//! of the same rule: `science` is DERIVED when it is asked and nothing behind
//! it is stored, so the same row a minute later is a statement about the world
//! a minute later. A posted-once read would paint a moment that had passed.
//!
//! **The row says which acts it earns.** A row with a handle is a candidate on
//! `attempt/<handle>` waiting to be accepted or released; a row without one is
//! the ball's own claim, whose delivery obligation is the thing a fan spreads.
//! So the three controls are not a mode this screen holds — they are what the
//! row IS, and every value they take is already on it (`codec::candidates`).
//!
//! **Two emptinesses, two sentences** (§13.9's pair, at a third site): nobody
//! has asked yet, and the engine answered with none.

use eframe::egui;

mod acts;

pub(in crate::shell) use acts::FLOOR;

use crate::codec::{Attempt, Spread};
use crate::seat::Snapshot;
use crate::shell::app::Shell;
use crate::shell::mark::{Back, TOUCH};

impl Shell {
    pub(in crate::shell) fn candidates(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        self.note_screen(SCREEN);
        let held = snap
            .candidates
            .clone()
            .filter(|spread| spread.about(snap.focus.workspace.as_deref().unwrap_or_default()));
        // The acts claim the floor first and the listing takes what is left —
        // the trail's order (§13.8, bl-192c), for its reason.
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            self.candidate_acts(ui);
            ui.add_space(4.0);
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                if self.bar(ui, SCREEN, &Back::To("conversations")) {
                    self.close_world();
                }
                super::super::banner(ui, snap);
                ui.separator();
                egui::ScrollArea::vertical()
                    .min_scrolled_height(0.0)
                    .show(ui, |ui| self.attempts(ui, held.as_ref()));
            });
        });
    }

    /// The rows, or the sentence saying which absence this is.
    fn attempts(&mut self, ui: &mut egui::Ui, held: Option<&Spread>) {
        let Some(spread) = held else {
            ui.weak("nothing read yet");
            return;
        };
        if spread.rows.is_empty() {
            ui.weak("no attempts here");
        }
        for row in &spread.rows {
            self.picking_attempt(ui, row);
        }
    }

    /// **One attempt, as a control**: tapping it makes it what the foot's acts
    /// address, tapping it again puts it down. A pick is navigation and
    /// carries no `act:` tag, because it fires no op (§13.10's row).
    ///
    /// What the attempt SAID — its goal, its answer, what was said about it —
    /// paints under the picked row alone. It is prose, and prose under every
    /// row would make a listing nobody can scan.
    fn picking_attempt(&mut self, ui: &mut egui::Ui, row: &Attempt) {
        let subject = (
            row.diff.project.clone(),
            row.diff.ball.clone(),
            row.diff.handle.clone(),
        );
        let picked = self.candidate.as_ref() == Some(&subject);
        let control = ui.add(
            egui::Button::new(line(row, picked)).min_size(egui::vec2(ui.available_width(), TOUCH)),
        );
        if control.clicked() {
            self.candidate = (!picked).then_some(subject);
        }
        if picked {
            said(ui, &row.goal);
            said(ui, &row.response);
            for verdict in &row.verdicts {
                ui.weak(format!("{} · {}", verdict.sender, verdict.body));
            }
        }
        ui.add_space(4.0);
    }
}

/// The screen's name, the op it asks and the harness's tap target, which are
/// one word because the surface has one read (§15.2).
pub(in crate::shell) const SCREEN: &str = "science";

/// One row's label. **The handle is the discriminant and the label says which
/// it is**, because that is what decides which controls below are live.
fn line(row: &Attempt, picked: bool) -> String {
    let mark = if picked { "▸ " } else { "" };
    let which = if row.diff.handle.is_empty() {
        "the claim".to_owned()
    } else {
        row.diff.handle.clone()
    };
    let landed = [row.commit.clone(), row.by.clone()]
        .into_iter()
        .filter(|said| !said.is_empty())
        .collect::<Vec<String>>()
        .join(" · ");
    [
        format!("{mark}{} · {} · {which}", row.diff.ball, row.diff.project),
        [row.outcome.clone(), row.diff.state.clone(), landed]
            .into_iter()
            .filter(|said| !said.is_empty())
            .collect::<Vec<String>>()
            .join(" · "),
        format!("{} steps · {}s", row.steps, row.wall_secs),
    ]
    .join("\n")
}

/// One line of the engine's own words, painted only where there are any: an
/// absent field is a fact and paints as nothing (`codec::balls`' rule).
fn said(ui: &mut egui::Ui, text: &str) {
    if !text.is_empty() {
        ui.weak(text);
    }
}
