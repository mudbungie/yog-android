//! **The acts a conversation ROW fires** (DESIGN §13.5, bl-f97c): interrupt,
//! retarget, flag. Split from `seat::acts` rather than added to it because
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

/// **Which receipt an act earns.** Two, because the wire answers these three
/// with two shapes: `interrupt` and `retarget` both hand back the `outcome` of
/// what they ran, and `flag` has a receipt of its own that carries nothing but
/// its `ok`. Read off the act rather than guessed at the reply, so an engine
/// answering the other shape is a named refusal instead of a silent success.
enum Receipt {
    Outcome,
    Flagged,
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
    }
}
