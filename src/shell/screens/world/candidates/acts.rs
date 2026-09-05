//! **The candidates screen's three acts** (DESIGN §13.12): the foot of the
//! listing its parent paints.
//!
//! **The row says which act it earns**, so nothing here is a mode: a row with
//! a handle is a candidate and takes `deliver` or `retire`; a row without one
//! is the claim and takes `fan`. Each control stands down when the picked row
//! is not its subject — §13.10's enablement rule with the SUBJECT missing
//! rather than the parameter.
//!
//! **Nothing here is armed** (lernie DESIGN §4.36, transferring §13.8's test):
//! an arming is for an act whose product is that its subject is gone.
//! `deliver` advances a ref by the ordinary recursive delivery — git holds
//! what it moved and the ball is not closed — and `retire` releases a worktree
//! and changes no delivery target. Whether the source ref goes with it is this
//! project's own declared retention, and the receipt says which way it went.
//!
//! **The text is the composer's** (§13.2's one shared row, §13.10's rule at a
//! third site). Two acts need words and they are different words — a fan's is
//! the instruction n conversations are given, a delivery's is what the
//! delivery IS — so the hint says which one the picked row is asking for.
//!
//! **The count is a stepper floored at two.** Upstream reads 1 and 0 as
//! *materialize nothing and hand back the ordinary claim binding*, which is a
//! start and this app already has one (§13.2's starter). So two is the
//! smallest thing this control can mean, and the floor is the control's rather
//! than the codec's — an encoder that refused a number it can spell would be
//! refusing a shape it understands.

use eframe::egui;

use crate::codec::CandidateAct;
use crate::shell::app::Shell;
use crate::shell::mark::TOUCH;

/// The sentence the foot states while no row is picked.
const PICK: &str = "tap an attempt to act on it";

/// What a fan asks for in the composer, said the way a handle act's own
/// `wants` is.
const GOAL: &str = "say what the candidates should do";

/// The smallest spread this control can mean.
pub(in crate::shell) const FLOOR: usize = 2;

impl Shell {
    /// The foot: the count, the three controls, and the field they compose in.
    pub(super) fn candidate_acts(&mut self, ui: &mut egui::Ui) {
        let claim = self
            .candidate
            .as_ref()
            .is_some_and(|(_, _, handle)| handle.is_empty());
        self.acts_band(ui);
        // **A second band, and only when there is something in it** — the
        // controls row's own rule (§13.2). The stepper is the fan's parameter
        // and the fan is offered on the claim alone, so a picked candidate
        // paints no count at all rather than a dead one.
        if claim {
            self.count_band(ui);
        }
        self.subject(ui, claim);
        if self.candidate.is_none() {
            ui.weak(PICK);
        }
    }

    /// The three controls, in one plain row given its own band — bl-f36e's
    /// finding, at a second site: a row inside a bottom-up layout is placed
    /// against a height egui guessed before it laid the row out, so a row of
    /// touch-floor controls hangs into the gesture-nav zone unless it states
    /// its own band first.
    fn acts_band(&mut self, ui: &mut egui::Ui) {
        let band = egui::vec2(ui.available_width(), TOUCH);
        ui.allocate_ui_with_layout(
            band,
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                self.fan_item(ui);
                for act in handles() {
                    self.candidate_item(ui, &act);
                }
            },
        );
    }

    /// The stepper. Its buttons carry the step rather than a bare glyph,
    /// because the label between them is the value and a reader must not have
    /// to guess which way each goes.
    fn count_band(&mut self, ui: &mut egui::Ui) {
        let band = egui::vec2(ui.available_width(), TOUCH);
        ui.allocate_ui_with_layout(
            band,
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                if ui
                    .add_enabled(self.spread > FLOOR, egui::Button::new("−1"))
                    .clicked()
                {
                    self.spread = self.spread.saturating_sub(1).max(FLOOR);
                }
                ui.label(format!("{} candidates", self.spread));
                if ui.button("+1").clicked() {
                    self.spread = self.spread.saturating_add(1);
                }
            },
        );
    }

    /// The fan control. Its subject is the CLAIM, so it is dark on a candidate
    /// row — and dark with no row at all, like everything else here.
    fn fan_item(&mut self, ui: &mut egui::Ui) {
        let picked = self.candidate.clone();
        let typed = !self.composer.trim().is_empty();
        let claim = picked
            .as_ref()
            .is_some_and(|(_, _, handle)| handle.is_empty());
        let label = if claim && !typed {
            format!("fan — {GOAL}")
        } else {
            "fan".to_owned()
        };
        let control = ui.add_enabled(
            claim && typed,
            egui::Button::new(label).min_size(egui::vec2(0.0, TOUCH)),
        );
        crate::shell::act::act(ui, &control, "fan");
        if !control.clicked() {
            return;
        }
        let (Some((project, ball, _)), Some(n)) = (picked, Some(self.spread)) else {
            return;
        };
        let goal = std::mem::take(&mut self.composer);
        if let Some(model) = self.model() {
            model.fan(project, ball, n, goal);
        }
    }

    /// One handle control. Its subject is a CANDIDATE, so it is dark on the
    /// claim's row.
    fn candidate_item(&mut self, ui: &mut egui::Ui, act: &CandidateAct) {
        let picked = self.candidate.clone();
        let typed = !self.composer.trim().is_empty();
        let handle = picked
            .as_ref()
            .map(|(_, _, handle)| handle.clone())
            .unwrap_or_default();
        let wants = act.wants();
        let live = !handle.is_empty() && wants.is_none_or(|_| typed);
        let label = match wants {
            Some(ask) if !handle.is_empty() && !typed => format!("{} — {ask}", act.op()),
            _ => act.op().to_owned(),
        };
        let control = ui.add_enabled(
            live,
            egui::Button::new(label).min_size(egui::vec2(0.0, TOUCH)),
        );
        crate::shell::act::act(ui, &control, act.op());
        if !control.clicked() {
            return;
        }
        let Some((project, ball, handle)) = picked else {
            return;
        };
        let text = if wants.is_some() {
            std::mem::take(&mut self.composer)
        } else {
            String::new()
        };
        if let Some(model) = self.model() {
            model.candidate_act(project, ball, act.on(handle, text));
        }
    }

    /// The field both text-taking acts compose in. It shares the composer's
    /// widget id and its text for the starter's reason exactly (§13.2): only
    /// one of them is ever on screen, and the IME bridge addresses one field
    /// by that id.
    fn subject(&mut self, ui: &mut egui::Ui, claim: bool) {
        let hint = if claim {
            GOAL
        } else {
            "say what this delivery is"
        };
        ui.add(
            egui::TextEdit::singleline(&mut self.composer)
                .id(egui::Id::new(crate::shell::app::COMPOSER.id))
                .desired_width(f32::INFINITY)
                .margin(crate::shell::composer::padding(ui))
                .hint_text(hint),
        );
    }
}

/// The two handle acts' empty forms, in the order a thumb meets them: accept
/// first, release under it — the unmaking's ORDER (lernie §4.33), which is
/// what these two get instead of an arming.
fn handles() -> [CandidateAct; 2] {
    [
        CandidateAct::Deliver {
            handle: String::new(),
            summary: String::new(),
        },
        CandidateAct::Retire {
            handle: String::new(),
        },
    ]
}
