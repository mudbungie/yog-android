//! **The controls row** (DESIGN §13.2, bl-0267): one row under the composer
//! carrying the acts that are about the CONVERSATION rather than about the
//! message being typed — which model answers it, what to do with a turn
//! already running, and how hard the model is asked to think.
//!
//! **Under the composer, inside the same floor.** It is the last thing added
//! to the bottom-up layout before the composer, so it sits between the
//! composer and the platform's floor (bl-9cfd) and rides the keyboard with
//! it. Its own height is the §13.2 touch floor, spent both as the row's
//! height and as the minimum interact size inside it — a control a thumb
//! misses is a defect, not a style.
//!
//! **Tap is the act; there is no apply.** Picking a model IS the assignment
//! (§13.2), so nothing here holds a draft the operator could leave unsent,
//! and an engine that refuses one says so in the banner the model already
//! publishes. What the selectors show is what the workspace ACTUALLY has,
//! overtaking an optimistic pick as soon as the roles read lands (bl-e9f9).
//!
//! This file is the row itself and the acts that are only offered while
//! something is running; the two selectors are `controls/pick.rs` and the
//! §9.4 tuning pair is `controls/tune.rs`. The seam is the one the row already
//! paints — a band of selectors, a band of knobs — rather than a line count.

use eframe::egui;

use super::app::Shell;
use crate::seat::Snapshot;

mod pick;
mod tune;

/// One row of controls, given its own height for the reason every row in this
/// app is (bl-193c): a `left_to_right(Center)` layout handed the rest of the
/// screen centres its widgets in it.
fn band(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), super::mark::TOUCH),
        egui::Layout::left_to_right(egui::Align::Center),
        add,
    );
}

impl Shell {
    /// The row. Painted only where a workspace is focused, because every
    /// control in it acts on one.
    pub(super) fn controls(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        let Some(workspace) = snap.focus.workspace.clone() else {
            return;
        };
        // The selection belongs to the workspace it was made in: another
        // workspace's pick is not a fact about this one, so it goes when the
        // focus does.
        if self.picked_in.as_deref() != Some(workspace.as_str()) {
            self.picked_in = Some(workspace);
            self.forget_picks();
        }
        // **Truth overtakes the guess** (bl-e9f9): every act on this row is
        // optimistic — the control shows the pick the moment it is tapped,
        // because the round trip is seconds — and the assignments read is
        // what settles it. When the seat's read count moves, whatever was
        // standing optimistically goes and what the workspace ACTUALLY has
        // is what paints. That covers a refusal for free: the engine never
        // took it, so the read never carries it, so the control snaps back
        // and the banner says why.
        if self.tuned_at != snap.roles_read {
            self.tuned_at = snap.roles_read;
            self.forget_picks();
        }
        let set = crate::codec::pick::worker(&snap.roles);
        // **What the picked provider will take** (bl-dfbb), read off its own
        // row in covered code: the paint asks, it never derives.
        let (effort, priority) = crate::codec::pick::tunable(
            &snap.providers,
            pick::provider(self, set.as_ref()).as_deref(),
        );
        ui.scope(|ui| {
            ui.spacing_mut().interact_size.y = super::mark::TOUCH;
            band(ui, |ui| {
                // The stop controls first: they are the acts an operator
                // reaches for while something is running, and they are
                // only there while it is (bl-48fa).
                self.stops(ui, snap);
                let wide = (ui.available_width() - ui.spacing().item_spacing.x) / 2.0;
                self.providers(ui, snap, set.as_ref(), wide);
                self.models(ui, snap, set.as_ref(), wide);
            });
            // **A second band, and only when there is something in it.** The
            // tuning controls cannot share the first: a selector narrow
            // enough to fit beside two others at a 320-point width is one an
            // operator cannot read a model name in (measured). egui's own
            // wrapping layout does not answer this — a `ComboBox` does not
            // declare its width to the wrap check, so it overflows the edge
            // instead of moving down (measured: 418 points in a 390-point
            // column) — so the second row is allocated rather than wrapped
            // into. It is the same controls block under the composer, not a
            // new place to look (§13.2).
            if effort || priority {
                band(ui, |ui| {
                    if effort {
                        self.effort(ui, set.as_ref());
                    }
                    if priority {
                        self.priority(ui, set.as_ref());
                    }
                });
            }
        });
    }

    /// Drop every optimistic pick. One helper because the two reasons to drop
    /// them — the focus moved, the truth arrived — drop exactly the same four.
    fn forget_picks(&mut self) {
        self.provider = None;
        self.model = None;
        self.effort = None;
        self.priority = None;
    }

    /// **The stop controls** (REMOTE §3.1, bl-48fa): shown by the gates the
    /// engine puts ON the row, never by a reading taken here — §9.4's rule is
    /// that a gate which is not derivable from a row goes on the row, and
    /// both of these are. They are independent: `stoppable` is true iff this
    /// conversation holds the executor lock, `stop_children` iff some other
    /// agent's id extends this one — so a quiet root with a working child
    /// offers *stop all* and no *stop*, which is exactly right and is why
    /// two gates cross rather than one.
    ///
    /// **The gesture is the op.** A deposited `/stop` line is content, and
    /// content wakes the very driver it meant to kill; the seat model sends
    /// the wire's own act.
    ///
    /// All three carry `act:` tags naming what they post. They are also the
    /// controls a walk can only see under their gate, which is why the
    /// `make screens` walk seeds a conversation in each state (DESIGN §15.4):
    /// a control that only exists on an unvisited screen is unproven, and
    /// unproven is red (PARITY §5).
    fn stops(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        let Some(row) = focused_row(snap) else {
            return;
        };
        let (stoppable, children) = (row.stoppable, row.stop_children);
        if stoppable {
            let control = ui.button("stop");
            super::act::act(ui, &control, "stop");
            if control.clicked() {
                self.stop_turn(false);
            }
        }
        if children {
            let control = ui.button("stop all");
            super::act::act(ui, &control, "stop");
            if control.clicked() {
                self.stop_turn(true);
            }
        }
        // **Nudge is the other half of the same question** (bl-d09e): stop is
        // for a turn that is running, nudge for a branch that stopped
        // advancing — so it is offered exactly when nothing is in flight,
        // read off the row's own `flight` (its `None` IS "at rest"). The
        // engine's own `nudgeable` gate rides the agent view this codec does
        // not spell; if the row's reading proves too coarse, the fix is that
        // gate on the row rather than a second derivation here.
        if row.flight.is_none() {
            let control = ui.button("nudge");
            super::act::act(ui, &control, "nudge");
            if control.clicked()
                && let Some(model) = self.model()
            {
                model.nudge();
            }
        }
    }

    /// Ask the worker to stop the focused turn.
    fn stop_turn(&self, children: bool) {
        if let Some(model) = self.model() {
            model.stop_turn(children);
        }
    }
}

/// The focused conversation's row, which is where every conversation-level
/// gate rides (REMOTE §9.4). A conversation the list has not caught up with
/// yet has no row and therefore no gates — the honest reading, and the same
/// one the roster's own display name falls back through.
fn focused_row(snap: &Snapshot) -> Option<crate::codec::ConvRow> {
    let agent = snap.focus.agent.as_deref()?;
    snap.conversations
        .iter()
        .find(|row| row.root_id == agent)
        .cloned()
}
