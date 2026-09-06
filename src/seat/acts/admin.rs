//! **The admin surface's five acts** (DESIGN §13.17): write a config file,
//! mark a workspace's task branch, flush its inbox, and the two deletions.
//!
//! **Three receipts for five acts**, read off the act rather than guessed at
//! the reply — `acts::row`'s rule at a second site, so an engine answering the
//! other shape is a named refusal instead of a silent success. A config write
//! answers the bare `applied`; `marks` answers the branch it landed on, which
//! is the engine's own re-read and the reason nothing here asks again; a scan
//! answers the `outcome` of what it ran; and both deletions answer `deleted`,
//! which carries nothing because what a deletion DID is that its subject is
//! gone.
//!
//! **Never sent twice, and the two deletions are why that matters most.** A
//! repeated delete finds nothing and refuses, which is harmless; a repeated
//! config write re-applies bytes the operator may have edited since. So each
//! names the read that settles it and none is re-sent (`seat::posted`).

use crate::codec::reply::Reply;
use crate::codec::{Act, AdminAct, Gesture, encode};
use crate::seat::pass::kind_err;
use crate::transport::Seat;

use super::super::posted::{Posted, faulted};

/// Post one admin act, and hand back the branch a `marks` write landed on —
/// the one receipt of the five that carries a fact this seat paints.
pub(crate) fn admin(seat: &Seat, act: AdminAct) -> (Posted, Option<String>) {
    let op = act.op();
    let settles = settles(&act);
    let gesture = Gesture::Act(Act::Admin(act));
    match seat.answered(&encode(&gesture)) {
        Ok(Reply::Applied | Reply::Deleted | Reply::Outcome { ok: true, .. }) => {
            (Posted::Took, None)
        }
        Ok(Reply::Marks(branch)) => (Posted::Took, Some(branch)),
        Ok(Reply::Outcome { stderr, .. }) => {
            (Posted::Refused(format!("{op} refused: {stderr}")), None)
        }
        Ok(other) => (Posted::Refused(kind_err(op, &other)), None),
        Err(why) => (faulted(&why, op, settles), None),
    }
}

/// **The read that settles a doubted admin act** — the world is the durable
/// record (REMOTE §9.8), and each of these five has a read that shows what
/// became of it.
fn settles(act: &AdminAct) -> &'static str {
    match act {
        AdminAct::Config { .. } => {
            "Tap the destination again — the config read says what the file holds now."
        }
        AdminAct::Marks { .. } => "The marks read says which branch the workspace is on.",
        AdminAct::Scan { .. } => {
            "The conversation's records say what mail is still undelivered. A scan \
             delivers what is there and nothing else, so a repeat would deliver \
             nothing twice."
        }
        AdminAct::DeleteAgent { .. } => {
            "The workspace's conversation list says whether it is gone, and the next \
             pass re-reads it."
        }
        AdminAct::DeleteWorkspace { .. } => {
            "The workspace roster says whether it is gone, and the next pass re-reads it."
        }
    }
}
