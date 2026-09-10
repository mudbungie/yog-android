//! **The acts on a turn already running** (REMOTE §3.1, bl-48fa), split from
//! the row that carries them (bl-0691) when the row became one band of chips:
//! the row is layout and the sentence a dark control answers with, and these
//! three are the conversation's own verbs.
//!
//! **Shown by the gates the engine puts ON the row, never by a reading taken
//! here** — §9.4's rule is that a gate which is not derivable from a row goes
//! on the row, and both stop gates are. They are independent: `stoppable` is
//! true iff this conversation holds the executor lock, `stop_children` iff
//! some other agent's id extends this one — so a quiet root with a working
//! child offers *stop all* and no *stop*, which is exactly right and is why
//! two gates cross rather than one.
//!
//! **The gesture is the op.** A deposited `/stop` line is content, and
//! content wakes the very driver it meant to kill; the seat model sends the
//! wire's own act.
//!
//! All three carry `act:` tags naming what they post. They are also the
//! controls a walk can only see under their gate, which is why the `make
//! screens` walk seeds a conversation in each state (DESIGN §15.4): a control
//! that only exists on an unvisited screen is unproven, and unproven is red
//! (PARITY §5).

use eframe::egui;

use super::super::app::Shell;
use super::{Chip, focused_row};
use crate::seat::Snapshot;

/// Which of the three the focused conversation offers, in the row's order.
pub(super) fn chips(snap: &Snapshot) -> Vec<Chip> {
    let Some(row) = focused_row(snap) else {
        return Vec::new();
    };
    let mut chips = Vec::new();
    if row.stoppable {
        chips.push(Chip::Stop(false, "stop".to_owned()));
    }
    if row.stop_children {
        chips.push(Chip::Stop(true, "stop all".to_owned()));
    }
    // **Nudge is the other half of the same question** (bl-d09e): stop is for
    // a turn that is running, nudge for a branch that stopped advancing — so
    // it is offered exactly when nothing is in flight, read off the row's own
    // `flight` (its `None` IS "at rest"). The engine's own `nudgeable` gate
    // rides the agent view this codec does not spell; if the row's reading
    // proves too coarse, the fix is that gate on the row rather than a second
    // derivation here.
    if row.flight.is_none() {
        chips.push(Chip::Nudge);
    }
    chips
}

impl Shell {
    /// Stop the focused turn, or every turn under it.
    pub(super) fn stop(&mut self, ui: &mut egui::Ui, face: String, wide: f32, children: bool) {
        let control = crate::shell::theme::chip(ui, face.into(), wide, true);
        super::super::act::act(ui, &control, "stop");
        if control.clicked()
            && let Some(model) = self.model()
        {
            model.stop_turn(children);
        }
    }

    /// Wake a branch that stopped advancing.
    pub(super) fn nudge(&mut self, ui: &mut egui::Ui, face: String, wide: f32) {
        let control = crate::shell::theme::chip(ui, face.into(), wide, true);
        super::super::act::act(ui, &control, "nudge");
        if control.clicked()
            && let Some(model) = self.model()
        {
            model.nudge();
        }
    }
}
