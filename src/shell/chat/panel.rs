//! **The conversation panel** (DESIGN §22 item 4; STYLE §5, *the panel at the
//! head*): a `SURFACE` card with a `HAIRLINE` frame and `RADIUS` corners at
//! the head of a transcript, one fact a line. Which facts, in which ink, and
//! which fold behind `+` are `crate::panel`'s, pure and tested; this paints
//! them. A long fact is a touch-tall line elided behind `+`, and a tap
//! unfolds it in place; which are open is the toolkit's memory, a viewport
//! state that dies with the screen.

use eframe::egui;

use super::tap::{self, Run};
use crate::panel::Fact;
use crate::shell::theme::rgb;
use crate::theme::{self, glyph, space};

/// What a fact's fold does, told on a long press (§23 item 10).
const FOLD_HELP: &str = "unfolds this fact whole, or folds it back to one line";

/// Paint the card.
pub(in crate::shell) fn card(ui: &mut egui::Ui, facts: &[Fact]) {
    egui::Frame::NONE
        .fill(rgb(theme::SURFACE))
        .stroke(egui::Stroke::new(1.0, rgb(theme::HAIRLINE)))
        .corner_radius(egui::CornerRadius::same(theme::RADIUS))
        .inner_margin(egui::Margin::symmetric(space::M as i8, space::S as i8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            for (at, said) in facts.iter().enumerate() {
                fact(ui, at, said);
            }
        });
    ui.add_space(space::M);
}

/// One fact: a line, or — long — a fold.
fn fact(ui: &mut egui::Ui, at: usize, fact: &Fact) {
    let ink = rgb(fact.ink);
    if !fact.long {
        ui.add(egui::Label::new(egui::RichText::new(&fact.text).color(ink)).wrap());
        return;
    }
    let id = ui.make_persistent_id(("panel", at));
    let open = ui.data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
    let tapped = if open {
        ui.horizontal_top(|ui| {
            tap::glyph(ui, glyph::OPEN, rgb(theme::INK_WEAK));
            let whole = ui.add(
                egui::Label::new(egui::RichText::new(&fact.text).color(ink))
                    .wrap()
                    .sense(egui::Sense::click()),
            );
            crate::shell::theme::told::told(ui, &whole, FOLD_HELP);
            whole.clicked()
        })
        .inner
    } else {
        let runs = [Run::body(vec![(fact.text.clone(), ink)])];
        tap::line(
            ui,
            0.0,
            (glyph::SHUT, rgb(theme::INK_WEAK)),
            &runs,
            theme::TOUCH,
            FOLD_HELP,
        )
        .clicked()
    };
    if tapped {
        ui.data_mut(|d| d.insert_temp(id, !open));
    }
}
