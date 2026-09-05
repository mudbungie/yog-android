//! **Answering the parked tool call** (yog §8.6; DESIGN §13.7, bl-b39d).
//!
//! Its own file beside `row.rs` for the opposite reason that one exists: a row
//! act names the row that was pressed, and this one names the FOCUS — you
//! answer the call in the conversation you are reading, because reading what
//! is about to happen is the whole of deciding. Nothing about a held call can
//! be answered from a list.
//!
//! **The verdict is the only parameter, and the call is not one.** The engine
//! reads the held mark off the branch at fire time, so this gesture cannot
//! land on a call that is no longer the one held — which is also why nothing
//! here carries the `tool_use` the queue read gave it.
//!
//! **One receipt, and one sentence worth saying about a success.** A releasing
//! verdict that did not advance the branch left the answer recorded and the
//! conversation exactly where it was: nothing will move until something
//! advances it, and that is the one outcome an operator cannot see by looking
//! at the screen they are on.

use crate::codec::reply::Reply;
use crate::codec::{Act, Answered, Gesture, Verdict, encode};
use crate::seat::pass::kind_err;
use crate::transport::Seat;

use super::super::Focus;
use super::super::posted::{Posted, faulted};

/// Answer the call parked at the focused conversation.
pub(crate) fn answer(seat: &Seat, focus: &Focus, verdict: Verdict) -> Posted {
    let Focus {
        workspace: Some(workspace),
        agent: Some(agent),
    } = focus.clone()
    else {
        return Posted::Refused("answer: no conversation is focused".to_owned());
    };
    let act = Act::Answer {
        workspace,
        agent,
        verdict,
    };
    match seat.answered(&encode(&Gesture::Act(act))) {
        Ok(Reply::Answered(answered)) => took(&answered),
        Ok(other) => Posted::Refused(kind_err("answer", &other)),
        Err(why) => faulted(&why, "answer", HELD),
    }
}

/// What a receipt means. The engine took it either way — this decides whether
/// there is anything left to say about what it did.
fn took(answered: &Answered) -> Posted {
    if answered.verdict.releases() && !answered.advanced {
        return Posted::Refused(format!(
            "answer {}: recorded on {}, but the conversation was not driven on — nothing \
             moves until it is",
            answered.verdict.word(),
            answered.tool
        ));
    }
    Posted::Took
}

/// The read that settles a doubted answer: the queue this seat re-reads every
/// pass, whose row carries the parked call for exactly as long as it is
/// parked. A call that was answered is a call the next queue read no longer
/// holds, which is the whole recovery — and the reason none of this is ever
/// re-sent.
const HELD: &str = "The conversation's queue row says whether the call is still parked, \
    and the band goes when it is not.";
