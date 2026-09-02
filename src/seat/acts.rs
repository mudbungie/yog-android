//! **The two acts this seat posts** (§8): a message into the focused
//! conversation, and a conversation started. Split from `pass.rs` — a pass is
//! what the seat ASKS, on its own clock; an act is what the operator said,
//! and its answer is a sentence for the banner rather than a row for a list.

use serde_json::Value;

use super::Focus;
use crate::codec::reply::Reply;
use crate::codec::{Act, Ask, Gesture, encode};
use crate::transport::Seat;

use super::pass::{answer, kind_err};

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

/// **The focused workspace's providers** (bl-0267), handed back as the
/// engine's own envelope: what the selectors hold is what the engine said,
/// and the cache stores exactly that (§14).
pub(super) fn providers(seat: &Seat, focus: &Focus) -> Result<(String, Value), String> {
    let workspace = focused(focus)?;
    let ask = Ask::Providers {
        workspace: workspace.clone(),
    };
    match answer(seat, &ask)? {
        (Reply::Providers(_), envelope) => Ok((workspace, envelope)),
        (other, _) => Err(kind_err("providers", &other)),
    }
}

/// One provider's models, the same way.
pub(super) fn models(
    seat: &Seat,
    focus: &Focus,
    provider: &str,
) -> Result<(String, Value), String> {
    let workspace = focused(focus)?;
    let ask = Ask::Models {
        workspace: workspace.clone(),
        provider: provider.to_owned(),
    };
    match answer(seat, &ask)? {
        (Reply::Models(_), envelope) => Ok((workspace, envelope)),
        (other, _) => Err(kind_err("models", &other)),
    }
}

/// **The pick** (bl-0267): one assignment, stated whole. The role is this
/// seat's own — a phone assigns the worker and nothing else — and the
/// receipt is read rather than discarded, so an engine that refused the
/// assignment says so on the glass instead of the selector claiming it took.
pub(super) fn pick(seat: &Seat, focus: &Focus, provider: &str, model: &str) -> Result<(), String> {
    let act = Act::PickModel {
        workspace: focused(focus)?,
        role: WORKER.to_owned(),
        provider: provider.to_owned(),
        model: model.to_owned(),
    };
    match seat.answered(&encode(&Gesture::Act(act)))? {
        Reply::Applied => Ok(()),
        // No `outcome` arm, deliberately: a pick is a config write and the
        // engine answers it `applied` or refuses in band (the kind-less
        // envelope, which `answered` has already turned into this `?`). A
        // deposit has an outcome because a deposit RUNS something.
        other => Err(kind_err("model", &other)),
    }
}

/// **The two §9.4 tuning gestures** (bl-dfbb): how much reasoning the
/// worker's model calls request, and whether they ask for the provider's
/// priority lane. Both are role config the engine switches at the next step,
/// so they take mid-conversation and neither restarts anything.
///
/// One body for the pair because they are one act with two shapes: the
/// receipt is `applied` either way, and a refusal is the engine's own
/// sentence in the banner every other refusal uses.
fn tune(seat: &Seat, tuning: Act) -> Result<(), String> {
    match seat.answered(&encode(&Gesture::Act(tuning)))? {
        Reply::Applied => Ok(()),
        other => Err(kind_err("tune", &other)),
    }
}

/// The `effort` gesture for the focused workspace's worker.
pub(super) fn effort(
    seat: &Seat,
    focus: &Focus,
    level: Option<crate::codec::Effort>,
) -> Result<(), String> {
    let act = Act::Effort {
        workspace: focused(focus)?,
        role: WORKER.to_owned(),
        level,
    };
    tune(seat, act)
}

/// The `priority` gesture for the same role.
pub(super) fn priority(seat: &Seat, focus: &Focus, on: bool) -> Result<(), String> {
    let act = Act::Priority {
        workspace: focused(focus)?,
        role: WORKER.to_owned(),
        on,
    };
    tune(seat, act)
}

/// The one role a phone assigns. Named here because it is this seat's whole
/// answer to the wire's free `role` token (DESIGN §13.2's controls row).
const WORKER: &str = "worker";

/// The focused workspace, or the sentence a control earns for acting with no
/// workspace under it — the same shape the deposit's own guard has.
fn focused(focus: &Focus) -> Result<String, String> {
    focus
        .workspace
        .clone()
        .ok_or_else(|| "no workspace is focused".to_owned())
}

/// **Stop the turn in flight** (REMOTE §3.1, bl-48fa) — the op, and never a
/// deposited `/stop`: a slash line is content, and content wakes the very
/// driver it meant to kill. `children` carries the subtree.
///
/// The receipt is an `outcome` and its `ok` is litany's own verdict, so a
/// stop that landed on nothing says so in the operator's banner rather than
/// being read here as success.
pub(super) fn stop(seat: &Seat, focus: &Focus, children: bool) -> Result<(), String> {
    let Focus {
        workspace: Some(workspace),
        agent: Some(agent),
    } = focus.clone()
    else {
        return Err("stop: no conversation is focused".to_owned());
    };
    let act = Act::Stop {
        workspace,
        agent,
        children,
    };
    match seat.answered(&encode(&Gesture::Act(act)))? {
        Reply::Outcome { ok: true, .. } => Ok(()),
        Reply::Outcome { stderr, .. } => Err(format!("stop refused: {stderr}")),
        other => Err(kind_err("stop", &other)),
    }
}

/// **Nudge the focused conversation** (§8.2, bl-d09e): re-prompt it from
/// where it stands. It deposits nothing — a nudge is a detached
/// `litany advance`, not a message — so a branch that stopped advancing goes
/// on without a line in its transcript saying an operator poked it.
///
/// The receipt is `nudged` and carries nothing: what the nudge did shows up
/// in the next cadence read like any other work.
pub(super) fn nudge(seat: &Seat, focus: &Focus) -> Result<(), String> {
    let Focus {
        workspace: Some(workspace),
        agent: Some(agent),
    } = focus.clone()
    else {
        return Err("nudge: no conversation is focused".to_owned());
    };
    match seat.answered(&encode(&Gesture::Act(Act::Nudge { workspace, agent })))? {
        Reply::Nudged => Ok(()),
        other => Err(kind_err("nudge", &other)),
    }
}

/// **The answer in flight** (REMOTE §5.5, bl-4822), one shot: §5.5 says a
/// read starts holding nothing and *"the first frame of any read is the whole
/// tail so far"*, so what comes back is the answer as it stands and this seat
/// replaces rather than appends. The append fold belongs to a seat that HOLDS
/// the connection, and this one does not (DESIGN §7).
pub(super) fn follow(seat: &Seat, focus: &Focus) -> Result<crate::codec::Stream, String> {
    let Focus {
        workspace: Some(workspace),
        agent: Some(agent),
    } = focus.clone()
    else {
        return Err("follow: no conversation is focused".to_owned());
    };
    let ask = Ask::Follow { workspace, agent };
    match super::pass::answer(seat, &ask)? {
        (Reply::Follow(stream), _) => Ok(stream),
        (other, _) => Err(kind_err("follow", &other)),
    }
}
