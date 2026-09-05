//! **The fleet screen** (DESIGN §13.13): the two armings a workspace carries —
//! the drone loop that runs its ready balls, and the alignment monitor that
//! reads what they commit.
//!
//! **It reads nothing, and that is one fact having one home.** Whether a loop
//! is armed, how full it is and what it holds are on the `board` answer, which
//! the ball pane already paints (§13.9) — so opening this screen asks the
//! engine nothing at all, and the screen says where the answer lives instead
//! of asking for a second copy of it.
//!
//! **It holds the workspace it was opened on**, which is what the whole app
//! does with an aimed screen: these acts start drones and spend money in ONE
//! workspace, so a gesture composed against whatever the focus happened to be
//! at the moment of the tap would be a control that moved under the operator.
//!
//! **Neither stopping act is armed, and that is an argument rather than an
//! omission** (lernie DESIGN §4.33, whose ruling transfers whole). §13.8's
//! arming is for the **unmaking**: an act whose product is that its subject is
//! gone. Neither of these is that — `disband` stops nothing that is running
//! and `disarm` leaves every verdict already on the trail — and each is undone
//! by doing the other thing. An arming on an act a tap reverses teaches an
//! operator to tap through armings, which is the one thing that would make the
//! unmaking's arming worthless. What they get instead is the unmaking's
//! ORDER: the acts that start things first, the acts that stop them under.
//!
//! **Two controls want DIFFERENT words out of one field, and the label is
//! what says which.** This app types text in one place (§13.2), and here a
//! project and a monitor's model are both wanted at once — there is no row to
//! disambiguate them, as there is on the candidates screen (§13.12). So each
//! control that needs a word states it in its own label while it is dark,
//! which is §13.10's enablement rule doing the disambiguating for free.

use eframe::egui;

use crate::codec::FleetAct;
use crate::seat::Snapshot;
use crate::shell::app::Shell;
use crate::shell::mark::{Back, TOUCH};

/// The screen's name and the harness's tap target (§15.2). It is the loop's
/// own op token, because that is the act an operator comes here for; the other
/// three are what you do to it afterwards.
pub(in crate::shell) const SCREEN: &str = "fleet";

/// The smallest cap this control can mean. A cap of zero is a loop that spawns
/// nothing and still reaps, which upstream refuses to spell as a cap at all.
pub(in crate::shell) const FLOOR: usize = 1;

/// Where what these acts DID is actually readable, said on the glass rather
/// than left for an operator to work out.
const ELSEWHERE: &str = "what a loop is doing is on the board";

impl Shell {
    pub(in crate::shell) fn fleet(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        self.note_screen(SCREEN);
        // The acts claim the floor first and the sentence takes what is left —
        // the trail's order (§13.8, bl-192c), for its reason.
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            self.fleet_acts(ui);
            ui.add_space(4.0);
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                if self.bar(ui, SCREEN, &Back::To("conversations")) {
                    self.close_world();
                }
                super::super::banner(ui, snap);
                ui.separator();
                ui.weak(ELSEWHERE);
            });
        });
    }

    /// The foot: the cap, the four controls, and the field two of them
    /// compose in.
    fn fleet_acts(&mut self, ui: &mut egui::Ui) {
        let band = egui::vec2(ui.available_width(), TOUCH);
        // The starting pair first, the stopping pair under them — the
        // unmaking's ORDER, which is what these four get instead of an arming.
        for row in [starters(), stoppers()] {
            ui.allocate_ui_with_layout(
                band,
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    for act in row {
                        self.fleet_item(ui, &act);
                    }
                },
            );
        }
        self.name(ui);
        self.cap_band(ui);
    }

    /// The cap stepper. Its buttons carry the step rather than a bare glyph,
    /// because the label between them is the value.
    fn cap_band(&mut self, ui: &mut egui::Ui) {
        let band = egui::vec2(ui.available_width(), TOUCH);
        ui.allocate_ui_with_layout(
            band,
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                if ui
                    .add_enabled(self.cap > FLOOR, egui::Button::new("−1"))
                    .clicked()
                {
                    self.cap = self.cap.saturating_sub(1).max(FLOOR);
                }
                ui.label(format!("{} at once", self.cap));
                if ui.button("+1").clicked() {
                    self.cap = self.cap.saturating_add(1);
                }
            },
        );
    }

    /// One control. The label is the wire's own op token and so is the `act:`
    /// tag it carries — one name, so the paint cannot show a word and post
    /// another — and while the act is dark the label says what would light it.
    fn fleet_item(&mut self, ui: &mut egui::Ui, act: &FleetAct) {
        let typed = !self.composer.trim().is_empty();
        let wants = act.wants();
        let live = wants.is_none_or(|_| typed);
        let label = match wants {
            Some(ask) if !typed => format!("{} — {ask}", act.op()),
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
        // Only an act that takes a name spends the field: a disband that
        // emptied it would eat a word it never read.
        let text = if wants.is_some() {
            std::mem::take(&mut self.composer)
        } else {
            String::new()
        };
        if let Some(model) = self.model() {
            model.fleet_act(act.with(text, self.cap));
        }
    }

    /// The field the two starting acts compose in. It shares the composer's
    /// widget id and its text for the starter's reason exactly (§13.2): only
    /// one of them is ever on screen, and the IME bridge addresses one field
    /// by that id. The hint names both words, because both controls above say
    /// which of them they take.
    fn name(&mut self, ui: &mut egui::Ui) {
        ui.add(
            egui::TextEdit::singleline(&mut self.composer)
                .id(egui::Id::new(crate::shell::app::COMPOSER.id))
                .desired_width(f32::INFINITY)
                .margin(crate::shell::composer::padding(ui))
                .hint_text("a project, or a monitor's model"),
        );
    }
}

/// The two that START something.
fn starters() -> [FleetAct; 2] {
    [
        FleetAct::Fleet {
            project: String::new(),
            cap: FLOOR,
        },
        FleetAct::Arm {
            model: String::new(),
        },
    ]
}

/// The two that stop it.
fn stoppers() -> [FleetAct; 2] {
    [FleetAct::Disband, FleetAct::Disarm]
}
