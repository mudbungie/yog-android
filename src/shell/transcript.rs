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
use super::theme::bubbled;
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
        // **The frame that paints a focused conversation is what opens its
        // tab** (§23 item 2), and a tab the listing forgot leaves here too.
        self.tabs.focused(workspace, agent);
        let known: Vec<String> = snap
            .conversations
            .iter()
            .map(|row| row.root_id.clone())
            .collect();
        self.tabs.prune(workspace, &known, Some(agent));
        // **The drawer** (§23 item 1): a drag from the left edge opens the
        // switcher, the one gesture the desktop's bar had that the switcher
        // lacked.
        let (origin, now) = ui.ctx().input(|i| {
            let down = i.pointer.any_down();
            (
                i.pointer.press_origin().filter(|_| down).map(|p| p.x),
                i.pointer.latest_pos().map(|p| p.x),
            )
        });
        if crate::drawer::opens(origin, now) {
            self.open(crate::overlay::Overlay::Switcher);
        }
        let speaker = super::chat::speaker_of(snap, agent);
        // **Which call is parked is the engine's answer, never a reading
        // taken here** (§13.7): the queue read names it, and the projection
        // marks the row it already committed rather than guessing from a
        // tool call with no result.
        let parked = crate::codec::queue::held_at(&snap.queue, workspace, agent);
        let painted = rows(
            &snap.transcript,
            &speaker,
            parked.as_ref(),
            self.knobs.auto(),
            &self.folds,
        );
        let lines = crate::rows::lines(&painted, &self.folds);
        let facts =
            crate::panel::conversation(&snap.conversations, agent, workspace, &snap.transcript);
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
            // Closest to the floor: the prompt line, then the status rule
            // above it, then the live dock and the ask above the rule (§22
            // items 1, 2, 9 and 8). **No bar** (§22 item 1): the rule carries the
            // mark and the name, and back is the platform's.
            if let Some(taken) = self.compose(ui, snap, "message", &["message"]) {
                self.deposit(snap, taken);
            }
            self.rule(ui, snap);
            // **The sheet takes the whole room above the rule** (§22 item
            // 5): the dock and the ask step aside while the draft is a slash
            // line, or the sheet is four rows over a six-cell box.
            if !crate::verbs::slashing(&self.composer) {
                self.dock(ui, snap);
                self.asked(ui, snap);
            }
            ui.add_space(4.0);
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                // **The tab strip stands over the transcript** (§23 item 2),
                // under the top inset, never riding the keyboard.
                self.tabs(ui, snap, workspace);
                // **The verb sheet takes this room while the draft is a slash
                // line** (§22 item 5): above the rule, never over the field.
                if self.sheet(ui, snap) {
                    return;
                }
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
                            // **The conversation's card at the head** (§22
                            // item 4), then the lines in the idiom.
                            super::chat::panel::card(ui, &facts);
                            // **Every block on its role's bubble** (§23
                            // item 7): the first line's parting stands
                            // outside the fill, the rest inside it.
                            let mut at = 0;
                            let bubbles = self.knobs.on(crate::settings::Toggle::Bubbles);
                            for block in crate::rows::blocks(&lines) {
                                let held = lines.get(at..at + block.lines).unwrap_or_default();
                                if let Some(first) = held.first() {
                                    super::chat::before(ui, first.before);
                                }
                                bubbled(ui, bubbles.then_some(block.bubble), |ui| {
                                    for (i, line) in held.iter().enumerate() {
                                        if i > 0 {
                                            super::chat::before(ui, line.before);
                                        }
                                        if super::chat::body_of(ui, line) {
                                            flipped = Some(line.row.key.clone());
                                        }
                                    }
                                });
                                at += block.lines;
                            }
                            // The message this seat has sent and the engine
                            // has not shown back yet (bl-66fb), where its row
                            // will be — above the answer to it, on the
                            // operator's own bubble (§23 item 5).
                            if let Some(echo) = &self.echo {
                                let own = bubbles.then_some(crate::theme::bubble::Bubble::Operator);
                                bubbled(ui, own, |ui| {
                                    super::chat::echo(ui, &echo.text, echo.fate);
                                });
                            }
                        });
                    });
            });
        });
        self.back_to_switcher();
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
