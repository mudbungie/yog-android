//! **Getting text off the glass** (DESIGN §13.2, bl-7781): the long press that
//! copies a prose surface, and the one seam every copy leaves by.
//!
//! **A phone has no selection here, and pretending otherwise is the defect.**
//! This app paints into a single GPU surface: there is no platform text view
//! under a transcript row, so there are no platform selection handles to have,
//! and egui does not synthesize them — it disables drag-to-select on a touch
//! screen outright (`Label::layout_in_ui`), because a drag over prose inside a
//! `ScrollArea` is how a phone SCROLLS. A transcript that cannot be scrolled
//! by dragging its own text would be a worse app than one that cannot select
//! a range inside it.
//!
//! **So the unit is the block, and the gesture is the one this app already
//! has.** A long press copies the whole of what a surface says — a transcript
//! row and its payload, a tool result, a records line, a file preview, an
//! address, a banner. That is the gesture §13.5's row menu already opens on,
//! read the same way: egui synthesizes a secondary click from a touch held
//! past its own click duration, which is `Response::secondary_clicked`. It
//! senses CLICK and never drag, so nothing here takes a scroll away.
//!
//! **The confirmation is the platform's own.** Android 13 and later put a
//! preview of the clip on the glass whenever an app writes one, so a second
//! sentence from this app would be the same fact said twice, in a place the
//! operator is not looking.
//!
//! **Every copy leaves by one door.** A surface asks egui to copy
//! (`Context::copy_text`) and [`copied`] hands what egui collected to the
//! platform — so a copy this app never wrote, one of egui's own out of a
//! `TextEdit`, reaches the Android clipboard by the same route. eframe's
//! clipboard is a desktop one and does nothing on this platform, which is why
//! the commands are taken here rather than left for it.
//!
//! The write itself is `tools::paper`'s door, which the interface corpus
//! already owns (`dev.yog.Paper.clipboardSet`): one clipboard in this app, not
//! two. This file is android-only and excluded from coverage — what it decides
//! is which commands are copies, and the sentence the door answers with is
//! parsed by `tools::bridged::answer`, which is pure and tested.

use eframe::egui;

/// Add a prose label whose whole text a long press copies. The response is
/// handed back for a caller that has its own use for it.
pub(crate) fn copyable(ui: &mut egui::Ui, label: egui::Label, text: &str) -> egui::Response {
    // CLICK, never drag: a label that took the drag would take the scroll
    // with it (module doc). The long press arrives as the secondary click
    // egui synthesizes from it, exactly as the row menu reads one.
    let response = ui.add(label.sense(egui::Sense::click()));
    if response.secondary_clicked() {
        ui.ctx().copy_text(text.to_owned());
    }
    response
}

/// The plain case: a line of prose in the ordinary ink.
pub(crate) fn said(ui: &mut egui::Ui, text: impl AsRef<str>) -> egui::Response {
    let text = text.as_ref();
    copyable(ui, egui::Label::new(text), text)
}

/// The same line in the secondary ink — `Ui::weak`'s replacement, spelling
/// for spelling, so a surface adopting this does not also change colour.
pub(crate) fn weak(ui: &mut egui::Ui, text: impl AsRef<str>) -> egui::Response {
    let text = text.as_ref();
    copyable(ui, egui::Label::new(egui::RichText::new(text).weak()), text)
}

/// And in a state's accent — `Ui::colored_label`'s replacement. The colour is
/// the caller's, which is always a `theme` token (`rules/no-literal-colour`).
pub(crate) fn tinted(
    ui: &mut egui::Ui,
    ink: egui::Color32,
    text: impl AsRef<str>,
) -> egui::Response {
    let text = text.as_ref();
    copyable(
        ui,
        egui::Label::new(egui::RichText::new(text).color(ink)),
        text,
    )
}

/// Hand what egui collected this pass to the platform's clipboard, and leave
/// every other command where it was for eframe to spend.
pub(crate) fn copied(ctx: &egui::Context) {
    let mut taken = Vec::new();
    ctx.output_mut(|out| {
        let commands = std::mem::take(&mut out.commands);
        for command in commands {
            match command {
                egui::OutputCommand::CopyText(text) => taken.push(text),
                other => out.commands.push(other),
            }
        }
    });
    for text in taken {
        // A refusal is the platform's sentence and there is nowhere on this
        // glass for it: the banner is what the ENGINE said, and a clipboard
        // that would not take a copy is not a fact about the world. logcat is
        // the witness, as it is for the inset probe.
        let said = crate::tools::paper::to_clipboard(&text);
        log::info!("clipboard: {said}");
    }
}
