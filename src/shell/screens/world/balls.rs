//! **The ball pane** (DESIGN §13.9): the task store the conversations are
//! working, which this seat carried the wire for and painted nowhere. Until it
//! landed an operator could watch work happen and could not see the ball it
//! was happening on.
//!
//! **One screen, three views, and one control apiece rather than a switcher.**
//! Each read has exactly one affordance and it sits where its subject is: the
//! roster carries `balls` and `board`, because neither names a workspace, and
//! a workspace's own conversation list carries `workspace-balls`, because that
//! one does. A switcher inside the pane would have offered the aimed read from
//! a screen with nothing aimed, which is the wrong claim wearing a control.
//!
//! **A view paints only its own answer.** The pane holds one, tagged with the
//! read that produced it, so opening the board under a held ball list paints
//! the board's own emptiness rather than the list under the board's name —
//! §14's pairing law over rows and their focus, applied one surface along.
//!
//! **The acts hang on the aimed view alone** (bl-f36e, `balls::acts`): the
//! `--as` stamp every `bl` verb carries is a WORKSPACE name, and
//! `workspace-balls` is the one of the three reads that names one. A row of
//! that view is what a control here addresses — the project is the one fact
//! these acts need that only a row carries — so tapping a row picks it, and
//! the foot spends what is picked.
//!
//! **Two emptinesses and they are different sentences** (lernie DESIGN §4.31,
//! whose four this is the phone's two of): nobody has asked yet, and the
//! engine answered and there is nothing. One sentence over both would say
//! *there are no balls* about a question nobody put.

use eframe::egui;

mod acts;

use crate::codec::{BallRow, Board, BoardRow, Pane, View, WsBallRow};
use crate::seat::Snapshot;
use crate::shell::app::Shell;
use crate::shell::mark::Back;

impl Shell {
    pub(in crate::shell) fn balls(&mut self, ui: &mut egui::Ui, snap: &Snapshot, view: View) {
        self.note_screen(view.screen());
        // The acts claim the floor first and the listing takes what is left —
        // the trail's order (§13.8, bl-192c), for its reason: what claims the
        // floor is what may never be pushed off it. The two unaimed views
        // carry none, so on those this is the listing alone.
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            if view == View::Here {
                self.ball_acts(ui);
                ui.add_space(4.0);
            }
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                self.listed(ui, snap, view);
            });
        });
    }

    /// The chrome and the rows under it.
    fn listed(&mut self, ui: &mut egui::Ui, snap: &Snapshot, view: View) {
        if self.bar(ui, view.screen(), &Back::To("workspaces")) {
            self.close_world();
        }
        super::super::banner(ui, snap);
        ui.separator();
        egui::ScrollArea::vertical()
            .min_scrolled_height(0.0)
            .show(ui, |ui| {
                match snap.pane.clone().filter(|p| p.view() == view) {
                    None => {
                        ui.weak("nothing read yet");
                    }
                    Some(Pane::Everywhere(rows)) if rows.is_empty() => empty(ui),
                    Some(Pane::Here(rows)) if rows.is_empty() => empty(ui),
                    Some(Pane::Everywhere(rows)) => {
                        for row in rows {
                            anywhere(ui, &row);
                        }
                    }
                    Some(Pane::Here(rows)) => {
                        for row in rows {
                            self.picking(ui, &row);
                        }
                    }
                    Some(Pane::Board(board)) => painted(ui, &board),
                }
            });
    }

    /// **One ball this workspace holds, as a control** (bl-f36e): tapping it
    /// makes it what the foot's acts address. A pick is navigation and nothing
    /// else — no more durable than a scroll position — and it carries no
    /// `act:` tag, because it fires no op.
    fn picking(&mut self, ui: &mut egui::Ui, row: &WsBallRow) {
        let subject = (row.project.clone(), row.id.clone());
        let picked = self.ball.as_ref() == Some(&subject);
        let control = ui.add(
            egui::Button::new(held(row, picked))
                .min_size(egui::vec2(ui.available_width(), crate::shell::mark::TOUCH)),
        );
        if control.clicked() {
            // Tapping the picked row again puts it down: a pick with no way
            // out would leave every act live for a ball nobody is looking at.
            self.ball = (!picked).then_some(subject);
            self.armed = false;
        }
        ui.add_space(4.0);
    }
}

/// The answered-and-holding-nothing sentence, told apart from the unasked one.
fn empty(ui: &mut egui::Ui) {
    ui.weak("no balls here");
}

/// One ball wherever it is: what it is, then who holds it and where.
fn anywhere(ui: &mut egui::Ui, row: &BallRow) {
    ui.label(format!("{} · {} · {}", row.id, row.project, row.state));
    if !row.title.is_empty() {
        ui.weak(&row.title);
    }
    if !row.claimant.is_empty() {
        ui.weak(format!("held by {} · {}", row.claimant, row.workspace));
    }
    ui.add_space(4.0);
}

/// One ball this workspace holds, with the spend **as the engine rendered it**
/// — nothing here multiplies a token count by a rate of its own, and a figure
/// the engine did not state paints as nothing rather than as a zero. One label
/// rather than three widgets, because the row is a control now (bl-f36e) and a
/// control is one rectangle.
fn held(row: &WsBallRow, picked: bool) -> String {
    let mark = if picked { PICKED } else { "" };
    let line = [row.owner.clone(), row.usd.clone()]
        .into_iter()
        .filter(|said| !said.is_empty())
        .collect::<Vec<String>>()
        .join(" · ");
    [
        format!("{mark}{} · {} · {}", row.id, row.project, row.state),
        row.badge.clone(),
        line,
    ]
    .into_iter()
    .filter(|said| !said.is_empty())
    .collect::<Vec<String>>()
    .join("\n")
}

/// What marks the picked row. A glyph rather than a colour: the pane already
/// inks nothing, and a tint is the one distinction a monochrome screen and a
/// colour-blind reader both lose.
const PICKED: &str = "▸ ";

/// The board: what each armed loop is doing, in the engine's own sentence,
/// then the rows in the engine's own columns.
fn painted(ui: &mut egui::Ui, board: &Board) {
    for line in &board.fleet {
        ui.weak(line);
    }
    if !board.fleet.is_empty() {
        ui.separator();
    }
    if board.rows.is_empty() {
        empty(ui);
    }
    for row in &board.rows {
        column(ui, row);
    }
}

/// One board row, under the column word the engine minted for it.
fn column(ui: &mut egui::Ui, row: &BoardRow) {
    ui.label(format!("{} · {} · {}", row.column, row.id, row.project));
    if !row.title.is_empty() {
        ui.weak(&row.title);
    }
    if !row.claimant.is_empty() {
        ui.weak(format!("held by {}", row.claimant));
    }
    if !row.drones.is_empty() {
        ui.weak(format!("working: {}", row.drones.join(", ")));
    }
    if !row.gates.is_empty() {
        ui.weak(format!("waits on {}", row.gates.join(", ")));
    }
    ui.add_space(4.0);
}
