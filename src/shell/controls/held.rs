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
//!
//! **And a scope above them** (PROTOCOL 18, yog bl-94a5). An answer settles
//! the held `call` by default; `conversation` and `workspace` settle that
//! call's CLASS for the conversation and its descent, or for every
//! conversation here. It is a chooser and not a fourth verdict, because it is
//! orthogonal to all three — a wide `refuse` is as real a decision as a wide
//! `pass`, and an operator holding a conversation answers the same question
//! for every call of a kind they already decided about.
//!
//! **The chooser is above the answers and the default is the narrow one.**
//! Reach is read before the verdict is tapped, which is the same ordering the
//! sentence above them already has; and the scope resets to `call` whenever
//! the parked call changes, because a width carried from one call to the next
//! would widen an answer nobody widened. Nothing here decides whether a wide
//! answer is allowed: a destructive or credential-reaching call takes the bare
//! scope only, and that refusal is the ENGINE's and arrives in band, which is
//! where every capability decision this seat does not own arrives.

use eframe::egui;

use super::band;
use crate::codec::{Scope, Verdict};
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
                    model.answer(verdict, self.scope.clone());
                }
            }
        });
        self.reach(ui, &held.tool_use);
        ui.weak(format!("held · {} · {}", held.tool, held.reason));
    }

    /// **How far the next answer stands.** One chip per scope, narrowest
    /// first, the picked one told by the brand — the one ink that says *the
    /// operator's own act* (STYLE.md §3) — exactly as the admin screen tells
    /// its picked destination. A band of chips and not a drop-down: three
    /// short words fit, and a chooser whose current value is behind a tap is a
    /// value nobody reads before deciding.
    ///
    /// `call` is what a band that has just appeared holds, because the scope
    /// is reset whenever the parked call changes: the chip carries the reach
    /// and the `tool_use` says which call it was chosen for, which is the same
    /// use the id already had here (`codec::hold`).
    fn reach(&mut self, ui: &mut egui::Ui, tool_use: &str) {
        if self.picked_for.as_deref() != Some(tool_use) {
            self.picked_for = Some(tool_use.to_owned());
            self.scope = Scope::default();
        }
        band(ui, |ui| {
            let wide = crate::shell::theme::share(ui, Scope::ALL.len());
            for scope in Scope::ALL {
                let ink = if self.scope == scope {
                    crate::shell::theme::rgb(crate::theme::BRAND)
                } else {
                    ui.visuals().text_color()
                };
                let label = crate::shell::theme::line(ui, &[(scope.word(), ink)]);
                let control = crate::shell::theme::chip(ui, label.into(), wide, true);
                crate::shell::act::act(ui, &control, "answer");
                if control.clicked() {
                    self.scope = scope;
                }
            }
        });
    }
}
