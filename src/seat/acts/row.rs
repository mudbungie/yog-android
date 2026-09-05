//! **The acts a conversation ROW fires** (DESIGN §13.5, bl-f97c; §13.7,
//! bl-b39d): interrupt, retarget, flag, revoke, restore. Split from `seat::acts` rather than added to it because
//! they differ from every act beside them in one structural way — **the
//! subject is the row that was pressed, not the focus.** A long-press names
//! its own conversation; nothing has to be opened first, and nothing about the
//! operator's current depth may reach the wire here. So the workspace comes
//! from the focus (a row is only ever painted under one) and the agent is
//! carried in, which is the whole difference and is worth a file to say once.
//!
//! **Three fates and never a resend**, exactly as `seat::acts` states it. None
//! of these three is idempotent — two interrupts are two interruptions, two
//! flags are two rows on the trail — so a lost reply is answered with the
//! sentence and the read, never a second gesture.

use crate::codec::reply::Reply;
use crate::codec::{Act, Gesture, RowAct, encode};
use crate::seat::pass::kind_err;
use crate::transport::Seat;

use super::super::Focus;
use super::super::posted::{Posted, faulted};

/// **Which receipt an act earns.** Three shapes for five acts: `interrupt`
/// and `retarget` hand back the `outcome` of what they ran, `flag` has a
/// receipt carrying nothing but its `ok`, and the floor pair answers with the
/// floor that stands over the conversation AFTERWARDS. Read off the act rather
/// than guessed at the reply, so an engine answering the other shape is a
/// named refusal instead of a silent success.
enum Receipt {
    Outcome,
    Flagged,
    /// The floor this act is asking for — `true` for a revoke, `false` for a
    /// restore. It is carried because the receipt is **re-derived by the
    /// engine, never echoed**: restoring a conversation whose ancestor is
    /// still revoked leaves it floored, and the answer says so. Silence there
    /// would be this seat telling an operator they got something back that
    /// they did not.
    Floored(bool),
}

/// Post one row act at `agent` in the focused workspace.
pub(crate) fn row(seat: &Seat, focus: &Focus, agent: String, act: RowAct) -> Posted {
    let op = act.op();
    let workspace = match super::focused(focus) {
        Ok(workspace) => workspace,
        Err(why) => return Posted::Refused(format!("{op}: {why}")),
    };
    let receipt = match act {
        RowAct::Flag { .. } => Receipt::Flagged,
        RowAct::Interrupt { .. } | RowAct::Retarget => Receipt::Outcome,
        RowAct::Revoke => Receipt::Floored(true),
        RowAct::Restore => Receipt::Floored(false),
    };
    let settles = settles(&act);
    let gesture = Gesture::Act(Act::Row {
        workspace,
        agent,
        act,
    });
    match (receipt, seat.answered(&encode(&gesture))) {
        // Yes, in the two shapes the wire has for it: the flag's own `ok`
        // (a `flagged` frame carries no other field), and the `outcome` its
        // two siblings hand back. Read against the receipt this act EXPECTS,
        // so an engine answering the other shape falls through to the named
        // refusal below rather than being taken for a success.
        (Receipt::Flagged, Ok(Reply::Flagged))
        | (Receipt::Outcome, Ok(Reply::Outcome { ok: true, .. })) => Posted::Took,
        // The floor pair's yes is the floor AGREEING with what was asked for.
        // A disagreement is not a wrong shape and not a lost reply — the
        // engine took the write and the world came out somewhere else — so it
        // is the one sentence an operator cannot get any other way.
        (Receipt::Floored(wanted), Ok(Reply::Floored { standing })) if standing == wanted => {
            Posted::Took
        }
        (Receipt::Floored(wanted), Ok(Reply::Floored { .. })) => Posted::Refused(stands(wanted)),
        (Receipt::Outcome, Ok(Reply::Outcome { stderr, .. })) => {
            Posted::Refused(format!("{op} refused: {stderr}"))
        }
        (_, Ok(other)) => Posted::Refused(kind_err(op, &other)),
        (_, Err(why)) => faulted(&why, op, settles),
    }
}

/// **The read that settles a doubted row act** — and for one of the three, the
/// honest statement that none does.
///
/// The contract (`seat::posted`) is that an act with no reply names the read
/// showing what became of it, because the world is the durable record. Two of
/// these can: an interrupt's text shows up in the conversation's transcript,
/// and a flag's mark on its own row. **A retarget cannot**, and saying so is
/// the only honest answer available: what a retarget writes is a mark the
/// `agent` read carries, and this seat does not make that read (bl-146b). A
/// sentence naming the conversation list here would be this app claiming a
/// recovery it has not got.
/// **What the floor came out as, when it is not what was asked for.** Only
/// one of the two ever happens in practice — a restore under a still-revoked
/// ancestor — and it is exactly the case the engine refuses to lie about, so
/// this seat does not either.
fn stands(wanted: bool) -> String {
    if wanted {
        "revoke: the engine took it and says no floor stands over the conversation".to_owned()
    } else {
        "restore: the conversation stays floored — an ancestor of it is still revoked".to_owned()
    }
}

fn settles(act: &RowAct) -> &'static str {
    match act {
        RowAct::Interrupt { .. } => {
            "The conversation's transcript says whether the text landed, and its row \
             whether a turn is still in flight."
        }
        RowAct::Retarget => {
            "No read this seat makes says whether it landed — the mark rides the \
             conversation's own machinery read (bl-146b). Nothing was discarded either \
             way, and the lineage it settles onto is the one it is already following."
        }
        RowAct::Flag { .. } => {
            "The conversation's row carries the attention mark, and the next pass \
             re-reads it."
        }
        RowAct::Revoke | RowAct::Restore => {
            "No read this seat makes says which floor stands — it rides the \
             conversation's own machinery read (bl-146b). Nothing was killed either \
             way: the conversation keeps running, keeps its branch and keeps reading, \
             and what a floor changes is how its NEXT tool call is adjudicated."
        }
    }
}
