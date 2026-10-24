//! **A model's prose, painted with its marks read** (bl-7355, DESIGN §22
//! item 4): the blocks and spans are `crate::markdown`'s, pure and tested;
//! this sets them. A fence is a `SURFACE` block in the code face that
//! scrolls sideways rather than wrapping (the desktop's bl-beef block, and
//! §13.2's wide-content rule); a heading is the heading size; a code span is
//! the code face on the surface; an italic span is egui's skew. **A strong
//! span is the body's face and ink with its marks taken off**: the bundled
//! set has no bold face and a font file cannot enter this tree (the
//! disclosure gate reads no binary), and a brighter ink would be a colour
//! claiming a state. The parser keeps the distinction for the day a face
//! arrives.
//!
//! A long press copies the row's whole words, marks and all — what the
//! server said, which is what an operator pastes elsewhere.

use eframe::egui;

use crate::markdown::{Block, Span, Style, blocks};
use crate::shell::theme::rgb;
use crate::theme::{self, space};

/// Paint `words`, a model's, block by block. `key` is the row's, which salts
/// each block's sideways scroll so two blocks never share one.
pub(super) fn paint(ui: &mut egui::Ui, key: &str, words: &str) {
    ui.vertical(|ui| {
        for (at, block) in blocks(words).into_iter().enumerate() {
            match block {
                Block::Prose(spans) => runs(ui, &spans, None, words),
                Block::Heading(spans) => {
                    runs(ui, &spans, Some(theme::type_scale::HEADING), words);
                }
                Block::Code(text) => code(ui, (key, at), &text, words),
            }
        }
    });
}

/// One stretch of prose or a heading, wrapped at the width it has; `size`
/// is the heading's, else the body's.
fn runs(ui: &mut egui::Ui, spans: &[Span], size: Option<f32>, whole: &str) {
    let body = egui::TextStyle::Body.resolve(ui.style());
    let face = size.map_or(body, egui::FontId::proportional);
    let mono = egui::TextStyle::Monospace.resolve(ui.style());
    let ink = ui.visuals().text_color();
    let mut job = egui::text::LayoutJob::default();
    for span in spans {
        let mut format = egui::TextFormat::simple(face.clone(), ink);
        match span.style {
            Style::Plain | Style::Strong => {}
            Style::Code => {
                format.font_id = mono.clone();
                format.background = rgb(theme::SURFACE);
            }
            Style::Italic => format.italics = true,
        }
        job.append(&span.text, 0.0, format);
    }
    let label = egui::Label::new(job).wrap();
    crate::shell::clip::copyable(ui, label, whole);
}

/// **One fenced block**: the code face on the surface, scrolled sideways
/// inside the column rather than wrapped or laid off the glass.
fn code(ui: &mut egui::Ui, salt: (&str, usize), text: &str, whole: &str) {
    egui::Frame::NONE
        .fill(rgb(theme::SURFACE))
        .corner_radius(egui::CornerRadius::same(theme::RADIUS))
        .inner_margin(egui::Margin::same(space::S as i8))
        .show(ui, |ui| {
            egui::ScrollArea::horizontal()
                .id_salt(("code", salt))
                .show(ui, |ui| {
                    let rich = egui::RichText::new(text).monospace();
                    let label = egui::Label::new(rich).extend();
                    crate::shell::clip::copyable(ui, label, whole);
                });
        });
}
