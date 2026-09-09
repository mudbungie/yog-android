//! The screens above the transcript: the bootstrap gate, the foot's standing,
//! the workspace roster and the conversation list. Pure presentation over the
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
/// **The attention mark**, re-exported from where it lives (`crate::roster`,
/// bl-4d17). It moved because the conversation row's words are composed in
/// that module and the mark sits in the middle of a row's first line; the
/// reasoning for the glyph itself travelled with it.
pub(super) use crate::roster::ATTENTION_MARK;
use crate::seat::Snapshot;
use crate::theme::State;

mod files;
mod preview;
mod records;
mod rows;
mod search;
mod world;

pub(crate) use files::SCREEN as FILES;
pub(crate) use world::World;
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
        // **The two world surfaces sit over the depths, not inside them**
        // (§13.8): neither read names a workspace or a conversation, so
        // neither belongs to a focus — and the focus is left exactly where it
        // was, which is where backing out of one lands.
        if let Some(world) = self.opened {
            self.settle_echo(&snap);
            self.world(ui, &snap, world);
            return;
        }
        // The outbox settles once per frame, whatever screen is up: an echo
        // whose conversation the operator left is not an echo any more
        // (bl-66fb).
        self.settle_echo(&snap);
        // The bar first, then the error banner under it (§13.2), then the
        // depth's own body. Back walks exactly one focus depth — the bar
        // returns the tap and this match is the one place a depth is spelled.
        match snap.focus.workspace.clone() {
            // **Two screens at this depth, and the search's own file chooses
            // between them** (§13.6, bl-4c2b): the roster, and the hits when
            // an answer is standing over it. The choice is one arm there
            // rather than a third arm here, because both paint the same
            // depth's chrome and only the body differs.
            None => self.top(ui, &snap),
            // **The two screens that anchor controls to the floor paint their
            // own bar** (bl-192c). Everywhere else the bar goes first because
            // nothing below it can be pushed anywhere; here the floor's order
            // is the opposite — the acts and the composer claim from the
            // floor and the chrome takes what is left above them — so the bar
            // is painted inside that remainder rather than before it. It is
            // still the first thing in the screen's body, which is what §13.2
            // says.
            Some(workspace) => match snap.focus.agent.clone() {
                None => {
                    self.note_screen("conversations");
                    self.conversations(ui, &snap, &workspace);
                }
                // **The records screen is a depth of this one** (§13.11):
                // its six reads are about the conversation the transcript is
                // showing, so it opens OVER it and backs out into it, and
                // the focus underneath never moves.
                Some(agent) if self.records => {
                    self.records(ui, &snap, &workspace, &agent);
                }
                // **The files screen is the other depth of the transcript**
                // (§13.15): `files` names the conversation the transcript is
                // showing, so it opens OVER it on the records screen's terms
                // exactly.
                Some(agent) if self.files => {
                    self.files(ui, &snap, &workspace, &agent);
                }
                Some(agent) => {
                    self.note_screen("transcript");
                    self.transcript(ui, &snap, &workspace, &agent);
                }
            },
        }
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

    pub(super) fn roster(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        ui.weak("workspaces");
        ui.weak(self.identity());
        Self::hosting(ui);
        // **The release channel's one affordance** (§20), beside the two
        // other structural things this screen says about the device rather
        // than about the world. It paints nothing when nothing is newer,
        // which is every launch but the one after a release.
        self.update_entry(ui);
        ui.separator();
        self.world_entries(ui, snap);
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for row in &snap.workspaces {
                // The name in ink, the count in weak ink, and the mark — when
                // there is one — in the attention accent: one line, three
                // inks, and the hierarchy is the ink (STYLE.md).
                let mut parts = vec![(row.workspace.clone(), ui.visuals().text_color())];
                if row.attention > 0 {
                    parts.push((ATTENTION_MARK.to_owned(), ink(State::Attention)));
                }
                parts.push((
                    format!("  {} agents", row.agents),
                    ui.visuals().weak_text_color(),
                ));
                let label = theme::line(ui, &parts);
                // Tapping a workspace focuses it, and the focus is what the
                // worker asks `conversations` at.
                if tap(ui, label.into(), "conversations").clicked() {
                    self.focus_workspace(Some(row.workspace.clone()));
                }
            }
        });
    }

    /// What this device offers a session, one line (REMOTE §5). It rides the
    /// roster because that is the screen an operator lands on, and a tool host
    /// nobody can see is one nobody can tell has stopped.
    ///
    /// It takes no `self`: since bl-8bd0 the host belongs to the PROCESS and
    /// this reads `state::standing()`, so a receiver here would be a claim
    /// that the frame owns the fact.
    fn hosting(ui: &mut egui::Ui) {
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

    /// **The way to the two world surfaces** (§13.8), above the workspaces
    /// because neither is one: the queue spans every workspace and the trail
    /// is the engine's own record. The mark on the queue entry is the roster
    /// rows' own reading — *any workspace is waiting on you* — and not a
    /// second count derived here.
    fn world_entries(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        let waiting = snap.workspaces.iter().any(|row| row.attention > 0);
        let mark = if waiting { ATTENTION_MARK } else { "" };
        let control = tap(ui, format!("waiting{mark}").into(), "attention");
        // Where the harness finds them (§15.2): neither carries a node it
        // could be addressed by, and they are the only way to two screens.
        self.note_control("waiting", ui, control.rect);
        if control.clicked() {
            self.open_world(World::Queue);
        }
        let control = tap(ui, "trail".into(), "ops");
        self.note_control("trail", ui, control.rect);
        if control.clicked() {
            self.open_world(World::Trail);
        }
        // The two ball reads that name no workspace (§13.9). They are here for
        // the queue's and the trail's reason exactly: what they are about is
        // everything this seat can see, and this is the screen where that is
        // already what is on the glass.
        self.balls_entry(ui, crate::codec::View::Everywhere);
        self.balls_entry(ui, crate::codec::View::Board);
        // The op table (§13.14). It names no place either — what it is about
        // is the vocabulary — and it costs no wire read at all.
        self.help_entry(ui);
    }

    pub(super) fn focus_workspace(&self, workspace: Option<String>) {
        if let Some(model) = self.model() {
            model.focus_workspace(workspace);
        }
    }
}

/// **What a wire being re-dialled says** (bl-eec1). The lower-case is the
/// banner's own register, and the ellipsis is the whole of the claim: it is
/// happening, it has not failed, and nothing is being asked of the operator.
const RECONNECTING: &str = "reconnecting…";

/// The connection banner: what the worker is standing on, under the bar and
/// above whatever screen it interrupted (§13.2).
///
/// **Two sentences and two states, never one** (bl-eec1). A wire being
/// re-dialled is the WORKING accent, because re-dialling is work; a failure
/// that outlived its grace, and a gesture the engine refused, are the ERROR
/// accent, because they will not mend themselves. The model publishes them as
/// two fields that cannot both be about the wire, so this paints whichever it
/// is handed and decides nothing.
pub(super) fn banner(ui: &mut egui::Ui, snap: &Snapshot) {
    if snap.reconnecting {
        ui.colored_label(ink(State::Working), RECONNECTING);
    }
    if let Some(error) = &snap.error {
        ui.colored_label(ink(State::Error), error);
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
pub(super) fn tap(ui: &mut egui::Ui, label: egui::WidgetText, op: &str) -> egui::Response {
    let response = theme::row(ui, label);
    super::act::act(ui, &response, op);
    response
}
