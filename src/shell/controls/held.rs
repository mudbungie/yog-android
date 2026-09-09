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
//! **What it says is the engine's own sentence, folded** (bl-8c94).
//! `held.reason` is the control's own words about the call, and REMOTE §8.1 is
//! explicit that rewriting it *"would put a different call in front of the
//! operator"* — so nothing here rewrites a word of it. What it does is SPLIT
//! it at the engine's own clause (`codec::queue::folded`): the line under the
//! composer says what is held and which class the control put it in, and the
//! decision paragraph opens behind a `why` fold.
//!
//! **The call's input is not in this band at all.** The sentence opens with a
//! clip of the tool's JSON input, and printing it here put ten lines of
//! `{"execution":"parallel","invocations":[…` between the message field and
//! the answers — the composer's floor is where an operator types, not where a
//! call is read. The input is a field of the transcript row that carries the
//! call (`rows::held`), one screen-length up and already on the glass, so
//! nothing is hidden by leaving it out of here.
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

/// The fold control's word, which it wears beside the transcript's own fold
/// glyph (`shell::chat`): `why` and not a triangle alone, because what it
/// opens is the control's reasoning and a band under a composer has room to
/// say so — and the triangle, because a fold's state is told by the same mark
/// everywhere on this glass.
const WHY: &str = "why";
/// What the fold control takes of the line's band. Two touch targets: enough
/// for the word and its mark, and no more — the LINE is what should have the
/// rest of the width, which is why this is a fixed seat rather than an equal
/// share. Derived from the touch scale, because no screen holds a size of its
/// own (STYLE.md §3).
const WIDE: f32 = crate::theme::TOUCH * 2.0;
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
        let folded = crate::codec::folded(&held.tool, &held.reason);
        // **Nothing here asks for the remaining rect** (bl-193c). The block
        // this is added to lays out bottom-up, so the answers are added
        // first and the sentence above them takes its own natural height —
        // where a `with_layout` would have been handed the whole screen above
        // the floor and painted from ITS top, over the bar (measured on the
        // emulator, first cut of this band).
        // **One call, one band's worth of state.** A reach and a fold chosen
        // for the call that was parked say nothing about the one parked now,
        // so both are dropped by the same comparison — the `tool_use` the
        // queue read carries, which is spent here and sent nowhere
        // (`codec::hold`).
        if self.picked_for.as_deref() != Some(held.tool_use.as_str()) {
            self.picked_for = Some(held.tool_use.clone());
            self.scope = Scope::default();
            self.why = false;
        }
        band(ui, |ui| {
            let wide = crate::shell::theme::share(ui, Verdict::ALL.len());
            for verdict in Verdict::ALL {
                let label =
                    crate::shell::theme::line(ui, &[(verdict.word(), ui.visuals().text_color())]);
                let control = crate::shell::theme::chip(ui, label.into(), wide, true);
                crate::shell::act::act(ui, &control, "answer");
                if control.clicked()
                    && let Some(model) = self.model()
                {
                    model.answer(verdict, self.scope.clone());
                }
            }
        });
        self.reach(ui);
        // Added after the line below it in a bottom-up block, so it paints
        // ABOVE the line — the fold opens upward, away from the answers, and
        // the line stays where the thumb last read it.
        if self.why {
            ui.add(egui::Label::new(&folded.why).wrap());
        }
        self.sentence(ui, &folded.line);
    }

    /// **The one line, and the fold that opens the rest.** The line is the
    /// attention accent — STYLE.md §3's *"a parked call held for an answer"* —
    /// beside a control whose word is `why`, because what it opens is the
    /// control's reasoning and not a nameless triangle. The chip is the band's
    /// own vocabulary, so the fold is a touch target rather than a glyph.
    fn sentence(&mut self, ui: &mut egui::Ui, line: &str) {
        band(ui, |ui| {
            // The line takes the band less the fold's seat, allocated rather
            // than left to a wrapping label: a label handed the whole band
            // pushes the control it shares the row with off the glass, which
            // is bl-e86c's finding at a new site.
            let wide = WIDE.min(ui.available_width());
            let rest = (ui.available_width() - wide - ui.spacing().item_spacing.x).max(0.0);
            let ink = crate::shell::theme::tone(&crate::codec::Tone::Held);
            let label = crate::shell::theme::line(ui, &[(line.to_owned(), ink)]);
            ui.allocate_ui_with_layout(
                egui::vec2(rest, super::super::mark::TOUCH),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.add(egui::Label::new(label).truncate());
                },
            );
            let mark = if self.why {
                crate::shell::chat::GLYPH_EXPANDED
            } else {
                crate::shell::chat::GLYPH_COLLAPSED
            };
            let label = crate::shell::theme::marked(ui, WHY, mark, ui.visuals().text_color());
            let control = crate::shell::theme::chip(ui, label.into(), wide, true);
            crate::shell::act::act(ui, &control, "answer");
            if control.clicked() {
                self.why = !self.why;
            }
        });
    }

    /// **How far the next answer stands.** One chip per scope, narrowest
    /// first, the picked one told by the brand — the one ink that says *the
    /// operator's own act* (STYLE.md §3) — exactly as the admin screen tells
    /// its picked destination. A band of chips and not a drop-down: three
    /// short words fit, and a chooser whose current value is behind a tap is a
    /// value nobody reads before deciding.
    ///
    /// `call` is what a band that has just appeared holds, because the scope
    /// is reset whenever the parked call changes — by [`Shell::capability`],
    /// which owns that comparison for the fold beside it too.
    fn reach(&mut self, ui: &mut egui::Ui) {
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
