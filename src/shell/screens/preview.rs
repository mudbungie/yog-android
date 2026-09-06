//! **A bounded file on the glass** — the one painter for the two answers that
//! carry one (DESIGN §13.15): a worktree file's preview, and a changed file's
//! patch.
//!
//! **The bytes are painted and never parsed.** A patch is a unified diff and
//! this app does not read one: the engine bounded it, and what a review from
//! the glass wants is what it says. The desktop does not decode it either
//! (lernie DESIGN §4.33).
//!
//! **A fenced block scrolls and prose wraps** (§13.2, bl-b62b). Every line of
//! a file MEANS something — a wrapped line of a patch is a changed line — so
//! this is the one shape in this app that gets a horizontal scroller of its
//! own, and the sentence beside it wraps like everything else.

use eframe::egui;

use crate::codec::Preview;

/// Paint one bounded file: what the engine could say about it, then the bytes
/// it handed over. A binary file has no bytes to show and says so — an empty
/// monospace block would read as an empty file.
pub(super) fn bytes(ui: &mut egui::Ui, preview: &Preview) {
    match preview {
        Preview::Text(text) => fenced(ui, text),
        Preview::Truncated { text, size } => {
            ui.weak(format!("cut short — {size} bytes in all"));
            fenced(ui, text);
        }
        Preview::Binary { size } => {
            ui.weak(format!("binary — {size} bytes, nothing to read"));
        }
    }
}

/// The fence: monospace, in its own horizontal scroller, wrapping nothing.
fn fenced(ui: &mut egui::Ui, text: &str) {
    egui::ScrollArea::horizontal()
        .id_salt("preview")
        .show(ui, |ui| {
            ui.add(
                egui::Label::new(egui::RichText::new(text).monospace())
                    .wrap_mode(egui::TextWrapMode::Extend),
            );
        });
}
