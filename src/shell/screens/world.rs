//! **The two world surfaces** (DESIGN §13.8, bl-35bd): the attention queue as
//! a queue, and the ops trail with the two acts over it.
//!
//! **They sit at the top depth for the search's reason** (§13.6). Neither read
//! names a workspace or a conversation — the queue is every workspace's and
//! the trail is the engine's — so the screen that reaches them is the one
//! where the whole world is already what is on the glass. An entry on a
//! workspace's conversation list would say the trail was that workspace's,
//! which is a scope the wire does not carry and §8 forbids this app to imply.
//!
//! **Each is asked for when it is opened, and not on the pass.** The queue is
//! already read at the conversation depth for the held-call band (§13.7) and
//! the trail is read by nothing standing, so a surface nobody has opened costs
//! this device no radio at all — which is the §14.1 lane's own argument
//! applied at the seat.
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

use crate::seat::Snapshot;
use crate::shell::app::Shell;

/// Which world surface is open over the roster. Navigation and nothing else —
/// no more durable than a scroll position — for `Shell::settings`' reason
/// exactly: what this device IS is derived from the leaf on disk, and what is
/// on the glass is not stored anywhere.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum World {
    /// Everything waiting on the operator, across every workspace.
    Queue,
    /// What the engine last did.
    Trail,
}

mod trail;
mod waiting;

impl Shell {
    /// Open one of them, and ask for what it paints in the same gesture: an
    /// operator opening a surface IS the request for its rows.
    pub(super) fn open_world(&mut self, world: World) {
        self.opened = Some(world);
        self.armed = false;
        if let Some(model) = self.model() {
            match world {
                World::Queue => model.list_queue(),
                World::Trail => model.list_trail(),
            }
        }
    }

    /// Paint whichever is open. One arm each, and each names its own screen
    /// (`app/probe.rs` — the name lives at the branch, never derived a second
    /// time from the same state).
    pub(super) fn world(&mut self, ui: &mut egui::Ui, snap: &Snapshot, world: World) {
        match world {
            World::Queue => self.waiting(ui, snap),
            World::Trail => self.trail(ui, snap),
        }
    }

    /// Leave, back to the roster. The arm goes with it: an armed control the
    /// operator walked away from is not still armed when they come back.
    fn close_world(&mut self) {
        self.opened = None;
        self.armed = false;
    }
}
