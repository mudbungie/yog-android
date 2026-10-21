//! **The rule's knobs** (DESIGN §13.2, bl-0267; §22 items 2 and 10,
//! bl-79bb): which workspace, provider and model the worker runs on, and how
//! hard it is asked to think. They rode a band of chips under the composer;
//! by the operator's ruling of 2026-10-12 they stand on the status rule as
//! dropups (`shell/status.rs` paints the rule), and the band is gone.
//!
//! **Tap is the act; there is no apply.** Picking a model IS the assignment
//! (§13.2), so nothing here holds a draft the operator could leave unsent,
//! and an engine that refuses one says so in the banner the model already
//! publishes. What the selectors show is what the workspace ACTUALLY has,
//! overtaking an optimistic pick as soon as the roles read lands (bl-e9f9).
//!
//! This file is what the knobs FORGET when the focus moves and the one read
//! they ask for. Which knobs stand and what each shows is `controls/chip.rs`;
//! the selectors are `controls/pick.rs`, the §9.4 tuning pair
//! `controls/tune.rs` and the list they open `controls/drop.rs`; a parked
//! call is answered in the ask (`shell/ask.rs`, §22 item 8). The stops that
//! rode the old band are verbs of the sheet (§22 items 5 and 10,
//! `shell/slash.rs`).

use eframe::egui;

use super::app::Shell;
use crate::seat::Snapshot;

pub(super) use chip::{Chip, Seat};

mod chip;
mod drop;
mod pick;
mod tune;

/// One row of controls, given its own height for the reason every row in this
/// app is (bl-193c): a `left_to_right(Center)` layout handed the rest of the
/// screen centres its widgets in it.
pub(super) fn band(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), super::mark::TOUCH),
        egui::Layout::left_to_right(egui::Align::Center),
        add,
    );
}

impl Shell {
    /// **What the knobs follow, once a frame**: a pick belongs to the
    /// workspace it was made in, and the assignments read overtakes an
    /// optimistic one. Called by the rule before it paints.
    pub(super) fn follow(&mut self, snap: &Snapshot) {
        let Some(workspace) = snap.focus.workspace.clone() else {
            return;
        };
        if self.picked_in.as_deref() != Some(workspace.as_str()) {
            self.picked_in = Some(workspace);
            self.forget_picks();
            // **What the knobs are gated on is asked for HERE, once per
            // workspace** (bl-0691): the assignments say which provider the
            // worker is on, and the provider rows carry the two capability
            // columns the tuning pair is dark or live by (REMOTE §9.13,
            // §9.14). Not once per frame and not once per act.
            self.ask_options();
        }
        // **Truth overtakes the guess** (bl-e9f9): when the seat's read count
        // moves, whatever was standing optimistically goes and what the
        // workspace ACTUALLY has is what paints — a refusal included.
        if self.tuned_at != snap.roles_read {
            self.tuned_at = snap.roles_read;
            self.settled();
        }
    }

    /// Drop every optimistic pick, and the sentence a dark control left
    /// standing: the focus has moved, so none of the five is about the
    /// workspace now in front of the operator.
    fn forget_picks(&mut self) {
        self.provider = None;
        self.settled();
    }

    /// **What an assignments read overtakes** (bl-e9f9): the three values a
    /// gesture WROTE, and the sentence beside them. The provider is not among
    /// them and that is the whole distinction (bl-0691): picking a provider
    /// posts nothing — the act is the model pick, which names both — so there
    /// is no write for the read to confirm or refuse, and dropping it turned
    /// *set an effort level* into *lose the provider you were choosing a
    /// model under*, with the tuning knobs going dark in the same frame
    /// because their gate is that provider's own row. It is a path, not an
    /// assignment, and it goes when the focus does.
    fn settled(&mut self) {
        self.model = None;
        self.effort = None;
        self.priority = None;
        self.tuning_said = None;
    }

    /// Ask for what this row shows and is gated on — the workspace's role
    /// assignments and its provider rows, fired as the one gesture they are.
    fn ask_options(&self) {
        if let Some(model) = self.model() {
            model.read_options();
        }
    }
}

/// The focused conversation's row, which is where every conversation-level
/// gate rides (REMOTE §9.4). A conversation the list has not caught up with
/// yet has no row and therefore no gates — the honest reading, and the same
/// one the roster's own display name falls back through.
pub(super) fn focused_row(snap: &Snapshot) -> Option<crate::codec::ConvRow> {
    let agent = snap.focus.agent.as_deref()?;
    snap.conversations
        .iter()
        .find(|row| row.root_id == agent)
        .cloned()
}
