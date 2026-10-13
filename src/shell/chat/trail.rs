//! **A trail** (DESIGN §22 item 4; lernie `ui::chat::trail`): the fold head
//! `tool calls (N)`, then each step on a stem — a call with `•` and how it
//! ended (`✔`, or `✖` in the error accent), a result folded to its first line
//! and unfolded in place in the code face, reasoning and remarks as prose —
//! and the `response` elbow that parts a trail from the prose after it.
//!
//! **The stems are painted strokes**, the conversation tree's own rails
//! (`screens/rows/thread.rs`, §13.20): `├ └ │` are in none of the bundled
//! faces, and a stroke spans a step of any height.

use eframe::egui;

use super::tap::{self, GUTTER, Run};
use crate::rows::{Kind, Row, Tone};
use crate::shell::theme::rgb;
use crate::theme::{self, space};

/// What parts a trail from the prose that follows it (the desktop's word).
const RESPONSE: &str = "response";

/// **The fold head**: its glyph in the gutter, `tool calls (N)` in weak ink
/// and a rolled-up turn's census beside it in faint — the whole line the
/// touch floor tall, and its toggle.
pub(super) fn head(ui: &mut egui::Ui, row: &Row) -> bool {
    let mut parts = vec![(row.prefix.clone(), rgb(theme::INK_WEAK))];
    if !row.preview.is_empty() {
        parts.push((format!(" · {}", row.preview), rgb(theme::INK_FAINT)));
    }
    let (mark, ink) = super::gutter(ui, row);
    tap::line(ui, 0.0, (&mark, ink), &[Run::body(parts)], theme::TOUCH).clicked()
}

/// **One step on its stem**: the stem's glyph, the step's words with how a
/// call ended beside them and its fold at the end, the folded payload's first
/// line under them; unfolded, the payload in place. A long press copies the
/// whole payload.
pub(super) fn step(ui: &mut egui::Ui, row: &Row, last: bool) -> bool {
    let top = ui.cursor().min.y;
    let left = ui.max_rect().left();
    let foldable = !row.body.is_empty();
    let code = matches!(row.kind, Kind::Call | Kind::Answered | Kind::Result);
    let mut parts = vec![(row.prefix.clone(), ink(ui, row))];
    if let Some(mark) = crate::rows::mark(row) {
        let hue = if row.tone == Tone::Bad {
            ui.visuals().error_fg_color
        } else {
            rgb(theme::INK_WEAK)
        };
        parts.push((format!(" {mark}"), hue));
    }
    if foldable {
        let fold = if row.expanded {
            theme::glyph::OPEN
        } else {
            theme::glyph::SHUT
        };
        parts.push((format!("  {fold}"), rgb(theme::INK_WEAK)));
    }
    let mut runs = vec![Run::body(parts)];
    let unfolded = foldable && row.expanded;
    if !unfolded && !row.preview.is_empty() {
        runs.push(Run {
            parts: vec![(row.preview.clone(), rgb(theme::INK_WEAK))],
            mono: code,
        });
    }
    let tall = if foldable { theme::TOUCH } else { 0.0 };
    let (glyph, hue) = super::gutter(ui, row);
    let hit = tap::line(ui, GUTTER, (&glyph, hue), &runs, tall);
    if hit.secondary_clicked() {
        let whole = if foldable { &row.body } else { &row.preview };
        ui.ctx().copy_text(whole.clone());
    }
    let font = egui::TextStyle::Body.resolve(ui.style());
    let line = ui.ctx().fonts_mut(|fonts| fonts.row_height(&font));
    let elbow = hit.rect.center().y - line * (runs.len() as f32 - 1.0) / 2.0;
    if unfolded {
        ui.horizontal_top(|ui| {
            ui.add_space(GUTTER * 2.0);
            if code {
                crate::shell::clip::copyable(
                    ui,
                    egui::Label::new(
                        egui::RichText::new(&row.body).text_style(egui::TextStyle::Monospace),
                    )
                    .wrap(),
                    &row.body,
                );
            } else {
                super::body(ui, &row.body);
            }
        });
    }
    let bottom = ui.cursor().min.y;
    stem(ui, left, top, elbow, (!last).then_some(bottom));
    hit.clicked() && foldable
}

/// **The `response` elbow**: a stem down into the word, in weak ink.
pub(super) fn response(ui: &mut egui::Ui) {
    ui.add_space(space::XS);
    let left = ui.max_rect().left();
    let top = ui.cursor().min.y;
    let said = ui
        .horizontal(|ui| {
            ui.add_space(GUTTER);
            ui.add(
                egui::Label::new(
                    egui::RichText::new(RESPONSE)
                        .small()
                        .color(rgb(theme::INK_WEAK)),
                )
                .truncate(),
            );
        })
        .response;
    stem(ui, left, top, said.rect.center().y, None);
}

/// A step's ink: a state's accent while it is the show (held, running) or
/// failed, weak ink at rest — how a call ended is its mark, not a hue.
fn ink(ui: &egui::Ui, row: &Row) -> egui::Color32 {
    match row.tone {
        Tone::Held | Tone::InFlight | Tone::Live | Tone::Bad => super::tone_hue(ui, &row.tone),
        _ => rgb(theme::INK_WEAK),
    }
}

/// **The stroke**: down the gutter from above `top` to the elbow, across to
/// the step's glyph, and on down to `through` where the stem continues.
fn stem(ui: &egui::Ui, left: f32, top: f32, elbow: f32, through: Option<f32>) {
    let stroke = egui::Stroke::new(1.0, rgb(theme::INK_FAINT));
    let x = left + space::XS;
    let reach = top - ui.spacing().item_spacing.y - space::M;
    let painter = ui.painter();
    painter.line_segment([egui::pos2(x, reach), egui::pos2(x, elbow)], stroke);
    painter.line_segment(
        [
            egui::pos2(x, elbow),
            egui::pos2(left + GUTTER - space::XS, elbow),
        ],
        stroke,
    );
    if let Some(bottom) = through {
        painter.line_segment([egui::pos2(x, elbow), egui::pos2(x, bottom)], stroke);
    }
}
