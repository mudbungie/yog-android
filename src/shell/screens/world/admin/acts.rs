//! **The admin screen's four acts** (DESIGN §13.17): the foot of the reads its
//! parent paints, split from them (bl-f645) on the seam every paint file in
//! this app is cut along — what the surface DOES with a tap, against what it
//! puts on the glass.
//!
//! **The unmaking is last and alone.** `delete-workspace` gets a band of its
//! own above the other three, which is the fleet screen's ORDER (§13.13) at
//! the one site where the act really is an unmaking — and it is an
//! ENABLEMENT rather than an arming, because the wire's own `typed` is what
//! the engine checks (lernie DESIGN §4.20): the parameter is missing, not the
//! subject, so the control stays on the glass saying what would fill it.
//!
//! **One field, four acts, and the LABEL says which word it wants** (§13.13's
//! rule): a config file, a branch name, or this workspace's own name. `scan`
//! takes none and is never dark.

use eframe::egui;

use crate::codec::AdminAct;
use crate::shell::app::Shell;
use crate::shell::mark::TOUCH;

impl Shell {
    /// The foot: the unmaking, the three ordinary acts, then the field they
    /// compose in.
    pub(super) fn admin_acts(&mut self, ui: &mut egui::Ui, workspace: &str) {
        let band = egui::vec2(ui.available_width(), TOUCH);
        ui.allocate_ui_with_layout(
            band,
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                self.admin_item(ui, self.written());
                self.admin_item(
                    ui,
                    Some(AdminAct::Marks {
                        workspace: workspace.to_owned(),
                        branch: String::new(),
                    }),
                );
                self.admin_item(
                    ui,
                    Some(AdminAct::Scan {
                        workspace: workspace.to_owned(),
                    }),
                );
            },
        );
        self.unmaking(ui, workspace);
        self.minting(ui);
        self.editor(ui);
    }

    /// The config write, or nothing at all while no destination is picked: the
    /// act has no SUBJECT then, which is a different absence from a missing
    /// parameter and says so in its own words.
    fn written(&self) -> Option<AdminAct> {
        self.destination.clone().map(|at| AdminAct::Config {
            at,
            text: String::new(),
        })
    }

    /// One control. The label is the wire's own op token and so is the `act:`
    /// tag; while the act is dark the label says what would light it.
    fn admin_item(&mut self, ui: &mut egui::Ui, act: Option<AdminAct>) {
        let Some(act) = act else {
            let dark = ui.add_enabled(
                false,
                egui::Button::new("config — tap a file").min_size(egui::vec2(0.0, TOUCH)),
            );
            crate::shell::act::act(ui, &dark, "config");
            return;
        };
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
        if control.clicked() {
            self.fire(act);
        }
    }

    /// **The mint, one control per grade** (REMOTE §8.4, DESIGN §13.18). Two
    /// controls rather than a toggle beside one, because the grade is not a
    /// setting an operator adjusts — it is which of two devices is being made,
    /// and REMOTE §4.2 says the two see different worlds. Each takes the name
    /// out of the field and says so while it is dark.
    fn minting(&mut self, ui: &mut egui::Ui) {
        let typed = self.composer.trim().to_owned();
        let band = egui::vec2(ui.available_width(), TOUCH);
        ui.allocate_ui_with_layout(
            band,
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                for grade in [crate::leaf::Grade::Operator, crate::leaf::Grade::Foot] {
                    let word = crate::codec::enroll::word(grade);
                    let label = if typed.is_empty() {
                        format!("enroll {word} — name the device")
                    } else {
                        format!("enroll {word}")
                    };
                    let control = ui.add_enabled(
                        !typed.is_empty(),
                        egui::Button::new(label).min_size(egui::vec2(0.0, TOUCH)),
                    );
                    crate::shell::act::act(ui, &control, "enroll");
                    if control.clicked()
                        && let Some(model) = self.model()
                    {
                        model.enroll(typed.clone(), grade);
                    }
                }
            },
        );
    }

    /// The unmaking, on a band of its own, above the three that are not one.
    fn unmaking(&mut self, ui: &mut egui::Ui, workspace: &str) {
        let named = self.composer.trim() == workspace && !workspace.is_empty();
        let act = AdminAct::DeleteWorkspace {
            workspace: workspace.to_owned(),
            typed: workspace.to_owned(),
        };
        let label = match act.wants() {
            Some(ask) if !named => format!("{} — {ask}", act.op()),
            _ => act.op().to_owned(),
        };
        let band = egui::vec2(ui.available_width(), TOUCH);
        ui.allocate_ui_with_layout(
            band,
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                let control = ui.add_enabled(
                    named,
                    egui::Button::new(label).min_size(egui::vec2(0.0, TOUCH)),
                );
                crate::shell::act::act(ui, &control, act.op());
                if control.clicked() {
                    self.fire(act);
                }
            },
        );
    }

    /// Fire one act, spending the field where the act reads it. A config write
    /// KEEPS the draft: the engine may refuse it, and clearing the editor
    /// would charge a retype for the engine's no (lernie §4.20's toll).
    fn fire(&mut self, act: AdminAct) {
        let text = self.composer.clone();
        let act = match act {
            AdminAct::Config { at, .. } => AdminAct::Config { at, text },
            AdminAct::Marks { workspace, .. } => AdminAct::Marks {
                workspace,
                branch: text.trim().to_owned(),
            },
            other => other,
        };
        if let Some(model) = self.model() {
            model.admin(act);
        }
    }

    /// The field every act here composes in — multi-line, because what it
    /// mostly holds is a config file. It shares the composer's widget id for
    /// the starter's reason exactly (§13.2).
    fn editor(&mut self, ui: &mut egui::Ui) {
        ui.add(
            egui::TextEdit::multiline(&mut self.composer)
                .id(egui::Id::new(crate::shell::app::COMPOSER.id))
                .desired_width(f32::INFINITY)
                .desired_rows(3)
                .margin(crate::shell::composer::padding(ui))
                .hint_text("the file, a branch name, or this workspace's name"),
        );
    }
}
