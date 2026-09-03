//! **`act:<op>` — the tag a verb-firing control carries** (yog
//! `docs/PARITY.md` §4, bl-fe4c), and the debug-gated inventory it is written
//! into.
//!
//! The visible label stays a human word; the tag is machine metadata, and the
//! op token is the one name that already exists everywhere — the help row's
//! `verb`, the envelope's `op`, the corpus filename — so no translation table
//! is born. `crate::parity` owns the spelling and the judging; this file is
//! only the paint-side recorder.
//!
//! **This is PARITY §6's fallback, and the preferred route was walked first**
//! (DESIGN §15.1). The tag was meant to ride the control's accessibility node,
//! where `uiautomator` and `TalkBack` would both read it. That route is one
//! feature and one dependency away and it aborts the process: AccessKit's
//! android adapter unwraps a JNI call while raising its first accessibility
//! event under `GameActivity`, so the app dies the moment ANY accessibility
//! client attaches — the harness, or a user's screen reader. The manifest
//! records the failure at the line where the dependency would go, and bl-a6f3
//! is the exit. So the inventory is self-reported for now, and §5's assertion
//! is unchanged: only where the bytes come from differs.
//!
//! **What a self-report can still honestly claim.** Not "this op exists in the
//! source" — a constant would say that, and it would keep saying it after the
//! control was deleted. What is recorded here is *this control was laid out,
//! and its rectangle was on the glass*, taken at the paint site of the widget
//! that fires the op, on a screen the walk actually visited. A control behind
//! a gate the walk never opens records nothing, which is what makes an
//! unvisited screen fail rather than pass.
//!
//! **The inventory is a file in app-private storage, written only when the
//! harness asks for it**, and the gate is the presence of a directory the
//! harness creates (`<internal>/parity/`). No marker, no writing, no file:
//! this costs a shipped app one `exists` check per change and discloses
//! nothing, which is the difference between this channel and `app/probe.rs`'s
//! — logcat is device-wide, and app-private storage is not. What it holds is
//! op tokens out of a published vocabulary, never world content.

use std::collections::BTreeSet;

use eframe::egui;

use super::app::Shell;

/// Where the inventory lives, under the app's own internal storage, and the
/// directory whose existence arms the writing.
const DIR: &str = "parity";
const FILE: &str = "acts.txt";

/// egui's own memory is where the pass accumulates: the recorder is called
/// from free functions and from three `impl Shell` blocks, and threading a
/// `&mut Shell` through every paint site to reach one set would be a borrow
/// tax on every control for a debug artifact.
fn slot() -> egui::Id {
    egui::Id::new("yog.parity.acts")
}

/// Record every op this control fires. More than one because a control may be
/// one gesture to a thumb and two ops on the wire: the starter stages and
/// fires (`prepare` then `prompt`, `seat::acts`), and a selector is the
/// affordance for both the read that fills its list and the act its items
/// post.
pub(super) fn acts(ui: &egui::Ui, response: &egui::Response, ops: &[&str]) {
    if !ui.is_rect_visible(response.rect) {
        return;
    }
    ui.data_mut(|data| {
        let held = data.get_temp_mut_or_default::<BTreeSet<String>>(slot());
        for op in ops {
            held.insert((*op).to_owned());
        }
    });
}

/// The one-op case, which is most of them.
pub(super) fn act(ui: &egui::Ui, response: &egui::Response, op: &str) {
    acts(ui, response, &[op]);
}

impl Shell {
    /// Write the inventory, at the end of the pass and only when it grew.
    ///
    /// The set accumulates for the life of the process rather than per frame:
    /// what the file answers is *what did this launch reach*, and the walk
    /// pulls it after each screen, so the union across a run's files is the
    /// union across its walk. A control painted on one screen does not stop
    /// being reachable when the next screen paints.
    pub(super) fn note_acts(&mut self, ctx: &egui::Context) {
        let held = ctx.data(|data| data.get_temp::<BTreeSet<String>>(slot()));
        let Some(held) = held else { return };
        if held.len() == self.acted {
            return;
        }
        let Some(dir) = self.android.internal_data_path().map(|at| at.join(DIR)) else {
            return;
        };
        if !dir.is_dir() {
            return;
        }
        match std::fs::write(dir.join(FILE), crate::parity::inventory(&held)) {
            Ok(()) => self.acted = held.len(),
            Err(why) => log::warn!("parity inventory: {why}"),
        }
    }
}
