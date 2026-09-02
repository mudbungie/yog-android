//! **The two acts this seat posts** (§8): a message into the focused
//! conversation, and a conversation started. Split from `pass.rs` — a pass is
//! what the seat ASKS, on its own clock; an act is what the operator said,
//! and its answer is a sentence for the banner rather than a row for a list.

use super::Focus;
use crate::codec::reply::Reply;
use crate::codec::{Act, Gesture, encode};
use crate::transport::Seat;

use super::pass::kind_err;

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
