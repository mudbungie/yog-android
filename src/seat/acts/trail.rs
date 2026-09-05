//! **The two acts over the ops trail** (yog DESIGN §4.2, §7.3; this repo's
//! §13.8) — the acknowledgement and the truncation, in their own file for the
//! reason the two beside it have one: what they address is not a conversation
//! and not a workspace, it is the engine's own record, and a file about
//! gestures aimed at the world reads differently from a file about gestures
//! aimed at a place.
//!
//! Neither names a row. The ack is a watermark over the trail as it stands and
//! the clear is a truncation of it, so there is nothing for a client to select
//! and nothing for it to select wrongly.

use crate::codec::reply::Reply;
use crate::codec::{Act, Gesture, encode};
use crate::transport::Seat;

use super::super::pass::kind_err;
use super::super::posted::{Posted, faulted};

/// **Acknowledge the trail's alarms** (yog §4.2, §7.3, DESIGN §13.8): the
/// watermark, moved to the trail as it stands. It names no row, so there is
/// nothing to select and nothing to select wrongly, and the receipt carries
/// nothing.
pub(crate) fn ack(seat: &Seat) -> Posted {
    match seat.answered(&encode(&Gesture::Act(Act::Ack))) {
        Ok(Reply::Acked) => Posted::Took,
        Ok(other) => Posted::Refused(kind_err("ack", &other)),
        Err(why) => faulted(&why, "ack", TRAIL),
    }
}

/// **Truncate the trail** (yog §4.2). The one act this seat sends that
/// discards a durable record; the arming is the control's (§13.8) and nothing
/// about it crosses.
///
/// **A doubted clear is the one act here whose repeat is harmless and is still
/// not repeated.** Clearing twice would discard nothing the first clear left,
/// so the temptation is to re-send — but the rule is the contract's and not
/// the act's (REMOTE §3, bl-07b1): nothing in this file ever sends an act
/// again, and an exception for the case that looks safe is how the rule stops
/// being one.
pub(crate) fn clear_trail(seat: &Seat) -> Posted {
    match seat.answered(&encode(&Gesture::Act(Act::ClearTrail))) {
        Ok(Reply::TrailCleared) => Posted::Took,
        Ok(other) => Posted::Refused(kind_err("clear-trail", &other)),
        Err(why) => faulted(&why, "clear-trail", TRAIL),
    }
}

/// The read that settles either of the two trail acts: the trail itself,
/// which the screen they are fired from re-reads.
const TRAIL: &str = "The trail says what it stands at when it is read again.";
