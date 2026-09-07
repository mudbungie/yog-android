//! **How one conversation row is painted** (DESIGN §13.20, bl-4d17): the
//! threading column beside it, the band it sits in, and each of its lines
//! elided against that band's width.
//!
//! **The band is the language's row** (docs/STYLE.md, bl-83be): bare ground
//! at rest, a tint only under a thumb, a hairline under it, and no outline.
//! Its first line — the name, the mark, the stamp — is in the row's tone,
//! and every line after it is weak ink, so a list of twenty reads as twenty
//! names and the state each is in, not twenty paragraphs.
//!
//! Split from the list that spends it on the seam that file already draws —
//! what a list IS against what one row LOOKS like — and it is the third split
//! of `rows.rs`, after the row menu (bl-06d3).

use eframe::egui;

/// **One row: the threading column, then the box** (§13.20, bl-4d17).
///
/// It is allocated and painted rather than handed to `egui::Button` for two
/// reasons the button cannot answer. A button CENTRES its text, which is why
/// the list read as two alignments — short rows centred, long rows apparently
/// left-aligned because they had overflowed; and `TextWrapMode::Truncate`
/// elides a whole galley to ONE row, so a two-line row cannot ask for it.
/// Laying each line as its own single-row job is the only spelling in which
/// every line is elided against the box's own width.
///
/// The box starts at the row's indent, so its left edge is where the parent's
/// TEXT begins less the connector column — which is what makes the elbow
/// point at something.
pub(super) fn threaded(
    ui: &mut egui::Ui,
    rails: &[bool],
    lines: &[String],
    ink: egui::Color32,
) -> egui::Response {
    let font = egui::TextStyle::Body.resolve(ui.style());
    let line = ui.ctx().fonts_mut(|fonts| fonts.row_height(&font));
    let pad = ui.spacing().button_padding;
    let gap = ui.spacing().item_spacing.y;
    let lines_tall = line * lines.len() as f32 + pad.y * 2.0;
    let tall = lines_tall.max(crate::shell::mark::TOUCH);
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), tall), egui::Sense::click());
    if !ui.is_rect_visible(rect) {
        return response;
    }
    let boxed = rect.with_min_x(rect.left() + crate::roster::indent(rails.len()));
    if response.is_pointer_button_down_on() || response.hovered() {
        ui.painter().rect_filled(
            boxed,
            egui::CornerRadius::same(crate::theme::RADIUS),
            crate::shell::theme::rgb(crate::theme::RAISED),
        );
    }
    ui.painter().hline(
        boxed.x_range(),
        rect.bottom(),
        egui::Stroke::new(1.0, crate::shell::theme::rgb(crate::theme::HAIRLINE)),
    );
    // A row shorter than the touch floor is padded to it, and its lines sit in
    // the MIDDLE of what that bought rather than at the top: a conversation
    // nobody has spoken in is one line, and one line stranded against the top
    // edge of a 44-point box reads as a row that lost something.
    let first = rect.top() + pad.y + (tall - lines_tall) / 2.0;
    let elbow = first + line / 2.0;
    connectors(ui, rect, boxed.left(), rails, elbow, gap);
    let wide = boxed.width() - pad.x * 2.0;
    let weak = ui.visuals().weak_text_color();
    for (at, text) in lines.iter().enumerate() {
        let job = elided(text, &font, wide);
        let galley = ui.ctx().fonts_mut(|fonts| fonts.layout_job(job));
        let top = first + line * at as f32;
        let color = if at == 0 { ink } else { weak };
        ui.painter()
            .galley(egui::pos2(boxed.left() + pad.x, top), galley, color);
    }
    response
}

/// **The connectors themselves**: a rule down every column that carries one,
/// and an elbow into this row's own box.
///
/// They are STROKES and not the box-drawing glyphs the idiom is usually
/// written with, for the reason `ATTENTION_MARK` records: a glyph the bundled
/// faces do not carry paints as the missing-glyph box, and U+2514/U+2502 are
/// not in egui's proportional face. A stroke also spans a row of any height,
/// which a character cannot — these rows are one, two or three lines tall.
///
/// Every column's centre is read out of `roster::indent`, so this file never
/// learns the step and cannot disagree with the offset the box is drawn at.
fn connectors(ui: &egui::Ui, rect: egui::Rect, boxed: f32, rails: &[bool], elbow: f32, gap: f32) {
    // Faint ink: a thread is structure, and structure is read after the
    // words, never before them (STYLE.md).
    let stroke = egui::Stroke::new(1.0, crate::shell::theme::rgb(crate::theme::INK_FAINT));
    for (at, carries) in rails.iter().enumerate() {
        let column = f32::midpoint(crate::roster::indent(at), crate::roster::indent(at + 1));
        let x = rect.left() + column;
        let own = at + 1 == rails.len();
        // The rule is drawn a gap high, so the segment meets the one the row
        // above drew: the space between two rows is where a thread would
        // otherwise break.
        let top = egui::pos2(x, rect.top() - gap);
        if *carries {
            ui.painter()
                .line_segment([top, egui::pos2(x, rect.bottom() + gap)], stroke);
        } else if own {
            ui.painter()
                .line_segment([top, egui::pos2(x, elbow)], stroke);
        }
        if own {
            ui.painter()
                .line_segment([egui::pos2(x, elbow), egui::pos2(boxed, elbow)], stroke);
        }
    }
}

/// One line, laid out to elide at the box's width rather than to run off the
/// screen. `max_rows: 1` is what says *this is a line*: the text arrives
/// already folded (`roster::lines`), so nothing here can wrap a clause into a
/// row the next line was going to use.
fn elided(text: &str, font: &egui::FontId, wide: f32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::single_section(
        text.to_owned(),
        egui::TextFormat {
            font_id: font.clone(),
            color: egui::Color32::PLACEHOLDER,
            ..Default::default()
        },
    );
    job.wrap = egui::text::TextWrapping {
        max_width: wide.max(0.0),
        max_rows: 1,
        break_anywhere: true,
        overflow_character: Some('\u{2026}'),
    };
    job
}
