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
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.auto.responses, "talk");
            ui.checkbox(&mut self.auto.others, "steps");
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
            if let Some(taken) = super::chat::composer(ui, &mut self.composer, "message")
                && let Some(model) = self.model()
            {
                model.deposit(taken);
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
