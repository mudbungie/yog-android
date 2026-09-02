//! **The transcript screen**: the conversation as the desktop window paints
//! it, collapsing included (DESIGN §7). Split from `screens.rs` because it is
//! the one screen with mechanics rather than taps — the projected rows, the
//! fold overrides, and the composer that rides above the keyboard.
//!
//! The projection itself is `crate::rows`, which is pure and host-tested under
//! the 100% floor; everything here is paint.

use eframe::egui;

use super::app::Shell;
use crate::rows::rows;
use crate::seat::Snapshot;

impl Shell {
    pub(super) fn transcript(&mut self, ui: &mut egui::Ui, snap: &Snapshot, agent: &str) {
        // The two auto knobs, the desktop's own pair: which KINDS open by
        // default. They are policy; a hand-flipped row is the override set
        // below and dies with the screen. (The heading and the way back are
        // the bar's — `screens.rs` spelled both before this body ran.)
        //
        // **Right-aligned, in a row of their own, and named in the
        // operator's words** (bl-f165). They sat at the left edge directly
        // under the bar's back control, one thumb-width from the one gesture
        // that leaves the screen — and were labelled `talk` and `steps`,
        // which are this file's words for them and nobody else's. The row is
        // allocated its own height for the reason every row in this app now
        // is (bl-193c): a `right_to_left(Center)` layout handed the whole
        // remaining screen centres its widgets in it, which would paint these
        // two halfway down the transcript. The 44 is §13.2's floor, spent
        // here as the minimum interact size so each checkbox is a target
        // rather than a glyph.
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
        let speaker = super::chat::speaker_of(snap, agent);
        let painted = rows(&snap.transcript, &speaker, self.auto, &self.folds);
        // Bottom-up: the composer rides above the keyboard (or the gesture-
        // nav bar), then the transcript takes whatever height remains. The
        // inset itself is not spent here — `app::pass` shrank the rect this
        // screen is painted into, so the bottom of this layout IS the floor
        // (bl-9cfd).
        let mut flipped = None;
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            // Added first, so it sits UNDER the composer and above the
            // platform's floor: the conversation-level acts (§13.2's
            // controls row, bl-0267).
            self.controls(ui, snap);
            if let Some(taken) = super::composer::composer(ui, &mut self.composer, "message") {
                self.deposit(snap, taken);
            }
            ui.add_space(4.0);
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    // Top-down inside the scroller: the rows are in message
                    // order and the bottom-up layout above is about where the
                    // composer sits, not about which way a transcript reads.
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        for row in &painted {
                            if super::chat::row(ui, row) {
                                flipped = Some(row.key.clone());
                            }
                        }
                        // The message this seat has sent and the engine has
                        // not shown back yet (bl-66fb), where its row will
                        // be — above the answer to it.
                        if let Some(echo) = &self.echo {
                            super::chat::echo(ui, &echo.text, echo.landed);
                        }
                        // The answer still being written, under the rows the
                        // engine has written down (bl-4822). The scroller
                        // sticks to the bottom, so growth here is what the
                        // operator watches; egui unsticks it the moment a
                        // hand scrolls up and re-sticks when it returns.
                        if let Some(stream) = &snap.live
                            && !stream.is_empty()
                        {
                            super::chat::live(ui, stream);
                        }
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
        self.echo = Some(super::app::Echo {
            text,
            landed: false,
            agent: snap.focus.agent.clone(),
            at: (snap.landed, snap.refused),
        });
    }

    /// **What became of the echo**, run once per frame before the transcript
    /// is painted: the engine's receipt inks it, a refusal gives the text
    /// back to the composer (the banner already carries the engine's own
    /// sentence), and the message appearing in a transcript read dissolves it
    /// into the row it became (`crate::outbox`).
    pub(super) fn settle_echo(&mut self, snap: &Snapshot) {
        let Some(echo) = &mut self.echo else { return };
        if echo.agent != snap.focus.agent {
            self.echo = None;
            return;
        }
        if snap.refused > echo.at.1 {
            self.composer = std::mem::take(&mut echo.text);
            self.echo = None;
            return;
        }
        echo.landed |= snap.landed > echo.at.0;
        if crate::outbox::taken(&snap.transcript, &echo.text) {
            self.echo = None;
        }
    }
}
