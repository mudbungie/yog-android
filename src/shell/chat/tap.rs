//! **A line that is its own touch target** (STYLE §5, *a block*): the glyph
//! in the gutter at the block's left edge, then one or more single-row runs of
//! words, each elided at the width it has. Allocated and painted rather than
//! laid out from labels, for `thread.rs`'s reason: a run that must stay one
//! row has to be elided against the box's own width, and the whole line —
//! not the glyph — is what a fingertip lands on.

use eframe::egui;

use crate::shell::theme::rgb;
use crate::theme::{self, space};

/// The gutter's width: the column every block's glyph stands in.
pub(super) const GUTTER: f32 = space::L + space::XS;

/// One run of a line: its words in their inks, in one face.
pub(super) struct Run {
    pub(super) parts: Vec<(String, egui::Color32)>,
    pub(super) mono: bool,
}

impl Run {
    /// A run in the body face.
    pub(super) fn body(parts: Vec<(String, egui::Color32)>) -> Self {
        Self { parts, mono: false }
    }
}

/// **The gutter alone**, for a block laid out from labels: the glyph at the
/// left edge, one body row tall, the column [`GUTTER`] wide.
pub(super) fn glyph(ui: &mut egui::Ui, mark: &str, ink: egui::Color32) {
    let font = egui::TextStyle::Body.resolve(ui.style());
    let tall = ui.ctx().fonts_mut(|fonts| fonts.row_height(&font));
    let (rect, _) = ui.allocate_exact_size(egui::vec2(GUTTER, tall), egui::Sense::hover());
    ui.painter().text(
        rect.left_center(),
        egui::Align2::LEFT_CENTER,
        mark,
        font,
        ink,
    );
}

/// **A line at `indent`**: the gutter glyph, then `runs` one row each, the
/// whole at least `tall` points high with the runs centred in it. A line
/// asked to be a target (`tall > 0`) wears `RAISED` under a thumb.
pub(super) fn line(
    ui: &mut egui::Ui,
    indent: f32,
    gutter: (&str, egui::Color32),
    runs: &[Run],
    tall: f32,
    help: &str,
) -> egui::Response {
    let body = egui::TextStyle::Body.resolve(ui.style());
    let mono = egui::TextStyle::Monospace.resolve(ui.style());
    let row = ui.ctx().fonts_mut(|fonts| fonts.row_height(&body));
    let text = row * runs.len().max(1) as f32;
    let high = (text + space::XS).max(tall);
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), high), egui::Sense::click());
    // **A line that is a target tells** (§23 item 10); a plain line is
    // prose, and prose keeps its long press for copying.
    if tall > 0.0 {
        crate::shell::theme::told::told(ui, &response, help);
    }
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let boxed = rect.with_min_x(rect.left() + indent);
    if tall > 0.0 && (response.is_pointer_button_down_on() || response.hovered()) {
        ui.painter().rect_filled(
            boxed,
            egui::CornerRadius::same(theme::RADIUS),
            rgb(theme::RAISED),
        );
    }
    let top = rect.top() + (high - text) / 2.0;
    ui.painter().text(
        egui::pos2(boxed.left(), top + row / 2.0),
        egui::Align2::LEFT_CENTER,
        gutter.0,
        body.clone(),
        gutter.1,
    );
    let x = boxed.left() + GUTTER;
    for (at, run) in runs.iter().enumerate() {
        let font = if run.mono { &mono } else { &body };
        let job = elided(run, font, rect.right() - x);
        let galley = ui.ctx().fonts_mut(|fonts| fonts.layout_job(job));
        ui.painter().galley(
            egui::pos2(x, top + row * at as f32),
            galley,
            rgb(theme::INK),
        );
    }
    response
}

/// One run laid out as ONE row, elided at `wide` (`thread.rs`'s spelling).
fn elided(run: &Run, font: &egui::FontId, wide: f32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    for (text, color) in &run.parts {
        job.append(
            text,
            0.0,
            egui::TextFormat {
                font_id: font.clone(),
                color: *color,
                ..Default::default()
            },
        );
    }
    job.wrap = egui::text::TextWrapping {
        max_width: wide.max(0.0),
        max_rows: 1,
        break_anywhere: true,
        overflow_character: Some('\u{2026}'),
    };
    job
}
