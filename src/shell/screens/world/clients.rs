//! **The machines roster** (REMOTE §5, §5.1; DESIGN §13.14): which machines
//! may execute for this workspace, and what each one says it offers.
//!
//! **Two lifetimes on one row, and the screen says both** (lernie DESIGN
//! §4.28, whose ruling transfers whole). `present` is an observation — true at
//! the instant the engine answered — and the advertised set is a statement the
//! machine last made, which stands whether or not it is connected. A row
//! therefore reads *not connected* beside a full set as the ordinary thing: a
//! tool host holds its connection only while it is waiting for work, so a busy
//! machine and an absent one are indistinguishable from here, and the sentence
//! says so rather than leaving an operator to read *absent* as *broken*.
//!
//! **The consent is stated on every tool, present or absent.** `subject_cwd`
//! is what yog's worktree lane routes on — the advertising box consenting to
//! run that tool at a directory the invocation names — and it is the fact the
//! desktop's own ball was filed about: *"an operator with no way to see which
//! entries consent cannot tell a foot that will take a subject from one that
//! will refuse it."* A line that appeared only when true would make its
//! absence ambiguous, so both answers are painted.
//!
//! **The screen carries no control, and that is the surface being honest.**
//! Every other op in REMOTE §5 is a MACHINE's — `advertise`, `invocations` and
//! `complete` are what a tool host speaks, and `invocations` in particular
//! must never be asked by a seat, because asking it DRAINS the foot's queue.
//! What an operator *does* about a tool call happens on the conversation that
//! is making it (§13.7), not on the machine that would run it.
//!
//! **A tool's `input_schema` is not painted.** It is the host's statement to a
//! model, and an operator reading a roster of machines is asking what a box
//! can do, not what shape its arguments take.

use eframe::egui;

use crate::codec::{ClientRow, Machines};
use crate::seat::Snapshot;
use crate::shell::app::Shell;
use crate::shell::mark::Back;

/// The screen's name, the op it asks and the harness's tap target (§15.2).
pub(in crate::shell) const SCREEN: &str = "clients";

impl Shell {
    pub(in crate::shell) fn clients(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        self.note_screen(SCREEN);
        let held = snap
            .clients
            .clone()
            .filter(|machines| machines.about(snap.focus.workspace.as_deref().unwrap_or_default()));
        if self.bar(ui, SCREEN, &Back::To("conversations")) {
            self.close_world();
        }
        super::super::banner(ui, snap);
        ui.separator();
        egui::ScrollArea::vertical()
            .min_scrolled_height(0.0)
            .show(ui, |ui| listed(ui, held.as_ref()));
    }
}

/// The rows, or the sentence saying which absence this is — the pair every
/// read-only surface in this app paints (§13.9).
fn listed(ui: &mut egui::Ui, held: Option<&Machines>) {
    let Some(machines) = held else {
        ui.weak("nothing read yet");
        return;
    };
    if machines.rows.is_empty() {
        ui.weak("no machines here");
    }
    for row in &machines.rows {
        machine(ui, row);
    }
}

/// One machine: what it is called, whether it was connected when the engine
/// answered, and what it says it offers.
fn machine(ui: &mut egui::Ui, row: &ClientRow) {
    let seen = if row.present {
        "connected"
    } else {
        "not connected — a busy host holds no connection either"
    };
    ui.label(format!("{} · {seen}", row.client));
    if row.tools.is_empty() {
        ui.weak("offers nothing");
    }
    for tool in &row.tools {
        let consent = if tool.subject_cwd {
            "takes the conversation's directory"
        } else {
            "runs where this machine runs things"
        };
        ui.weak(format!("{} · {consent}", tool.name));
        if !tool.description.is_empty() {
            ui.weak(&tool.description);
        }
    }
    ui.add_space(4.0);
}
