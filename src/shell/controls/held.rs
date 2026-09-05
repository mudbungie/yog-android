//! **The capability band** (DESIGN §13.7, bl-b39d): the tool call the engine
//! has parked at this conversation, and the three answers to it.
//!
//! **It paints only where the engine says a call is held.** The fact is the
//! queue read's (`codec::queue`), never a reading taken here — a transcript
//! whose last tool call has no result is *either* a parked call or a driver
//! that died, and this app must not put an approval in front of an operator on
//! a guess. That is §8's rule at the one site where getting it wrong means
//! authorizing an action.
//!
//! **What it says is the engine's own sentence.** `held.reason` is the
//! control's own words about the call — what it is about to do — and REMOTE
//! §8.1 is explicit that rewriting it *"would put a different call in front of
//! the operator"*. So the band paints the tool's name and that sentence, and
//! nothing here summarizes either.
//!
//! **Three answers, and tap is the act** (§13.2). None of the three is armed:
//! `pass` runs one call the operator has just read, `refuse` declines it in
//! band, `hold` keeps it exactly where it is. The destructive one — the call
//! itself — is the thing being read before the tap, which is the whole reason
//! this band lives on the transcript screen and nowhere else.

use eframe::egui;

use super::band;
use crate::codec::Verdict;
use crate::seat::Snapshot;
use crate::shell::app::Shell;

impl Shell {
    /// The band, or nothing at all.
    pub(super) fn capability(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        let (Some(workspace), Some(agent)) =
            (snap.focus.workspace.as_deref(), snap.focus.agent.as_deref())
        else {
            return;
        };
        let Some(held) = crate::codec::queue::held_at(&snap.queue, workspace, agent) else {
            return;
        };
        // **Nothing here asks for the remaining rect** (bl-193c). The block
        // this is added to lays out bottom-up, so the answers are added
        // first and the sentence above them takes its own natural height —
        // where a `with_layout` would have been handed the whole screen above
        // the floor and painted from ITS top, over the bar (measured on the
        // emulator, first cut of this band).
        band(ui, |ui| {
            for verdict in Verdict::ALL {
                let control = ui.button(verdict.word());
                crate::shell::act::act(ui, &control, "answer");
                if control.clicked()
                    && let Some(model) = self.model()
                {
                    model.answer(verdict);
                }
            }
        });
        ui.weak(format!("held · {} · {}", held.tool, held.reason));
    }
}
