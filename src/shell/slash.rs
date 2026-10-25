//! **The verb sheet** (DESIGN §22 item 5, bl-0d15): the rows `crate::verbs`
//! offers, painted in a framed box above the composer — never over the
//! platform field — and what taking one does, performed. Which rows, how a
//! line is read and what a tap or a send yields are `crate::verbs`, pure and
//! host-tested; this file is the paint and the hands.
//!
//! **Open exactly while the draft begins with `/`**, so nothing here holds a
//! flag: the field's text is the one home. The platform's back with the sheet
//! open takes the `/` off and leaves the rest (the door, run backwards).
//!
//! **A row laid in the sheet's scroll records its tags** (`shell::act`, the
//! box rule): the sheet floats in the overlay's frame, so a verb scrolled out
//! of view is reachable, not lost.

use eframe::egui;

use super::app::Shell;
use super::theme::rgb;
use crate::codec::RowAct;
use crate::seat::Snapshot;
use crate::verbs::{Does, Fired, Gates, Offers, Row};

impl Shell {
    /// What the sheet is gated on, read off the focus and the focused row.
    pub(super) fn gates(snap: &Snapshot) -> Gates {
        let row = super::controls::focused_row(snap);
        let row = row.as_ref();
        Gates {
            workspace: snap.focus.workspace.is_some(),
            conversation: snap.focus.agent.as_ref().map(|_| Offers {
                stoppable: row.is_some_and(|row| row.stoppable),
                stop_children: row.is_some_and(|row| row.stop_children),
                at_rest: row.is_some_and(|row| row.flight.is_none()),
            }),
        }
    }

    /// **The sheet, in whatever this `ui` has left**, while the draft is a
    /// slash line. Answers whether it painted, so the screen under it stands
    /// down.
    pub(super) fn sheet(&mut self, ui: &mut egui::Ui, snap: &Snapshot) -> bool {
        if !crate::verbs::slashing(&self.composer) {
            self.verb_said = None;
            return false;
        }
        if std::mem::take(&mut self.back) {
            self.composer = crate::verbs::door(&self.composer);
        }
        let all = crate::verbs::rows(&Self::gates(snap));
        let shown = crate::verbs::matching(&all, &self.composer);
        let rect = ui.available_rect_before_wrap();
        let mut took = None;
        // A refusal is said on the rule's banner line, under the sheet (§22
        // item 2), not at the sheet's head.
        super::overlay::framed(ui, rect, "verbs", |ui| {
            egui::ScrollArea::vertical()
                .min_scrolled_height(0.0)
                .auto_shrink(false)
                .show(ui, |ui| {
                    for row in &shown {
                        if self.verb_row(ui, row) {
                            took = Some(row.clone());
                        }
                    }
                });
        });
        if let Some(row) = took {
            let fired = crate::verbs::tap(&row, &all, &self.composer);
            self.fired(snap, fired);
        }
        true
    }

    /// One row: `/word` in ink, its summary in weak ink, the touch floor tall.
    fn verb_row(&mut self, ui: &mut egui::Ui, row: &Row) -> bool {
        let label = super::theme::line(
            ui,
            &[
                (
                    format!("{}{}", crate::verbs::LEAD, row.word),
                    rgb(crate::theme::INK),
                ),
                (format!("  {}", row.summary), rgb(crate::theme::INK_WEAK)),
            ],
        );
        // **What the row does, told** (§23 item 10): the word and its own
        // summary where the server gave one.
        let help = if row.summary.is_empty() {
            format!("/{} — opens it, or fires it", row.word)
        } else {
            format!("/{} — {}", row.word, row.summary)
        };
        let control = super::theme::row(ui, label.into(), &help);
        super::act::acts(ui, &control, row.ops);
        // Where the walk taps it (§15.2): by its word. A two-word row is not
        // named — a space would split the probe line it is said on.
        if !row.word.contains(' ') {
            self.note_control(row.word, ui, control.rect);
        }
        control.clicked()
    }

    /// `send` on a slash line.
    pub(super) fn send_line(&mut self, snap: &Snapshot) {
        let all = crate::verbs::rows(&Self::gates(snap));
        let fired = crate::verbs::send(&all, &self.composer);
        self.fired(snap, fired);
    }

    /// What a tap or a send yielded, performed.
    fn fired(&mut self, snap: &Snapshot, fired: Fired) {
        match fired {
            Fired::Fire(does, words) => {
                self.composer = String::new();
                self.field.took(&self.android);
                self.verb_said = None;
                self.does(snap, does, words);
            }
            Fired::Complete(line) => self.composer = line,
            Fired::Refused(why) => self.verb_said = Some(why),
        }
    }

    /// The act itself. The conversation's acts address the FOCUSED row — the
    /// row menu stays for acting on any other (§13.5).
    fn does(&mut self, snap: &Snapshot, does: Does, words: String) {
        let act = match does {
            Does::Open(overlay) => return self.open(overlay),
            // The drawer's two verbs (§24 items 2 and 4): `/conversations`
            // opens it, `/workspace` opens it with the head's list open.
            Does::Drawer => return self.drawer.open = true,
            Does::Knob(crate::status::Slot::Workspace) => {
                self.drawer.open = true;
                return self.drawer.list = true;
            }
            Does::Knob(crate::status::Slot::Priority) => return self.priority_verb(snap),
            Does::Knob(slot) => return self.rule.verb = Some(slot),
            // The hits float over the chat wherever the focus is (§24 item
            // 2): a search drops no focus.
            Does::Find => {
                self.needle.clone_from(&words);
                return self.ask(|model| model.search(words));
            }
            Does::Stop { children } => return self.ask(|model| model.stop_turn(children)),
            Does::Nudge => return self.ask(crate::seat::Model::nudge),
            Does::Interrupt => RowAct::Interrupt { content: words },
            Does::Retarget => RowAct::Retarget,
            Does::Flag => RowAct::Flag { reason: words },
            Does::Revoke => RowAct::Revoke,
            Does::Restore => RowAct::Restore,
        };
        let agent = snap.focus.agent.clone().unwrap_or_default();
        self.ask(|model| model.row_act(agent, act));
    }

    /// Hand the model one gesture, where there is a model.
    fn ask(&self, gesture: impl FnOnce(&crate::seat::Model)) {
        if let Some(model) = self.model() {
            gesture(model);
        }
    }
}
