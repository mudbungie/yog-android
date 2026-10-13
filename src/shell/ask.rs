//! **The ask** (DESIGN §22 item 8, bl-289f; §13.7's held band, re-shaped):
//! while the focused conversation is in the queue, a box framed and named in
//! the attention accent stands between the transcript and the rule, carrying
//! the queue row's words and its choices as numbered cells two to a row.
//! Which cells, their words and what each fires are `crate::ask`, pure and
//! host-tested; this file paints the box and spends a tap.
//!
//! **A tap on a cell is the answer** — the verdict and its reach in one, so
//! the scope chooser, its reset-on-call-change state and the `why` chip's
//! band are gone. The acts are §13.7's own (`seat::acts::held`,
//! `seat::acts::seen`), unchanged on the wire.
//!
//! **What it says of the parked call is the engine's sentence, folded**
//! (bl-8c94): `held: <tool> · <class>` on one line, the decision behind the
//! `why` fold on the box's first line, and the call's input never here — it
//! is a field of the transcript row that carries the call (`rows::held`).
//!
//! **Laid bottom-up**, because the block it is added to is (bl-193c): the
//! last row of cells first and the name line last, so nothing here is handed
//! the whole screen above the floor to paint from its top.

use eframe::egui;

use super::app::Shell;
use super::theme::{rgb, share};
use crate::ask::{ACROSS, Choice, HEADING, WHY};
use crate::seat::Snapshot;
use crate::theme::{self, State, glyph, space};

impl Shell {
    /// The box, or nothing at all.
    pub(super) fn asked(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        let (Some(workspace), Some(agent)) =
            (snap.focus.workspace.as_deref(), snap.focus.agent.as_deref())
        else {
            return;
        };
        let Some(row) = crate::ask::asking(&snap.queue, workspace, agent) else {
            return;
        };
        let accent = rgb(theme::accent(State::Attention));
        let parked = crate::ask::parked(&row);
        let mut taken = None;
        let mut flip = false;
        egui::Frame::NONE
            .fill(rgb(theme::SURFACE))
            .stroke(egui::Stroke::new(1.0, accent))
            .corner_radius(theme::RADIUS)
            .inner_margin(egui::Margin::symmetric(space::S as i8, space::XS as i8))
            .show(ui, |ui| {
                for cells in crate::ask::cells(&row).iter().rev() {
                    super::controls::band(ui, |ui| {
                        let wide = share(ui, ACROSS);
                        for (word, choice) in cells {
                            let control = cell(ui, word, wide);
                            super::act::act(ui, &control, &choice.op());
                            if control.clicked() {
                                taken = Some(choice.clone());
                            }
                        }
                    });
                }
                for (said, ink) in crate::ask::words(&row).iter().rev() {
                    let text = egui::RichText::new(said).color(rgb(*ink));
                    ui.add(egui::Label::new(text).truncate());
                }
                if let Some(folded) = &parked {
                    if self.why {
                        ui.add(egui::Label::new(&folded.why).wrap());
                    }
                    let line = egui::RichText::new(&folded.line).color(accent);
                    ui.add(egui::Label::new(line).truncate());
                }
                super::controls::band(ui, |ui| {
                    ui.label(egui::RichText::new(HEADING).color(accent));
                    if parked.is_some() {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            flip = self.fold(ui);
                        });
                    }
                });
            });
        if flip {
            self.why = !self.why;
        }
        let (Some(choice), Some(model)) = (taken, self.model()) else {
            return;
        };
        match choice {
            Choice::Seen => model.seen(row.workspace, row.agent),
            Choice::Answer(verdict, scope) => model.answer(verdict, scope),
        }
    }

    /// **The `why` fold** on the name line: its word and the fold glyph, a
    /// touch target, named `ask-why` for the walk. Answers whether it was
    /// tapped.
    fn fold(&mut self, ui: &mut egui::Ui) -> bool {
        let mark = if self.why { glyph::OPEN } else { glyph::SHUT };
        let ink = ui.visuals().text_color();
        let label = super::theme::marked(ui, WHY, mark, ink);
        let control = super::theme::chip(ui, label.into(), theme::TOUCH * 2.0, true);
        self.note_control("ask-why", ui, control.rect);
        control.clicked()
    }
}

/// **One cell**: half the row, a touch target tall, its words wrapped onto a
/// second line rather than elided — `pass for this conversation` is the
/// desktop's spelling byte for byte, and half a phone's width cannot hold it
/// on one line.
fn cell(ui: &mut egui::Ui, word: &str, wide: f32) -> egui::Response {
    ui.spacing_mut().button_padding = egui::vec2(space::S, space::XS);
    let room = (wide - space::S * 2.0).max(0.0);
    let label: egui::WidgetText = egui::WidgetText::from(word)
        .into_galley(
            ui,
            Some(egui::TextWrapMode::Wrap),
            room,
            egui::TextStyle::Button,
        )
        .into();
    ui.add(egui::Button::new(label).min_size(egui::vec2(wide, theme::TOUCH)))
}
