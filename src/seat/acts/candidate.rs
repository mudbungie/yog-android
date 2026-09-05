//! **The three acts on an obligation** (DESIGN §13.12): spread it over n
//! candidates, accept one, release the rest. Its own file for `acts/ball.rs`'s
//! reason — what these address is a ball in a project, and one of them is a
//! chain rather than a gesture.
//!
//! **A fan is the start with n in the middle**, not a second start path
//! (lernie DESIGN §4.36). It stages a conversation the way `started` does, and
//! then hands the prepared body to `fan` with the count; what comes back is
//! one prepared body per candidate, and **the seat fires them**. A candidate
//! prepared and never fired is a worktree balls made for nothing, so firing is
//! the completion of the act rather than a convenience the operator is offered
//! afterwards.
//!
//! **What that forfeits is stated rather than hidden**: upstream's terminal
//! fires each candidate itself *"with whatever variation you want between
//! them"*, and this seat fires n with one goal. Per-candidate variation is a
//! surface — n fields, or one edited n times — and it arrives with the ball
//! that builds it; what it is not is a reason to leave n worktrees empty.
//!
//! **Nothing here is armed** (lernie §4.36, transferring §13.8's test): an
//! arming is for an act whose product is that its subject is gone. `deliver`
//! advances a ref by the ordinary recursive delivery — git holds what it
//! moved, the ball is not closed. `retire` releases a worktree and changes no
//! delivery target; whether the source ref goes with it is this project's own
//! declared retention acting, and the receipt says which way it went, so the
//! seat PAINTS that answer rather than predicting a policy it has not read.
//!
//! **And never a resend.** A repeated fan is n more worktrees and a repeated
//! deliver is a second delivery, so a lost reply is in doubt and the read that
//! settles it is the candidates listing itself.

use crate::codec::reply::Reply;
use crate::codec::{Act, CandidateAct, Gesture, Prepared, encode};
use crate::transport::Seat;

use super::super::Focus;
use super::super::pass::kind_err;
use super::super::posted::{Posted, faulted};

/// Post one handle act on the obligation the row named — answered — and **two receipts worth a sentence on success**,
/// which is `acts::held`'s rule at a second site: the banner says the one
/// thing an operator cannot see by looking at the screen they are on.
///
/// A retirement always releases the worktree, and whether the source ref went
/// with it is this project's own declared retention acting — a policy set
/// elsewhere, invisible on this listing, and stated by the engine rather than
/// predicted here. A delivery that landed no commit is the other: the ref did
/// not move, and a silent success would report a delivery that did not happen.
pub(crate) fn candidate(seat: &Seat, project: String, ball: String, act: CandidateAct) -> Posted {
    let op = act.op();
    let gesture = Gesture::Act(Act::Candidate { project, ball, act });
    match seat.answered(&encode(&gesture)) {
        Ok(Reply::Delivered(landed)) if landed.commit.is_empty() => Posted::Refused(format!(
            "delivered onto {}: nothing landed — the source ref moved nothing",
            landed.target
        )),
        Ok(Reply::Delivered(_)) => Posted::Took,
        Ok(Reply::Retired { discarded }) => Posted::Refused(retired(discarded)),
        Ok(other) => Posted::Refused(kind_err(op, &other)),
        Err(why) => faulted(&why, op, LISTING),
    }
}

/// Which way this project's retention went, in the engine's terms.
fn retired(discarded: bool) -> String {
    let ref_ = if discarded { "discarded" } else { "kept" };
    format!("retired: the worktree is released and its source ref is {ref_}")
}

/// **The fan, whole**: stage, spread, then fire each candidate with the goal.
///
/// The first failure is the answer. A spread that materialized worktrees and
/// then failed to fire one leaves the rest unfired, and the sentence says
/// which gesture stopped — the listing is what shows what actually exists,
/// which is the read every act here names.
pub(crate) fn spread(
    seat: &Seat,
    focus: &Focus,
    project: String,
    ball: String,
    n: usize,
    goal: String,
) -> Posted {
    let Some(workspace) = focus.workspace.clone() else {
        return Posted::Refused("fan: no workspace is focused".to_owned());
    };
    let staged = match seat.answered(&encode(&Gesture::Act(Act::Prepare { workspace }))) {
        Ok(Reply::Prepared(prepared)) => prepared,
        Ok(other) => return Posted::Refused(kind_err("fan", &other)),
        Err(why) => return faulted(&why, "fan", LISTING),
    };
    let gesture = Gesture::Act(Act::Fan {
        project,
        ball,
        prepared: staged,
        n,
    });
    let rows = match seat.answered(&encode(&gesture)) {
        Ok(Reply::Fanned(rows)) => rows,
        Ok(other) => return Posted::Refused(kind_err("fan", &other)),
        Err(why) => return faulted(&why, "fan", LISTING),
    };
    for prepared in rows {
        if let Err(stopped) = fired(seat, prepared, goal.clone()) {
            return stopped;
        }
    }
    Posted::Took
}

/// One candidate fired, with the operator's own goal — `acts::started`'s
/// second gesture, said n times. The prepared body is the engine's, rebound to
/// this candidate, and rides through whole; the goal is the operator's, said
/// once and given to every candidate, which is exactly the variation this seat
/// forfeits and says so.
fn fired(seat: &Seat, prepared: Prepared, goal: String) -> Result<Posted, Posted> {
    match seat.answered(&encode(&Gesture::Act(Act::Prompt { prepared, goal }))) {
        Ok(Reply::Started { .. } | Reply::Outcome { ok: true, .. }) => Ok(Posted::Took),
        Ok(Reply::Outcome { stderr, .. }) => Err(Posted::Refused(format!("fan refused: {stderr}"))),
        Ok(other) => Err(Posted::Refused(kind_err("fan", &other))),
        Err(why) => Err(faulted(&why, "fan", LISTING)),
    }
}

/// The read that settles a doubted one — and the one the screen makes anyway:
/// the listing is re-read after every act, so what exists is what the next
/// frame paints.
const LISTING: &str = "The candidates are listed again; what they say is what exists.";
