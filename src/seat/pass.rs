//! **One pass of the model's loop**: the standing questions asked as deep as
//! the focus goes, the two acts the seat posts, and what survives a pass the
//! engine did not answer. Split from `model.rs` when the grace gave a pass
//! state to carry between calls (bl-3202) — the handle and the loop are
//! there, what a pass MEANS is here.

use super::{Focus, Snapshot};
use crate::codec::reply::Reply;
use crate::codec::{Act, Ask, Gesture, encode};
use crate::transport::Seat;

/// **How many consecutive failed passes an error waits for.** The cadence is
/// the clock (bl-3202): passes are one rest apart, so a second consecutive
/// failure is exactly *"it did not clear within one rest"* — no timestamp to
/// keep, none to inject, and one clock rather than two.
const GRACE: u32 = 1;

/// What the worker carries between passes: the last answer the engine
/// actually gave, and how many passes have failed since one did.
#[derive(Default)]
pub(super) struct Standing {
    last: Snapshot,
    failed: u32,
}

impl Standing {
    /// One refresh pass, and the snapshot the frame should paint for it.
    ///
    /// **A failure is not an error until it persists** (bl-3202). Swapping
    /// back into the app raced the network coming back: the first pass after
    /// a resume failed on a name lookup, and the frame painted a red banner
    /// over three emptied lists for a second. Both halves of that were the
    /// pass throwing away what it already had, so both are fixed here rather
    /// than in the paint — one clock, and a frame that only ever renders what
    /// it is handed.
    ///
    /// - **The rows survive.** A failed pass republishes the last answer the
    ///   engine gave, *under the focus it was asked at*: pairing one focus's
    ///   rows with another's is the one thing [`Snapshot`] promises never to
    ///   do, so a focus that moved gets the empty lists it honestly has.
    /// - **The sentence waits.** A refresh failure paints once it has
    ///   persisted past [`GRACE`]; a pass that succeeds clears it instantly,
    ///   because a standing success is never in doubt.
    /// - **`note` never waits.** It is a gesture's own answer — a refused
    ///   deposit, a start the engine would not run — and the operator just
    ///   acted. Silence there is a message that vanished.
    pub(super) fn pass(&mut self, seat: &Seat, focus: &Focus, note: Option<String>) -> Snapshot {
        let mut fresh = Snapshot {
            focus: focus.clone(),
            ..Snapshot::default()
        };
        let failed = fill(seat, focus, &mut fresh).err();
        if failed.is_none() {
            self.failed = 0;
            self.last = fresh;
        } else {
            self.failed += 1;
            if self.last.focus != *focus {
                self.last = fresh;
            }
        }
        let mut out = self.last.clone();
        out.error = match (note, failed.filter(|_| self.failed > GRACE)) {
            (Some(note), Some(failed)) => Some(format!("{note}; {failed}")),
            (note, failed) => note.or(failed),
        };
        out
    }
}

/// The standing questions, as deep as the focus goes. The first failure
/// stops the walk: an unreachable engine is one sentence, not three.
fn fill(seat: &Seat, focus: &Focus, snap: &mut Snapshot) -> Result<(), String> {
    snap.workspaces = match answer(seat, &Ask::Workspaces)? {
        Reply::Workspaces { rows, .. } => rows,
        other => return Err(kind_err("workspaces", &other)),
    };
    let Some(workspace) = focus.workspace.clone() else {
        return Ok(());
    };
    let ask = Ask::Conversations {
        workspace: workspace.clone(),
    };
    snap.conversations = match answer(seat, &ask)? {
        Reply::Conversations(rows) => rows,
        other => return Err(kind_err("conversations", &other)),
    };
    let Some(agent) = focus.agent.clone() else {
        return Ok(());
    };
    snap.transcript = match answer(seat, &Ask::Transcript { workspace, agent })? {
        Reply::Transcript(rows) => rows,
        other => return Err(kind_err("transcript", &other)),
    };
    Ok(())
}

fn answer(seat: &Seat, ask: &Ask) -> Result<Reply, String> {
    // The transport's two classes collapse to the sentence here, and rightly:
    // this model opens a connection per ask, so a broken channel is already
    // re-dialled by the next pass and there is nothing for it to decide
    // (bl-8641). The tool host, which holds one channel, is the caller that
    // reads the class.
    Ok(seat.answered(&encode(&Gesture::Ask(ask.clone())))?)
}

/// Post one message. The receipt is an `outcome` whose `ok` is the server's
/// own verdict; anything else is a sentence for the banner.
pub(super) fn deposit(seat: &Seat, focus: &Focus, content: String) -> Result<(), String> {
    let Focus {
        workspace: Some(workspace),
        agent: Some(agent),
    } = focus.clone()
    else {
        return Err("deposit: no conversation is focused".to_owned());
    };
    let act = Act::Message {
        workspace,
        agent,
        content,
    };
    match seat.answered(&encode(&Gesture::Act(act)))? {
        Reply::Outcome { ok: true, .. } => Ok(()),
        Reply::Outcome { stderr, .. } => Err(format!("deposit refused: {stderr}")),
        other => Err(kind_err("deposit", &other)),
    }
}

/// Stage a conversation and fire it — the §8.1 pair, run as one act. Named
/// for the wire's own word rather than the handle's, which is `Model::start`
/// for the worker and cannot be this.
///
/// The prepared body the engine answers with is carried into the firing
/// gesture **whole**: it is the engine's own statement about what was staged,
/// and a client that re-derived any field of it would be inventing world
/// state it does not own.
pub(super) fn started(seat: &Seat, focus: &Focus, goal: String) -> Result<(), String> {
    let Some(workspace) = focus.workspace.clone() else {
        return Err("start: no workspace is focused".to_owned());
    };
    let staged = match seat.answered(&encode(&Gesture::Act(Act::Prepare { workspace })))? {
        Reply::Prepared(prepared) => prepared,
        other => return Err(kind_err("start", &other)),
    };
    match seat.answered(&encode(&Gesture::Act(Act::Prompt {
        prepared: staged,
        goal,
    })))? {
        Reply::Started { .. } | Reply::Outcome { ok: true, .. } => Ok(()),
        Reply::Outcome { stderr, .. } => Err(format!("start refused: {stderr}")),
        other => Err(kind_err("start", &other)),
    }
}

/// The wrong-kind sentence names the kind, never the rows it carried.
fn kind_err(asked: &str, got: &Reply) -> String {
    format!("{asked}: the engine answered {} instead", got.kind())
}
