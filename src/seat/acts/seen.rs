//! **The act that answers an attention mark** (yog §8.5, DESIGN §13.8): the
//! watermark that takes one conversation's row off the queue.
//!
//! Its own file for `acts/row.rs`'s reason, one step further along. A row act
//! takes its agent from the row and its workspace from the focus, because a
//! conversation list is only ever painted under one workspace. **The queue is
//! not**: it spans every workspace this seat can see (REMOTE §14.1), so a
//! queue row carries both halves of its own address and neither may come from
//! where the operator happens to be standing.
//!
//! **Three fates and never a resend**, as `seat::acts` states it. A repeated
//! `seen` would discard nothing the first one left — the watermark is a
//! position, not an increment — and it is still not repeated, for the reason
//! `clear_trail` records beside its own harmless repeat: the rule is the
//! contract's rather than the act's, and an exception for the case that looks
//! safe is how a rule stops being one.

use crate::codec::reply::Reply;
use crate::codec::{Act, Gesture, encode};
use crate::transport::Seat;

use super::super::pass::kind_err;
use super::super::posted::{Posted, faulted};

/// Acknowledge the conversation one queue row names.
///
/// The receipt carries the queue that remains and this seat keeps none of it:
/// the lane is the queue's one writer here (DESIGN §14.1), and it states the
/// same answer again the moment the write lands. `codec::reply` argues that
/// where the shape is read.
pub(crate) fn seen(seat: &Seat, workspace: String, agent: String) -> Posted {
    let act = Act::Seen { workspace, agent };
    match seat.answered(&encode(&Gesture::Act(act))) {
        Ok(Reply::Acknowledged(_)) => Posted::Took,
        Ok(other) => Posted::Refused(kind_err("seen", &other)),
        Err(why) => faulted(&why, "seen", QUEUE),
    }
}

/// The read that settles a doubted `seen` — and the one that is already
/// standing: the attention lane restates the whole queue whenever it changes,
/// so a row that is still there is a row that was not acknowledged.
const QUEUE: &str = "The queue's next frame says whether that row is still waiting.";
