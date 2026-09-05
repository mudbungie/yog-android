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
    /// **The ball pane at one of its three views** (§13.9). The view rides on
    /// the navigation rather than on the answer, so a screen names itself from
    /// what was opened and paints only the answer that belongs under it.
    Balls(crate::codec::View),
}

mod balls;
mod trail;
mod waiting;

impl Shell {
    /// Open one of them. For the trail, opening IS the ask — an operator
    /// opening the surface is the request for its rows; the queue is held
    /// standing and asks nothing.
    pub(super) fn open_world(&mut self, world: World) {
        self.opened = Some(world);
        self.armed = false;
        // A ball picked on one visit is not still picked on the next: the act
        // controls address a row, and a row nobody can see is not one.
        self.ball = None;
        let Some(model) = self.model() else { return };
        // **Opening IS the ask**, for the trail and for the ball pane alike:
        // both are read by nothing standing, so a surface nobody has opened
        // costs this device no radio at all (§14.1's argument, at the seat).
        // The queue is the exception and asks nothing — it is the lane's.
        match world {
            World::Trail => model.list_trail(),
            World::Balls(view) => model.list_balls(view),
            World::Queue => (),
        }
    }

    /// **The one control that reaches one of the pane's reads**, painted where
    /// that read's subject is: `balls` and `board` on the roster, because
    /// neither names a workspace, and `workspace-balls` on a workspace's own
    /// conversation list. The name the harness taps it by is the op's own
    /// (§15.2), which is also the screen it opens.
    pub(in crate::shell) fn balls_entry(&mut self, ui: &mut egui::Ui, view: crate::codec::View) {
        let control = super::tap(ui, view.screen().into(), view.screen());
        self.note_control(view.screen(), ui, control.rect);
        if control.clicked() {
            self.open_world(World::Balls(view));
        }
    }

    /// Paint whichever is open. One arm each, and each names its own screen
    /// (`app/probe.rs` — the name lives at the branch, never derived a second
    /// time from the same state).
    pub(super) fn world(&mut self, ui: &mut egui::Ui, snap: &Snapshot, world: World) {
        match world {
            World::Queue => self.waiting(ui, snap),
            World::Trail => self.trail(ui, snap),
            World::Balls(view) => self.balls(ui, snap, view),
        }
    }

    /// Leave, back to the roster. The arm goes with it: an armed control the
    /// operator walked away from is not still armed when they come back.
    fn close_world(&mut self) {
        self.opened = None;
        self.armed = false;
        self.ball = None;
    }
}
