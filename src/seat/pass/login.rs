//! **The third held lane** (DESIGN §14.1, §13.19): what this seat is
//! following of one provider's sign-in, and what a frame of it does to the
//! standing. Its own file rather than three fields on [`Standing`], because
//! the three are one fact — *which run the glass is watching, and everything
//! that run has said* — and a lane's stop condition is a decision this ball
//! had to make rather than a field it could add.
//!
//! **The watch is the glass's, the end is the run's.** Opening the sign-in
//! screen on a row is what makes a lane wanted; the run settling is what makes
//! it unwanted, and both halves are needed. Without the first, a lane would
//! stand on a provider nobody is looking at; without the second, a seat would
//! redial a FINISHED sign-in every pass forever — the engine's lane ends at
//! the outcome frame, so the pass that reopened it would replay a settled
//! buffer and end again, once per cadence, for as long as the screen was open.
//!
//! **A frame replaces or appends by its LANE's id, never by its subject.**
//! The engine hands each read the lines it has not sent *to that read*, so a
//! lane that expired at its hold and was redialled by the next pass replays
//! the whole buffer from zero (REMOTE §8.3). Folding that onto what stands
//! would show every line twice. So the first frame of a lane this fold has
//! not seen REPLACES it and every later frame of that same lane appends —
//! which is also why the act's receipt can seed the fold with no special
//! case: it is the same value the lane's first frame will carry.

use crate::codec::LoginView;
use crate::seat::lane::Subject;
use crate::seat::{Focus, Signing};

/// The sign-in this seat is following.
#[derive(Default)]
pub(in crate::seat) struct SignIn {
    /// Which provider the glass has open, or none. Set by the screen and by
    /// the act, cleared when the screen is left.
    watching: Option<String>,
    /// The lane whose frames the fold below is following — `None` before one
    /// has spoken, so the next frame from any lane replaces rather than
    /// appends.
    lane: Option<u64>,
    view: LoginView,
}

impl SignIn {
    /// **Watch one provider's run, or nothing.** A subject that moved drops
    /// the fold with it: one provider's lines under another's name are the
    /// same wrong claim as one focus's rows under another's.
    pub(in crate::seat) fn watch(&mut self, provider: Option<String>) {
        if self.watching != provider {
            self.lane = None;
            self.view = LoginView::default();
        }
        self.watching = provider;
    }

    /// **The act's receipt, adopted.** It is the run's standing at the moment
    /// it started, so it seeds the fold whole — and it names its own provider,
    /// because firing the act is also asking to watch it.
    pub(in crate::seat) fn started(&mut self, provider: String, view: LoginView) {
        self.watch(Some(provider));
        self.view = view;
    }

    /// **One lane frame, folded** — replaced when the lane is new to this
    /// fold, appended when it is the one already being followed.
    pub(in crate::seat) fn framed(&mut self, id: u64, view: LoginView) {
        if self.lane == Some(id) {
            self.view.absorb(view);
        } else {
            self.lane = Some(id);
            self.view = view;
        }
    }

    /// The lane a pass wants standing: one while a provider is watched inside
    /// the focused workspace and its run has not settled.
    pub(in crate::seat) fn wanted(&self, focus: &Focus) -> Option<Subject> {
        let (workspace, provider) = (focus.workspace.clone()?, self.watching.clone()?);
        (!self.view.settled()).then_some(Subject::Login {
            workspace,
            provider,
        })
    }

    /// What the frame paints, under the provider it belongs to.
    pub(in crate::seat) fn painted(&self) -> Option<Signing> {
        Some(Signing {
            provider: self.watching.clone()?,
            view: self.view.clone(),
        })
    }
}
