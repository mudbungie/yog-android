//! **The ask** (DESIGN §22 item 8, bl-289f; §13.7's held band, re-shaped):
//! while the focused conversation is in the queue, a box framed and named in
//! the attention accent stands between the transcript and the box under it
//! (§23 item 4, the desktop's order): its name
//! line, ONE line of the row's words (the parked call, else the failure, else
//! the last words) with the rest behind the name line's fold (bl-58ca), and
//! its choices as numbered cells three to a row where three fit, else two.
//! Which cells, their words and what each fires are `crate::ask`, pure and
//! host-tested; this file paints the box and spends a tap.
//!
//! **A tap on a cell is the answer** — the verdict and its reach in one, so
//! the scope chooser, its reset-on-call-change state and the `why` chip's
//! band are gone. The acts are §13.7's own (`seat::acts::held`,
//! `seat::acts::seen`), unchanged on the wire.
//!
//! **What it says of the parked call is the server's sentence, folded**
//! (bl-8c94): `held: <tool> · <class>` on one line, the decision and the
//! row's other words behind the `why` fold on the box's first line, and the call's input never here — it
//! is a field of the transcript row that carries the call (`rows::held`).
//!
//! **Laid bottom-up**, because the block it is added to is (bl-193c): the
//! last row of cells first and the name line last, so nothing here is handed
//! the whole screen above the floor to paint from its top.

use eframe::egui;

use super::app::Shell;
use super::theme::{rgb, share};
use crate::ask::{Choice, HEADING, WHY};
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
                let across = crate::ask::across(ui.available_width(), ui.spacing().item_spacing.x);
                ui.scope(|ui| {
                    if across == 3 {
                        // Three to a row at `SMALL` (bl-58ca): every cell's
                        // words wrap onto two small lines, none elided.
                        ui.style_mut().text_styles.insert(
                            egui::TextStyle::Button,
                            egui::FontId::proportional(theme::type_scale::SMALL),
                        );
                    }
                    for cells in crate::ask::cells(&row, across).iter().rev() {
                        super::controls::band(ui, |ui| {
                            let wide = share(ui, across);
                            for (word, choice) in cells {
                                let control = cell(ui, word, wide, &choice.help());
                                super::act::act(ui, &control, &choice.op());
                                if control.clicked() {
                                    taken = Some(choice.clone());
                                }
                            }
                        });
                    }
                });
                if self.why {
                    for (said, ink) in crate::ask::behind(&row).iter().rev() {
                        line(ui, said, *ink);
                    }
                    if let Some(folded) = &parked {
                        ui.add(egui::Label::new(&folded.why).wrap());
                    }
                }
                // **The name line carries the summary** (bl-58ca): the name,
                // then the one line in its own ink, elided before the fold —
                // a line of its own under the name cost the transcript the
                // last of the half of the glass it is owed.
                super::controls::band(ui, |ui| {
                    ui.label(egui::RichText::new(HEADING).color(accent));
                    let fold = if crate::ask::folds(&row) {
                        theme::TOUCH * 2.0 + ui.spacing().item_spacing.x
                    } else {
                        0.0
                    };
                    if let Some((said, ink)) = crate::ask::summary(&row) {
                        let room = (ui.available_width() - fold).max(0.0);
                        ui.allocate_ui(egui::vec2(room, theme::TOUCH), |ui| {
                            line(ui, &format!("{} {said}", glyph::NOTE), ink);
                        });
                    }
                    if fold > 0.0 {
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
        let control = super::theme::chip(
            ui,
            label.into(),
            theme::TOUCH * 2.0,
            true,
            "opens the control's decision about the parked call, or folds it",
        );
        self.note_control("ask-why", ui, control.rect);
        control.clicked()
    }
}

/// One line of the row's words, in its ink, elided to the box.
fn line(ui: &mut egui::Ui, said: &str, ink: theme::Rgb) {
    ui.add(egui::Label::new(egui::RichText::new(said).color(rgb(ink))).truncate());
}

/// **One cell**: a half or a third of the row, a touch target tall, its words wrapped onto a
/// second line rather than elided — `pass for this conversation` is the
/// desktop's spelling byte for byte, and half a phone's width cannot hold it
/// on one line.
fn cell(ui: &mut egui::Ui, word: &str, wide: f32, help: &str) -> egui::Response {
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
    super::theme::button(ui, label, wide, help)
}
