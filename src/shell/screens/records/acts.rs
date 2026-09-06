//! **The records screen's two acts** (DESIGN §13.11, §13.16): read one step's
//! records, and fork a child at a picked point.
//!
//! **Two picks, two lists, and each control says which it takes.** `step`
//! addresses a census row; `fork` addresses a fork point — a notch of the
//! spine or a lineage head. They are different lists, so neither pick moves
//! the other, and the disambiguation is the fleet screen's (§13.13): a control
//! that is dark states in its own label what would light it.
//!
//! **One of the three IS armed, and it is the only one whose product is that
//! its subject is gone** (§13.8's test). `delete-agent` takes two taps on one
//! label — `clear-trail`'s idiom — and NOT the typed-name enablement the
//! admin screen's `delete-workspace` takes, because the wire's own grammar
//! says which is which (§13.17): a workspace deletion is refused unless the
//! typed name matches, so the box IS the arming there; a conversation
//! deletion reads an empty box as *this one alone* and its name as *and its
//! descendants*, so both values are gestures somebody meant and the box can
//! arm nothing. The arming has to come from the glass, and two taps is what
//! this app already has.
//!
//! The other two are not armings' business: a fork's product is a child, and
//! a drill-in reads.

use eframe::egui;

use crate::shell::app::Shell;
use crate::shell::mark::TOUCH;

/// What a fork asks for in the composer, said the way every other
/// text-taking act's hint is.
const GOAL: &str = "say what the child should do";

impl Shell {
    /// The foot: the two controls, the field one of them takes, and the
    /// sentences saying what is missing.
    pub(super) fn records_acts(&mut self, ui: &mut egui::Ui, workspace: &str, agent: &str) {
        self.records_band(ui);
        self.delete_band(ui, workspace, agent);
        self.goal(ui);
        if self.step.is_none() {
            ui.weak("tap a step to read what it recorded");
        }
    }

    /// **The unmaking, on a band of its own and above the two that are not
    /// one** — the fleet screen's ORDER (§13.13) at the site where the act
    /// really is an unmaking.
    ///
    /// The label says which of the two gestures the field has composed, since
    /// nothing else on the glass can: an empty box deletes this conversation
    /// and its own name typed back takes the descendants with it.
    fn delete_band(&mut self, ui: &mut egui::Ui, workspace: &str, agent: &str) {
        let typed = self.composer.trim().to_owned();
        let op = "delete-agent";
        let label = if self.armed {
            format!("{op} — tap again")
        } else if typed.is_empty() {
            format!("{op} · this conversation")
        } else {
            format!("{op} · and its descendants")
        };
        let band = egui::vec2(ui.available_width(), TOUCH);
        ui.allocate_ui_with_layout(
            band,
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                let control = ui.add(egui::Button::new(label).min_size(egui::vec2(0.0, TOUCH)));
                crate::shell::act::act(ui, &control, op);
                if !control.clicked() {
                    return;
                }
                if !std::mem::replace(&mut self.armed, true) {
                    return;
                }
                self.armed = false;
                let act = crate::codec::AdminAct::DeleteAgent {
                    workspace: workspace.to_owned(),
                    agent: agent.to_owned(),
                    typed: std::mem::take(&mut self.composer),
                };
                if let Some(model) = self.model() {
                    model.admin(act);
                }
            },
        );
    }

    /// The two controls, in one plain row given its own band — bl-f36e's
    /// finding: a row inside a bottom-up layout is placed against a height
    /// egui guessed before it laid the row out, so a row of touch-floor
    /// controls hangs into the gesture-nav zone unless it states its band.
    fn records_band(&mut self, ui: &mut egui::Ui) {
        let band = egui::vec2(ui.available_width(), TOUCH);
        ui.allocate_ui_with_layout(
            band,
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                self.drill_item(ui);
                self.fork_item(ui);
            },
        );
    }

    /// The drill-in. Its subject is a picked census row.
    fn drill_item(&mut self, ui: &mut egui::Ui) {
        let picked = self.step.clone();
        let control = ui.add_enabled(
            picked.is_some(),
            egui::Button::new("step").min_size(egui::vec2(0.0, TOUCH)),
        );
        // The tag rides the control that fires the op, disabled or not: what
        // it records is that the control was laid out and its rectangle was
        // on the glass (PARITY §4).
        crate::shell::act::act(ui, &control, "step");
        if control.clicked()
            && let (Some(seq), Some(model)) = (picked, self.model())
        {
            model.drill_step(seq);
        }
    }

    /// The fork. Its subject is a picked POINT, and it takes a goal.
    fn fork_item(&mut self, ui: &mut egui::Ui) {
        let picked = self.from.clone();
        let typed = !self.composer.trim().is_empty();
        let label = match (&picked, typed) {
            (None, _) => "fork — tap a notch or a lineage".to_owned(),
            (Some(_), false) => format!("fork — {GOAL}"),
            (Some(_), true) => "fork".to_owned(),
        };
        let control = ui.add_enabled(
            picked.is_some() && typed,
            egui::Button::new(label).min_size(egui::vec2(0.0, TOUCH)),
        );
        crate::shell::act::act(ui, &control, "fork");
        if !control.clicked() {
            return;
        }
        let Some(from) = picked else { return };
        let goal = std::mem::take(&mut self.composer);
        if let Some(model) = self.model() {
            model.fork(from, goal);
        }
    }

    /// The field the fork composes in. It shares the composer's widget id and
    /// its text for the starter's reason exactly (§13.2): only one of them is
    /// ever on screen, and the IME bridge addresses one field by that id.
    fn goal(&mut self, ui: &mut egui::Ui) {
        ui.add(
            egui::TextEdit::singleline(&mut self.composer)
                .id(egui::Id::new(crate::shell::app::COMPOSER.id))
                .desired_width(f32::INFINITY)
                .margin(crate::shell::composer::padding(ui))
                .hint_text(GOAL),
        );
    }
}
