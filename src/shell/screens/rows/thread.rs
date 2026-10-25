//! **The threading column beside a conversation row** (DESIGN §13.20,
//! bl-4d17): a rule down every column that carries one, and an elbow into the
//! row's own box. The row itself is the drawer's (`shell/drawer/list.rs`,
//! §24 item 2) — the glyph, the name, the age at the right end and the line
//! beneath — and this is the one piece of it that is about the LIST rather
//! than the row.
//!
//! **The connectors are STROKES** and not the box-drawing glyphs the idiom is
//! usually written with, for the reason `ATTENTION_MARK` records: a glyph the
//! bundled faces do not carry paints as the missing-glyph box, and
//! U+2514/U+2502 are not in egui's proportional face. A stroke also spans a
//! row of any height, which a character cannot — these rows are one or two
//! lines tall.
//!
//! Every column's centre is read out of `roster::indent`, so this file never
//! learns the step and cannot disagree with the offset the box is drawn at.
//! Which columns carry a rule is `roster::threads`, pure and asserted.

use eframe::egui;

/// **The connectors**: `rect` is the row's whole band, `boxed` the left edge
/// of its box, `elbow` the height the elbow reaches it at, `gap` the item
/// spacing the rule is drawn through so it meets the row above's.
///
/// Faint ink: a thread is structure, and structure is read after the words,
/// never before them (STYLE.md).
pub(in crate::shell) fn connectors(
    ui: &egui::Ui,
    rect: egui::Rect,
    boxed: f32,
    rails: &[bool],
    elbow: f32,
    gap: f32,
) {
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
