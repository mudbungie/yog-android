//! **The two world surfaces** (DESIGN §13.8, bl-35bd): the attention queue as
//! a queue, and the ops trail with the two acts over it.
//!
//! **They are reached by verb, from any prompt line** (§22 items 5 and 7,
//! bl-3edc): `/attention` and `/trail`. They sat at the top depth for the
//! search's reason (§13.6) — neither read names a workspace or a conversation
//! — and the verb keeps that reason: a sheet row names no scope either.
//!
//! **The trail is asked for when it is opened, and the queue is never asked
//! at all.** The trail is read by nothing standing, so a surface nobody has
//! opened costs this device no radio — the §14.1 lane's own argument applied
//! at the seat. The queue is that lane's (DESIGN §14.1): it stands for the
//! seat's whole life and its frames write the one holder every screen paints
//! from, so opening this surface is a look at what is already held.
//!
//! **The queue's rows navigate; the trail's do not.** A queue row is an
//! address this seat already focuses (the workspace and the agent, in the
//! words every gesture takes), so tapping one opens that conversation, exactly
//! as a search hit does. A trail row addresses nothing: it is a line of the
//! record, and nothing this device could open would be *that action* — so it
//! paints and does not tap, the same answer a ball hit gets on the search
//! screen.
//!
//! **`clear-trail` is the first armed control in this app** (§13.2's *tap is
//! the act* has an exception at last). Every gesture the seat had until now
//! kept what it acted on: an interrupt keeps what is committed, a stop leaves
//! the transcript, a refused call stays parked. This one DISCARDS a durable
//! record, and the record is the thing every other recovery sentence in this
//! client points at (REMOTE §9.8). The arm is two taps on one control, spelled
//! in the control's own label rather than in a dialog: a phone's back gesture
//! must dismiss anything modal, and a confirmation that a back press can
//! answer is a confirmation nobody read.

use eframe::egui;

use crate::overlay::Overlay;
use crate::seat::Snapshot;
use crate::shell::app::Shell;

mod admin;
mod balls;
mod candidates;
mod clients;
mod fleet;
mod help;
mod signin;
mod work;

pub(in crate::shell) use candidates::FLOOR;
pub(in crate::shell) use fleet::FLOOR as CAP;
mod trail;
mod waiting;

impl Shell {
    /// Paint the standing overlay's body, inside its frame. One arm each, and
    /// each names its own screen (`app/probe.rs` — the name lives at the
    /// branch, never derived a second time from the same state).
    pub(super) fn overlaid(&mut self, ui: &mut egui::Ui, snap: &Snapshot, overlay: Overlay) {
        let workspace = snap.focus.workspace.clone().unwrap_or_default();
        let agent = snap.focus.agent.clone().unwrap_or_default();
        match overlay {
            Overlay::Queue => self.waiting(ui, snap),
            Overlay::Trail => self.trail(ui, snap),
            Overlay::Balls(view) => self.balls(ui, snap, view),
            Overlay::Candidates => self.candidates(ui, snap),
            Overlay::Fleet => self.fleet(ui, snap),
            Overlay::Clients => self.clients(ui, snap),
            Overlay::Help => self.help(ui, snap),
            Overlay::Work => self.work(ui, snap),
            Overlay::Admin => self.admin(ui, snap),
            Overlay::SignIn => self.signin(ui, snap),
            Overlay::Records => self.records(ui, snap, &workspace, &agent),
            Overlay::Files => self.files(ui, snap, &workspace, &agent),
            Overlay::Search => self.searched(ui, snap),
            Overlay::Switcher => self.switcher(ui, snap),
        }
    }
}
