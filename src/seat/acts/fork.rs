//! **The attempt, posted** (DESIGN §13.16): start a child of the focused
//! conversation at a picked point in its history.
//!
//! Its own file rather than a sixth arm of `acts::row` for `codec::fork`'s
//! reason exactly: the subject is a POINT and not the conversation, and what
//! it carries — a ref, a role, a goal — is nothing the row acts carry. What it
//! shares with them is the fate: **never sent twice.** A fork materializes a
//! worktree and starts a driver, so a repeat is a second child doing the same
//! work, and the read that settles a lost one is the spine — the child hangs
//! on the notch it was born at.

use crate::codec::reply::Reply;
use crate::codec::{Act, Gesture, encode};
use crate::seat::pass::kind_err;
use crate::transport::Seat;

use super::super::Focus;
use super::super::posted::{Posted, faulted};

/// **The read that settles a fork in doubt**: the spine, which is the same
/// screen the gesture was fired from — a child card hangs on the notch it was
/// forked at, so re-opening the records says whether one appeared.
const SETTLES: &str = "The conversation's spine says whether a child appeared at that notch — \
     re-open its records to see. Nothing was sent again: a repeat would be a second child \
     doing the same work.";

/// Post one attempt at the focused conversation, forked from `at`.
pub(crate) fn fork(seat: &Seat, focus: &Focus, at: String, goal: String) -> Posted {
    let Focus {
        workspace: Some(workspace),
        agent: Some(parent),
    } = focus.clone()
    else {
        return Posted::Refused("fork: no conversation is focused".to_owned());
    };
    let gesture = Gesture::Act(Act::Fork {
        workspace,
        parent,
        from: at,
        role: super::WORKER.to_owned(),
        goal,
    });
    match seat.answered(&encode(&gesture)) {
        Ok(Reply::Outcome { ok: true, .. }) => Posted::Took,
        Ok(Reply::Outcome { stderr, .. }) => Posted::Refused(format!("fork refused: {stderr}")),
        Ok(other) => Posted::Refused(kind_err("fork", &other)),
        Err(why) => faulted(&why, "fork", SETTLES),
    }
}
