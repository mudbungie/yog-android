//! **The offer, and the two acts that settle one** (yog REMOTE §15.1 steps 1
//! and 5; DESIGN §13.18).
//!
//! `offer` answers a value — the registration URL — rather than a receipt, so
//! it rides back beside the outcome. **Nothing here is re-sent on a lost
//! reply**, and for each act the read that settles the doubt is the same one:
//! the workspace's pending offers. A mint that landed is a row there; an
//! approval that landed took its row away and seated a machine; a decline
//! that landed took an open row away.

use crate::codec::reply::Reply;
use crate::codec::{Act, Gesture, OfferAct, Offered, Settle, encode};
use crate::leaf::Grade;
use crate::seat::pass::kind_err;
use crate::transport::Seat;

use super::super::Focus;
use super::super::posted::{Posted, faulted};

/// What an act in doubt says about how to find out (bl-07b1).
const SETTLES: &str = "The pending offers list what the server holds — refresh it to see \
     whether this landed. Nothing was sent again.";

/// **Mint an offer** for `name` at `grade` in the focused workspace.
pub(crate) fn offer(
    seat: &Seat,
    focus: &Focus,
    name: String,
    grade: Grade,
) -> (Posted, Option<Offered>) {
    let workspace = match super::focused(focus) {
        Ok(workspace) => workspace,
        Err(why) => return (Posted::Refused(format!("offer: {why}")), None),
    };
    let gesture = Gesture::Act(Act::Offer(OfferAct::Mint {
        workspace,
        name,
        grade,
    }));
    match seat.answered(&encode(&gesture)) {
        Ok(Reply::Offered(offered)) => (Posted::Took, Some(offered)),
        Ok(other) => (Posted::Refused(kind_err("offer", &other)), None),
        Err(why) => (faulted(&why, "offer", SETTLES), None),
    }
}

/// **Approve or decline** the pending offer under `name`.
pub(crate) fn settle(seat: &Seat, focus: &Focus, settle: Settle, name: String) -> Posted {
    let op = settle.op();
    let workspace = match super::focused(focus) {
        Ok(workspace) => workspace,
        Err(why) => return Posted::Refused(format!("{op}: {why}")),
    };
    let gesture = Gesture::Act(Act::Offer(OfferAct::Settle {
        settle,
        workspace,
        name,
    }));
    match (settle, seat.answered(&encode(&gesture))) {
        (Settle::Approve, Ok(Reply::Approved)) | (Settle::Decline, Ok(Reply::Declined)) => {
            Posted::Took
        }
        (_, Ok(other)) => Posted::Refused(kind_err(&op, &other)),
        (_, Err(why)) => faulted(&why, &op, SETTLES),
    }
}
