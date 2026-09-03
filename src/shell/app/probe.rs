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
//! - **where the mark was painted**, in device pixels. The mark is the only
//!   way into the configuration surface (§13.2) and it carries no text at
//!   all, so it is the one control a harness cannot otherwise find. It says
//!   where a tap must land; the app is still the thing that decides what a
//!   tap there means.
//! - **where the first conversation row was painted** (bl-f97c), on the one
//!   screen that has rows. It is the mark's case exactly: the row menu opens
//!   on a long press and nothing else opens it, and a row carries no node a
//!   harness can address. Only the FIRST — the walk needs one row to press,
//!   and a rectangle per row would be a channel that grows with the WORLD
//!   rather than with the app, which is the line this file draws.
//!
//! **Nothing else may go down this channel.** No bar title, no row label, no
//! identity: logcat is device-wide and readable by anything holding the debug
//! bridge, so a workspace name written here is world state disclosed to the
//! whole device. A screen name and a rectangle disclose the shape of the app,
//! which its own store already publishes.
//!
//! Both facts are **frame-scoped**, like `Shell::back`: they are taken at the
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

    /// Record where the mark was painted, in **device pixels** — the unit
    /// `adb shell input tap` takes, so the harness does no arithmetic of its
    /// own and cannot get the scale wrong. egui works in points; the scale is
    /// read here, at the paint, from the context that laid the rect out.
    pub(crate) fn note_mark(&mut self, ui: &egui::Ui, rect: egui::Rect) {
        self.mark_at = Some(pixels(ui, rect));
    }

    /// Record where the first conversation row was painted, in device pixels
    /// and by the same arithmetic the mark takes.
    pub(crate) fn note_row(&mut self, ui: &egui::Ui, rect: egui::Rect) {
        self.row_at = Some(pixels(ui, rect));
    }

    /// Say it, at the end of the pass and only when it changed.
    pub(super) fn probe(&mut self) {
        let mark = self.mark_at.take();
        let row = self.row_at.take();
        let Some(screen) = self.screen.take() else {
            return;
        };
        let mut line = format!("{MARKER} screen={screen}");
        if let Some(at) = mark {
            line.push_str(&format!(" mark={}", spell(at)));
        }
        if let Some(at) = row {
            line.push_str(&format!(" row={}", spell(at)));
        }
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
