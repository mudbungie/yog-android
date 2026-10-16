//! The screens above the transcript: the bootstrap gate, the foot's standing,
//! the chat in start mode, and the overlay standing over any of them — the
//! switcher among them (DESIGN §22 item 7). Pure presentation over the
//! model's snapshot — every wire crossing already happened on the model's
//! worker thread, and a tap is a command sent, never a call waited on.
//!
//! The transcript is the third depth and lives in `transcript.rs`: it is the
//! only screen with its own mechanics (the projected rows, the fold overrides,
//! the composer riding the keyboard) rather than a list of taps.

use eframe::egui;

use super::app::Shell;
use super::boot::Running;

use super::mark::Back;
use super::theme::{self, ink};
use crate::host::Health;
use crate::overlay::{Depth, Overlay, Under, standing};
/// **The attention mark**, re-exported from where it lives (`crate::roster`,
/// bl-4d17). It moved because the conversation row's words are composed in
/// that module and the mark sits in the middle of a row's first line; the
/// reasoning for the glyph itself travelled with it.
pub(super) use crate::roster::ATTENTION_MARK;
use crate::seat::Snapshot;
use crate::theme::State;

mod banner;
mod files;
mod preview;
mod records;
mod rows;
mod search;
mod world;

pub(super) use banner::banner;
pub(super) use rows::threaded;

pub(in crate::shell) use world::{CAP, FLOOR};

impl Shell {
    /// Everything below the top inset: the yog mark, then the component this
    /// launch is running, and then the screen its focus depth selects.
    ///
    /// **The outermost branch is the configuration surface** (bl-387f), and
    /// the bootstrap gate (yog bl-15bd) is its forced-open case: a cold
    /// device paints the three offers because nothing else exists, and a
    /// provisioned one paints them because the mark was tapped — one
    /// surface, two ways in. A foot otherwise paints what it is hosting,
    /// because a foot may not ask the world anything (REMOTE §4.2) and
    /// painting it a chat screen would be an app promising a surface its own
    /// certificate refuses.
    pub(crate) fn screens(&mut self, ui: &mut egui::Ui) {
        if self.settings || matches!(self.running, Running::Cold { .. }) {
            self.configuration(ui);
            return;
        }
        if matches!(self.running, Running::Foot { .. }) {
            self.note_screen("foot");
            self.bar(ui, &crate::bootstrap::Component::Foot.brand(), &Back::None);
            self.foot(ui);
            return;
        }
        let Some(snap) = self.model_mut().map(crate::seat::Model::snapshot) else {
            return;
        };
        // The outbox settles once per frame, whatever screen is up: an echo
        // whose conversation the operator left is not an echo any more
        // (bl-66fb).
        self.settle_echo(&snap);
        // **An overlay floats over whatever depth the focus is at** (DESIGN
        // §22 item 6): one stands at a time, the focus underneath is left
        // exactly where it was, and nothing under it paints — the composer
        // included, so its field comes off the glass.
        let depth = match (&snap.focus.workspace, &snap.focus.agent) {
            (None, _) => Depth::Roster,
            (Some(_), None) => Depth::Workspace,
            (Some(_), Some(_)) => Depth::Conversation,
        };
        if let Some(overlay) = standing(self.overlay, depth, snap.search.is_some()) {
            self.floating(ui, &snap, overlay);
            return;
        }
        // **Nothing focused never reaches here**: the switcher stands over it
        // (`crate::overlay::standing`). The two depths left are the chat —
        // in start mode, and on a conversation — and back from either opens
        // the switcher (§22 item 7).
        //
        // **They anchor controls to the floor and paint their own bar**
        // (bl-192c): the acts and the composer claim from the floor and the
        // chrome takes what is left above them, so the bar is painted inside
        // that remainder rather than before it.
        match (snap.focus.workspace.clone(), snap.focus.agent.clone()) {
            (Some(workspace), None) => {
                self.note_screen("start");
                self.start(ui, &snap, &workspace);
            }
            (Some(workspace), Some(agent)) => {
                self.note_screen("transcript");
                self.transcript(ui, &snap, &workspace, &agent);
            }
            (None, _) => {}
        }
    }

    /// **The standing overlay, in its frame**: the name line from
    /// `crate::overlay`, and the screen's own body under it.
    fn floating(&mut self, ui: &mut egui::Ui, snap: &Snapshot, overlay: Overlay) {
        let agent = snap.focus.agent.clone().unwrap_or_default();
        let under = Under {
            workspace: snap.focus.workspace.clone().unwrap_or_default(),
            conversation: super::chat::speaker_of(snap, &agent),
            needle: snap
                .search
                .as_ref()
                .map(|found| found.needle.clone())
                .unwrap_or_default(),
        };
        let line = overlay.line(snap.minted.is_some(), &under);
        self.float(ui, &line, |shell, ui| shell.overlaid(ui, snap, overlay));
    }

    /// The foot's whole screen: what this machine offers and what it has run.
    /// There is nothing else to paint, and that is the component working —
    /// §4.2's *"a foot cannot ask about the world"* is the sentence, and an
    /// empty roster would be this app asking anyway and hiding the refusal.
    fn foot(&mut self, ui: &mut egui::Ui) {
        ui.weak(self.identity());
        ui.separator();
        Self::hosting(ui);
        ui.add_space(8.0);
        ui.weak(
            "Thrall advertises what this machine can run, waits for work \
             addressed to it, and hands back what happened. It says nothing \
             else about the world — mint this device a Lernie (operator-grade) \
             leaf to seat it instead.",
        );
    }

    /// What this device offers a session, one line (REMOTE §5). It rides the
    /// switcher's seat panel because that is the screen every back lands on,
    /// and a tool host nobody can see is one nobody can tell has stopped.
    ///
    /// It takes no `self`: since bl-8bd0 the host belongs to the PROCESS and
    /// this reads `state::standing()`, so a receiver here would be a claim
    /// that the frame owns the fact.
    pub(super) fn hosting(ui: &mut egui::Ui) {
        let Some(standing) = crate::state::standing() else {
            return;
        };
        // Health first: a host that is climbing back says so with the
        // sentence that broke the channel, rather than showing the last tool
        // it ran as though it were still there (bl-8641).
        let line = match (&standing.health, &standing.last) {
            (Health::Stopped(why), _) => format!("tools stopped: {why}"),
            (Health::Redialling(why), _) => format!("tools: reconnecting… · {why}"),
            (Health::Serving, Some(last)) => format!(
                "tools: {} · served {} · {last}",
                standing.tools.join(", "),
                standing.served
            ),
            (Health::Serving, None) if standing.advertised => {
                format!("tools: {} · waiting", standing.tools.join(", "))
            }
            (Health::Serving, None) => "tools: presenting…".to_owned(),
        };
        // The error accent is for the one that will not mend itself. A
        // redial is ordinary on a phone and reads as ordinary; the word
        // carries it.
        if matches!(standing.health, Health::Stopped(_)) {
            ui.colored_label(ink(State::Error), line);
        } else {
            ui.weak(line);
        }
        // **A disarming that healed itself is still worth a sentence**
        // (REMOTE §5.1, bl-cc54): the set this device offers was replaced
        // while it was running a tool, and the host put it back. Yellow and
        // not red, by the rule above — it HAS mended itself — but not weak
        // either, because two processes claiming one device's name is
        // something only an operator can end. The words are
        // `host::RESTORED`'s; this line only says how many times.
        if standing.restored > 0 {
            ui.colored_label(
                ink(State::Annotation),
                format!("{} (×{})", crate::host::RESTORED, standing.restored),
            );
        }
    }

    pub(super) fn focus_workspace(&self, workspace: Option<String>) {
        if let Some(model) = self.model() {
            model.focus_workspace(workspace);
        }
    }
}

/// One full-width list row at the §13.2 touch floor. Every navigation list
/// paints its rows through this, so the floor is a fact of the helper rather
/// than a discipline at each site — and so is the parity tag: `op` is the read
/// this row's tap reaches (PARITY §2, *"the owed interactable for a read is
/// the affordance that reaches the view it populates"*), which is the one
/// thing that differs between the two lists. The row itself is the
/// language's (`shell::theme::row`): left-aligned, outline-free, a tint only
/// under a thumb.
pub(super) fn tap(
    ui: &mut egui::Ui,
    label: egui::WidgetText,
    op: &str,
    help: &str,
) -> egui::Response {
    let response = theme::row(ui, label, help);
    super::act::act(ui, &response, op);
    response
}
