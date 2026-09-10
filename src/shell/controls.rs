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
//! **ONE row, since bl-0691** (operator ruling, amending bl-dfbb's second
//! band). Every control here is a chip in one band of chips (`docs/STYLE.md`)
//! and the width is divided by `place::row`: a control that names a verb gets
//! the width of its words, and the two selectors share what is left and elide
//! inside it. The second band cost the transcript forty-eight points of the
//! screen the transcript is the whole point of, and a model name half-read is
//! cheaper than a conversation three lines tall.
//!
//! **Tap is the act; there is no apply.** Picking a model IS the assignment
//! (§13.2), so nothing here holds a draft the operator could leave unsent,
//! and an engine that refuses one says so in the banner the model already
//! publishes. What the selectors show is what the workspace ACTUALLY has,
//! overtaking an optimistic pick as soon as the roles read lands (bl-e9f9).
//!
//! This file is the row itself — where it sits, what it drops when the focus
//! moves and the one read it asks for. WHICH controls the row has and what
//! each is showing is `controls/chip.rs`; the acts that are only offered
//! while something is running are `controls/stops.rs`, the two selectors are
//! `controls/pick.rs` and the §9.4 tuning pair is `controls/tune.rs`.

use eframe::egui;

use super::app::Shell;
use crate::seat::Snapshot;

use chip::Chip;

mod chip;
mod drop;
mod held;
mod pick;
mod stops;
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
            // **And what the row is gated on is asked for HERE, once per
            // workspace** (bl-0691). Two reads, one gesture: the assignments
            // say which provider the worker is on, and the provider rows
            // carry the two capability columns the tuning pair is dark or
            // live by (REMOTE §9.13, §9.14). Asking for the listing only when
            // a selector was OPENED left the gate answered by an empty
            // listing for as long as nobody opened one — both knobs dark on a
            // provider that takes both, and the priority control doing
            // nothing when tapped — and asking for the assignments only on a
            // focus MOVE left them unread on the workspace this seat resumed
            // onto, where no focus gesture ever fires. The surface that needs
            // an answer is what asks for it, once per workspace, not once per
            // frame and not once per act.
            self.ask_options();
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
            self.settled();
        }
        // **The tappable area, read off the rect the inset was already spent
        // into** (bl-78c2). `app::pass` shrinks what every screen is painted
        // into by the platform's bottom inset (bl-9cfd), and this `ui` is
        // that rect — so taking its two edges here reads the one spend
        // rather than performing a second one. The selectors need it because
        // an opened list is NOT laid out inside it: egui gives a popup an
        // `Area` of its own against the whole display, which is how a list
        // reached the gesture-nav zone (`shell::place`).
        let area = crate::shell::place::Band {
            top: ui.max_rect().top(),
            bottom: ui.max_rect().bottom(),
        };
        ui.scope(|ui| {
            ui.spacing_mut().interact_size.y = super::mark::TOUCH;
            band(ui, |ui| {
                // **Smaller words and tighter chips, in this band only**
                // (bl-0691, `docs/STYLE.md` §4). Six controls at body size
                // with a control's ordinary padding spend two hundred points
                // of a phone's row on three verbs, which leaves the two
                // selectors thirty points of text apiece — an elision so deep
                // it says nothing. The type scale's own `SMALL` and the `S`
                // step of the spacing scale buy the selectors back most of a
                // provider name, and they are set HERE rather than on the
                // block so the capability band below keeps the prose size the
                // sentence it carries needs.
                ui.style_mut().text_styles.insert(
                    egui::TextStyle::Button,
                    egui::FontId::proportional(crate::theme::type_scale::SMALL),
                );
                ui.spacing_mut().button_padding.x = crate::theme::space::S;
                let chips = self.chips(snap);
                let faces: Vec<String> = chips.iter().map(Chip::face).collect();
                let wants: Vec<f32> = chips
                    .iter()
                    .zip(&faces)
                    .map(|(chip, face)| chip.wants(ui, face))
                    .collect();
                let widths = crate::shell::place::row::widths(
                    ui.available_width(),
                    ui.spacing().item_spacing.x,
                    &wants,
                );
                for ((chip, face), wide) in chips.into_iter().zip(faces).zip(widths) {
                    self.paint(ui, snap, chip, face, wide, area);
                }
            });
            // **The sentence a dark control answered with** (bl-0691), above
            // the row it is about and nowhere else: a knob that cannot be set
            // says why when it is tapped, because on a phone there is no
            // hover to say it in and a control that is merely grey teaches
            // nothing. It stands until the next act replaces the picks.
            if let Some(said) = self.tuning_said.clone() {
                crate::shell::clip::tinted(
                    ui,
                    super::theme::ink(crate::theme::State::Annotation),
                    said,
                );
            }
            // **A band of its own, and only where a call is parked** (§13.7,
            // bl-b39d). It is added last, so in this bottom-up block it
            // stands highest — directly under the composer, where what it
            // says is read before what it offers is tapped. A held call is a
            // sentence and its answers, which is why it does not squeeze into
            // the row above: nothing else here has anything to read.
            self.capability(ui, snap);
        });
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
fn focused_row(snap: &Snapshot) -> Option<crate::codec::ConvRow> {
    let agent = snap.focus.agent.as_deref()?;
    snap.conversations
        .iter()
        .find(|row| row.root_id == agent)
        .cloned()
}
