//! **The render-and-see seam** (bl-243b): the one thing this app says about
//! what it is painting, so a harness driving a headless emulator can assert
//! *reachability* rather than compare pictures.
//!
//! **The platform's accessibility tree is not that surface and cannot be
//! made one.** egui paints into a single opaque view, so `uiautomator dump`
//! over this app comes back as one `android.view.View` with no text in it at
//! all — no label, no button, no row. Every assertion the ball behind this
//! file wanted to write against that dump is unwritable, and no amount of
//! care at the harness end changes it: there is nothing there to read. The
//! harness captures the dump beside every screenshot anyway, so that the
//! emptiness is evidence in the run rather than a claim in a document.
//!
//! So the app states it instead, in the one channel an APK has (logcat), and
//! states exactly **three facts — none of which is world content**:
//!
//! - **the name of the screen the dispatch chose**, written at the arm that
//!   chose it, so the name has one home and cannot drift from the branch;
//! - **where a NAMED control was painted**, in device pixels — one field per
//!   control, spelled by the paint site (`note_control`). egui's controls
//!   carry no accessibility node (§15.1), so a rectangle the app reports is
//!   the only way a harness reaches one: the mark, which is the sole way into
//!   the configuration surface and carries no text at all; the first
//!   conversation row, whose long press is the only way into the row menu
//!   (bl-f97c); and the two world entries on the roster, which are the only
//!   way to the trail and the queue (§13.8, bl-35bd).
//!
//!   **Only the FIRST of a list, and never a row per row.** The line names
//!   controls the APP has — a fixed set that grows when a surface is built —
//!   and never one per thing in the WORLD, which is the line this file draws
//!   and the reason the vocabulary is a paint-site string rather than an
//!   enumeration of what is on screen.
//!
//! **Nothing else may go down this channel.** No bar title, no row label, no
//! identity: logcat is device-wide and readable by anything holding the debug
//! bridge, so a workspace name written here is world state disclosed to the
//! whole device. A screen name and a rectangle disclose the shape of the app,
//! which its own store already publishes.
//!
//! Every fact is **frame-scoped**, like `Shell::back`: they are taken at the
//! end of the pass, so a screen that stops painting stops saying it is there,
//! and a rectangle is never one from a frame ago. The line is emitted only
//! when it CHANGES — a repaint at 60 Hz is not news, and a log a harness has
//! to de-duplicate is a log that will be de-duplicated wrongly once.

use eframe::egui;

use super::Shell;

/// The marker every probe line begins with, and the whole of the harness's
/// vocabulary — `scripts/screens.sh` greps for exactly this.
///
/// **It is in the MESSAGE, not the logcat tag.** `android_logger` derives a
/// record's tag from its module path, so this line arrives tagged
/// `yog_android::shell::app::probe` whatever `target:` says — a tag filter
/// would be a harness coupled to where this file happens to live, and would
/// go quietly silent the day it moves. A marker in the text moves with it.
pub(crate) const MARKER: &str = "yog.screen";

impl Shell {
    /// Name the screen this pass is painting.
    ///
    /// Called by the dispatch arm that chose it rather than derived from the
    /// same state a second time: a derivation beside the branch is a second
    /// authority for one fact, and the two disagree the first time a branch
    /// moves.
    pub(crate) fn note_screen(&mut self, name: &'static str) {
        self.screen = Some(name);
    }

    /// Record where a named control was painted, in **device pixels** — the
    /// unit `adb shell input tap` takes, so the harness does no arithmetic of
    /// its own and cannot get the scale wrong. egui works in points; the scale
    /// is read here, at the paint, from the context that laid the rect out.
    ///
    /// The name is the paint site's own word and is the harness's whole
    /// vocabulary for reaching that control. A second call under one name in
    /// one pass is the FIRST one — the first row of a list is what the walk
    /// presses, and a list that reported its last row would move the target
    /// with the world.
    pub(crate) fn note_control(&mut self, name: &'static str, ui: &egui::Ui, rect: egui::Rect) {
        if self.at.iter().any(|(known, _)| *known == name) {
            return;
        }
        self.at.push((name, pixels(ui, rect)));
    }

    /// Say it, at the end of the pass and only when it changed.
    pub(super) fn probe(&mut self) {
        let at = std::mem::take(&mut self.at);
        let Some(screen) = self.screen.take() else {
            return;
        };
        // The rectangle fields, in the order they were painted. Built by
        // `fold` rather than by pushing `format!` at a `String`, which is the
        // shape `clippy::pedantic` refuses two ways at once
        // (`format_push_string`, `format_collect`) and whose suggested
        // replacement — `write!` into a `String` — hands back a `Result` that
        // cannot fail and that this crate may not `unwrap` (AGENTS rule 4).
        let line = at
            .iter()
            .fold(format!("{MARKER} screen={screen}"), |line, (key, rect)| {
                line + " " + key + "=" + &spell(*rect)
            });
        if self.probed == line {
            return;
        }
        log::info!("{line}");
        self.probed = line;
    }
}

/// A rect in **device pixels** — the unit `adb shell input tap` takes, so the
/// harness does no arithmetic of its own and cannot get the scale wrong. egui
/// works in points; the scale is read here, at the paint, from the context
/// that laid the rect out. One helper because two rectangles now cross this
/// channel and a second copy of the rounding would be a second answer.
fn pixels(ui: &egui::Ui, rect: egui::Rect) -> [i32; 4] {
    let ppp = ui.ctx().pixels_per_point();
    let px = |v: f32| (v * ppp).round() as i32;
    [
        px(rect.left()),
        px(rect.top()),
        px(rect.width()),
        px(rect.height()),
    ]
}

/// One rectangle in the harness's own comma-separated spelling.
fn spell([x, y, w, h]: [i32; 4]) -> String {
    format!("{x},{y},{w},{h}")
}
