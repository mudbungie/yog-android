//! **The two armings a workspace carries** (DESIGN §13.13): the drone loop,
//! and the alignment monitor watching what it commits. Its own file for
//! `acts/ball.rs`'s reason — what these address is a workspace, and the answer
//! they earn cannot be read without the gesture that earned it.
//!
//! **The receipt says a state, never a setting.** All four answer
//! `{"kind": "armed", "armed": BOOL}`, so the sentence is composed from the
//! ANSWER and the OP together (`FleetAct::said`) — the reply alone cannot say
//! which of the two settings it is about, and a seat that classified off it
//! would be guessing.
//!
//! **A success says something here, and that is the exception earning its
//! keep** (`acts::held`'s rule). Everywhere else in this crate a success is
//! silent because a read straight after shows what it did; this workspace's
//! loop state lives on the `board` (§13.9), which is a different screen opened
//! from a different depth, so silence would leave an operator with no
//! confirmation at all.
//!
//! **And never a resend.** A repeated `fleet` re-arms a loop that may already
//! have claimed balls, so a lost reply is in doubt and the read that settles
//! it is the board.

use crate::codec::reply::Reply;
use crate::codec::{Act, FleetAct, Gesture, encode};
use crate::transport::Seat;

use super::super::Focus;
use super::super::pass::kind_err;
use super::super::posted::{Posted, faulted};

/// Post one arming, stamped with the workspace the screen is painted under.
pub(crate) fn fleet(seat: &Seat, focus: &Focus, act: FleetAct) -> Posted {
    let workspace = match super::focused(focus) {
        Ok(workspace) => workspace,
        Err(why) => return Posted::Refused(why),
    };
    let op = act.op();
    let gesture = Gesture::Act(Act::Fleet {
        workspace,
        act: act.clone(),
    });
    match seat.answered(&encode(&gesture)) {
        Ok(Reply::Armed { armed }) => Posted::Refused(act.said(armed)),
        Ok(other) => Posted::Refused(kind_err(op, &other)),
        Err(why) => faulted(&why, op, BOARD),
    }
}

/// The read that settles a doubted one: the board, which carries a line per
/// armed loop and is the one home of what a loop is doing (§13.9).
const BOARD: &str = "The board says which loops are armed and what they hold.";
