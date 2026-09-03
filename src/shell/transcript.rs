//! **The transcript screen**: the conversation as the desktop window paints
//! it, collapsing included (DESIGN §7). Split from `screens.rs` because it is
//! the one screen with mechanics rather than taps — the projected rows, the
//! fold overrides, and the composer that rides above the keyboard.
//!
//! The projection itself is `crate::rows`, which is pure and host-tested under
//! the 100% floor; everything here is paint. **The growing answer is one of
//! those rows and not a paint of its own** (bl-e3d1): the follow lane
//! freshens the transcript's own streaming entry (`crate::live`), so the tail
//! wears the speaker's name and dissolves structurally when the read stops
//! carrying it.

use eframe::egui;

use super::app::Shell;
use super::mark::Back;
use crate::rows::rows;
use crate::seat::Snapshot;

impl Shell {
    pub(super) fn transcript(
        &mut self,
        ui: &mut egui::Ui,
        snap: &Snapshot,
        workspace: &str,
        agent: &str,
    ) {
        let speaker = super::chat::speaker_of(snap, agent);
        let painted = rows(&snap.transcript, &speaker, self.auto, &self.folds);
        let mut flipped = None;
        // **The floor's order** (bl-192c): what is anchored to the platform's
        // floor claims its space FIRST, and the chrome and the transcript
        // take what is left above it. Painted the other way round — chrome
        // first, then a bottom-up stack in the remainder — a screen with more
        // rows than room pushed the controls and the composer straight
        // through the floor and under the gesture-nav bar, because a bound
        // rect is not a clip: `app::pass` bounds what a screen is GIVEN
        // (bl-9cfd), and nothing made the overflow give way. Measured at 320
        // and 400 points, keyboard up and down, with the tuning band shown:
        // the controls, the composer and the knobs all painted past the floor
        // before this, and none of them does after. That rig was a throwaway
        // and nothing of it was committed (bl-78c2 went looking) — so this
        // paragraph is a reading somebody took, not an assertion anything
        // re-runs. `shell::place` is where the geometry that IS asserted
        // lives.
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            // Closest to the floor: the acts on this conversation, then the
            // composer that rides above the keyboard.
            self.controls(ui, snap);
            if let Some(taken) =
                super::composer::composer(ui, &mut self.composer, "message", &["message"])
            {
                self.deposit(snap, taken);
            }
            ui.add_space(4.0);
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                if self.bar(ui, &speaker, &Back::To("conversations")) {
                    self.focus_workspace(Some(workspace.to_owned()));
                }
                super::screens::banner(ui, snap);
                // The two auto knobs, the desktop's own pair: which KINDS
                // open by default. They are policy; a hand-flipped row is the
                // override set below and dies with the screen.
                //
                // **Right-aligned, in a row of their own, and named in the
                // operator's words** (bl-f165). They sat at the left edge
                // directly under the bar's back control, one thumb-width from
                // the one gesture that leaves the screen — and were labelled
                // `talk` and `steps`, which are this file's words for them
                // and nobody else's. The row is allocated its own height for
                // the reason every row in this app now is (bl-193c): a
                // `right_to_left(Center)` layout handed the whole remaining
                // screen centres its widgets in it. The 44 is §13.2's floor,
                // spent here as the minimum interact size so each checkbox is
                // a target rather than a glyph.
                ui.scope(|ui| {
                    ui.spacing_mut().interact_size.y = super::mark::TOUCH;
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), super::mark::TOUCH),
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            ui.checkbox(&mut self.auto.others, "show intermediate steps");
                            ui.checkbox(&mut self.auto.responses, "show full response");
                        },
                    );
                });
                ui.separator();
                // **The transcript is what gives way** (bl-192c): it takes
                // whatever the two above it left, down to nothing.
                // `min_scrolled_height` is why that is not automatic — a
                // vertical `ScrollArea` refuses to be shorter than 64 points
                // however little it is given, which is the same defect the
                // composer's own scroller had (bl-9cfd), one level up.
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .min_scrolled_height(0.0)
                    .show(ui, |ui| {
                        // Top-down inside the scroller: the rows are in
                        // message order and the bottom-up layout above is
                        // about where the composer sits, not about which way
                        // a transcript reads.
                        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                            for row in &painted {
                                if super::chat::row(ui, row) {
                                    flipped = Some(row.key.clone());
                                }
                            }
                            // The message this seat has sent and the engine
                            // has not shown back yet (bl-66fb), where its row
                            // will be — above the answer to it.
                            if let Some(echo) = &self.echo {
                                super::chat::echo(ui, &echo.text, echo.fate);
                            }
                        });
                    });
            });
        });
        // Applied after the walk: flipping mid-iteration would re-project the
        // rows the loop is still reading.
        if let Some(key) = flipped
            && !self.folds.remove(&key)
        {
            self.folds.insert(key);
        }
    }
}

impl Shell {
    /// **Send, and echo it at once** (bl-66fb). The deposit crosses the wire
    /// on the model's worker and answers in its own time; what the operator
    /// sees immediately is this — their own words, muted, where the row will
    /// be. The counters are remembered here because the echo's other two
    /// states are read off their movement.
    fn deposit(&mut self, snap: &Snapshot, text: String) {
        let Some(model) = self.model() else { return };
        model.deposit(text.clone());
        self.echo = Some(crate::outbox::Echo::sent(text, snap));
    }

    /// **What became of the echo**, run once per frame before the transcript
    /// is painted. The rule is `crate::outbox`'s and is proven there; this is
    /// the one thing this file does with each of its three answers.
    pub(super) fn settle_echo(&mut self, snap: &Snapshot) {
        let Some(echo) = self.echo.take() else { return };
        match echo.settle(snap) {
            crate::outbox::Settled::Standing(echo) => self.echo = Some(echo),
            crate::outbox::Settled::Gone => {}
            crate::outbox::Settled::Draft(text) => self.composer = text,
        }
    }
}
