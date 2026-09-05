//! **The five acts on a ball** (DESIGN §13.9, bl-f36e) — the act half of the
//! pane whose reads landed in bl-d587. Its own file for `acts/row.rs`'s
//! reason: what these address is neither a conversation nor a workspace, it
//! is a ball in a project, and the address they carry says so.
//!
//! **The `--as` stamp is the WORKSPACE's name** (lernie DESIGN §4.35, whose
//! ruling transfers whole): yog binds a ball to a workspace on exactly that
//! equality, so a seat that invented an operator name would break the binding
//! it was making. Here it is the focused workspace, which is also the only
//! one the pane that fires these is ever painted under.
//!
//! **Nothing new is decoded and no receipt is invented.** All five answer with
//! a captured run, which `deposit` already reads the same way: a refusal
//! arrives in the engine's own words on the banner, and a success says
//! nothing. That silence is the pane's own read paying for itself — the act is
//! followed by a re-read of the view it was fired on, and a ball filed, moved
//! or closed is a row that changed there. A receipt here would be a second,
//! worse statement of what the pane is about to show.
//!
//! **And never a resend**, like every act in this crate: a repeated `close` is
//! a second close, a repeated `create` is a second ball. The read that settles
//! a doubted one is the pane's own.

use crate::codec::reply::Reply;
use crate::codec::{Act, BallAct, Gesture, encode};
use crate::transport::Seat;

use super::super::pass::kind_err;
use super::super::posted::{Posted, faulted};

/// Post one ball act, stamped with the workspace the pane is painted under.
pub(crate) fn ball(seat: &Seat, project: String, name: String, act: BallAct) -> Posted {
    let op = act.op();
    let gesture = Gesture::Act(Act::Ball { project, name, act });
    match seat.answered(&encode(&gesture)) {
        Ok(Reply::Outcome { ok: true, .. }) => Posted::Took,
        Ok(Reply::Outcome { stderr, .. }) => Posted::Refused(format!("{op} refused: {stderr}")),
        Ok(other) => Posted::Refused(kind_err(op, &other)),
        Err(why) => faulted(&why, op, PANE),
    }
}

/// The read that settles a doubted one — and the one the pane makes anyway:
/// the view is re-read after every act, so what the store now holds is what
/// the next frame paints.
const PANE: &str = "The pane is read again; what it lists is what the store holds.";
