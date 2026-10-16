//! **The settings overlay** (DESIGN §23 item 11; lernie `ui::settings`):
//! the knob table painted, one `TOUCH`-tall row per knob over its sentence
//! in weak ink — a slider for a number, a check for a switch — and
//! `defaults` at the foot. A change applies at the next frame and is
//! written to the device the same frame; nothing here crosses the boundary.
//! Paints the table; decides nothing.

use eframe::egui;

use super::app::Shell;
use super::theme::rgb;
use crate::seat::Snapshot;
use crate::settings::{KNOBS, Kind, Knobbed, Settings};
use crate::theme::{self, space};

/// The screen's name, which is also the sheet row's word.
pub(in crate::shell) const SCREEN: &str = crate::overlay::Overlay::Settings.name();
/// The word at the foot that puts every knob back.
const DEFAULTS: &str = "defaults";

impl Shell {
    /// The pane: every knob, then the way back to the defaults.
    pub(in crate::shell) fn tuned(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        self.note_screen(SCREEN);
        super::screens::banner(ui, snap);
        ui.separator();
        let mut moved = false;
        egui::ScrollArea::vertical()
            .min_scrolled_height(0.0)
            .show(ui, |ui| {
                for knob in KNOBS {
                    moved |= row(ui, &mut self.knobs, knob);
                    ui.add_space(space::S);
                }
                let back = super::theme::row(
                    ui,
                    egui::RichText::new(DEFAULTS)
                        .color(rgb(theme::BRAND))
                        .into(),
                    "puts every knob back to its default",
                );
                self.note_control("defaults", ui, back.rect);
                if back.clicked() {
                    self.knobs = Settings::default();
                    moved = true;
                }
            });
        if moved && let Err(why) = crate::settings::write(&self.knobs_file, &self.knobs) {
            // The knob moved on the glass either way; logcat is the witness.
            log::warn!("settings: {why}");
        }
    }
}

/// **One knob**: its label and control on a touch-tall line, its sentence
/// under them. Answers whether it moved.
fn row(ui: &mut egui::Ui, settings: &mut Settings, knob: &crate::settings::Knob) -> bool {
    let held = (knob.get)(settings);
    let mut moved = false;
    ui.horizontal(|ui| {
        ui.set_min_height(theme::TOUCH);
        match (knob.kind, held) {
            (Kind::Toggle, Knobbed::On(was)) => {
                let mut on = was;
                super::theme::check(ui, &mut on, knob.label, knob.help);
                if on != was {
                    (knob.set)(settings, Knobbed::On(on));
                    moved = true;
                }
            }
            (
                Kind::Number {
                    unit,
                    floor,
                    ceiling,
                    step,
                },
                Knobbed::At(mut at),
            ) => {
                ui.add(egui::Label::new(knob.label).truncate());
                let slider =
                    super::theme::slider(ui, &mut at, (floor, ceiling, step), unit, knob.help);
                if slider.changed() {
                    (knob.set)(settings, Knobbed::At(at));
                    moved = true;
                }
            }
            // The table holds no knob of one kind at a value of the other
            // (`settings::tests`); nothing to paint for one.
            _ => {}
        }
    });
    ui.horizontal_wrapped(|ui| {
        ui.add_space(space::M);
        ui.add(egui::Label::new(egui::RichText::new(knob.help).color(rgb(theme::INK_WEAK))).wrap());
    });
    moved
}
