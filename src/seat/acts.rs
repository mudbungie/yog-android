//! **The acts this seat posts** (§8): a message into the focused
//! conversation, a conversation started, the turn stopped or nudged, and the
//! worker role's tuning. A pass is what the seat ASKS, on its own clock
//! (`seat::pass`); the reads a gesture asks for are `seat::asks`; an act is
//! what the operator said, and its answer is a sentence for the banner rather
//! than a row for a list.
//!
//! **Every act here ends in one of three states and never in two** (yog
//! REMOTE §3, bl-d1f1, consumed in bl-07b1). The engine took it, the engine
//! refused it, or *the reply was lost* — and the third is not a failure. An
//! act is not idempotent (REMOTE §9.8: two clicks of Nudge are two nudges),
//! no idempotency token rides the envelope and no redelivery slot exists for
//! acts, so **nothing in this file ever sends an act a second time**: the
//! recovery is a read of the world, which this model already makes on its own
//! cadence. What is owed to the operator is the sentence saying so, and the
//! read that answers it.

use super::Focus;
use crate::codec::reply::Reply;
use crate::codec::{Act, Gesture, encode};
use crate::transport::Seat;

use super::pass::kind_err;
use super::posted::{Posted, faulted};

mod admin;
mod ball;
mod candidate;
mod fleet;
mod fork;
mod held;
mod row;
mod seen;
mod trail;

pub(super) use admin::admin;
pub(super) use ball::ball;
pub(super) use candidate::{candidate, spread};
pub(super) use fleet::fleet;
pub(super) use fork::fork;
pub(super) use held::answer;
pub(super) use row::row;
pub(super) use seen::seen;
pub(super) use trail::{ack, clear_trail};

/// Post one message. The receipt is an `outcome` whose `ok` is the server's
/// own verdict; anything else is a sentence for the banner.
pub(super) fn deposit(seat: &Seat, focus: &Focus, content: String) -> Posted {
    let Focus {
        workspace: Some(workspace),
        agent: Some(agent),
    } = focus.clone()
    else {
        return Posted::Refused("deposit: no conversation is focused".to_owned());
    };
    let act = Act::Message {
        workspace,
        agent,
        content,
    };
    match seat.answered(&encode(&Gesture::Act(act))) {
        Ok(Reply::Outcome { ok: true, .. }) => Posted::Took,
        Ok(Reply::Outcome { stderr, .. }) => Posted::Refused(format!("deposit refused: {stderr}")),
        Ok(other) => Posted::Refused(kind_err("deposit", &other)),
        Err(why) => faulted(&why, "deposit", TRANSCRIPT),
    }
}

/// The read that settles a doubted message: the transcript this seat re-reads
/// every cadence, where the echo is already standing.
const TRANSCRIPT: &str = "The transcript says whether it landed, and the echo above stands \
    until it does.";

/// Stage a conversation and fire it — the §8.1 pair, run as one act. Named
/// for the wire's own word rather than the handle's, which is `Model::start`
/// for the worker and cannot be this.
///
/// The prepared body the engine answers with is carried into the firing
/// gesture **whole**: it is the engine's own statement about what was staged,
/// and a client that re-derived any field of it would be inventing world
/// state it does not own.
///
/// **A lost `prepare` is in doubt for the same reason a lost `prompt` is**,
/// and the sentence is the same read: staging mints a body in the engine, so
/// re-staging on a dead reply would leave two, and only the list can say
/// whether anything started.
pub(super) fn started(seat: &Seat, focus: &Focus, goal: String) -> Posted {
    let Some(workspace) = focus.workspace.clone() else {
        return Posted::Refused("start: no workspace is focused".to_owned());
    };
    let staged = match seat.answered(&encode(&Gesture::Act(Act::Prepare { workspace }))) {
        Ok(Reply::Prepared(prepared)) => prepared,
        Ok(other) => return Posted::Refused(kind_err("start", &other)),
        Err(why) => return faulted(&why, "start", LIST),
    };
    match seat.answered(&encode(&Gesture::Act(Act::Prompt {
        prepared: staged,
        goal,
    }))) {
        Ok(Reply::Started { .. } | Reply::Outcome { ok: true, .. }) => Posted::Took,
        Ok(Reply::Outcome { stderr, .. }) => Posted::Refused(format!("start refused: {stderr}")),
        Ok(other) => Posted::Refused(kind_err("start", &other)),
        Err(why) => faulted(&why, "start", LIST),
    }
}

/// The read that settles a doubted start.
const LIST: &str = "The workspace's conversation list says whether one started.";

/// **The pick** (bl-0267): one assignment, stated whole. The role is this
/// seat's own — a phone assigns the worker and nothing else — and the
/// receipt is read rather than discarded, so an engine that refused the
/// assignment says so on the glass instead of the selector claiming it took.
pub(super) fn pick(seat: &Seat, focus: &Focus, provider: &str, model: &str) -> Posted {
    let workspace = match focused(focus) {
        Ok(workspace) => workspace,
        Err(why) => return Posted::Refused(why),
    };
    let act = Act::PickModel {
        workspace,
        role: WORKER.to_owned(),
        provider: provider.to_owned(),
        model: model.to_owned(),
    };
    match seat.answered(&encode(&Gesture::Act(act))) {
        Ok(Reply::Applied) => Posted::Took,
        // No `outcome` arm, deliberately: a pick is a config write and the
        // engine answers it `applied` or refuses in band (the kind-less
        // envelope, which `answered` has already turned into a `Refused`). A
        // deposit has an outcome because a deposit RUNS something.
        Ok(other) => Posted::Refused(kind_err("model", &other)),
        Err(why) => faulted(&why, "model", ASSIGNMENTS),
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
fn tune(seat: &Seat, tuning: Act) -> Posted {
    match seat.answered(&encode(&Gesture::Act(tuning))) {
        Ok(Reply::Applied) => Posted::Took,
        Ok(other) => Posted::Refused(kind_err("tune", &other)),
        Err(why) => faulted(&why, "tune", ASSIGNMENTS),
    }
}

/// The read that settles a doubted config write — and the one the worker
/// makes anyway: every tuning act is followed by the assignments read that
/// overtakes the control's optimistic value (bl-e9f9), so the recovery here
/// is already the next thing that happens.
const ASSIGNMENTS: &str = "The workspace's assignments are read straight after either way, \
    and what they say is what is in force.";

/// The `effort` gesture for the focused workspace's worker.
pub(super) fn effort(seat: &Seat, focus: &Focus, level: Option<crate::codec::Effort>) -> Posted {
    match focused(focus) {
        Ok(workspace) => tune(
            seat,
            Act::Effort {
                workspace,
                role: WORKER.to_owned(),
                level,
            },
        ),
        Err(why) => Posted::Refused(why),
    }
}

/// The `priority` gesture for the same role.
pub(super) fn priority(seat: &Seat, focus: &Focus, on: bool) -> Posted {
    match focused(focus) {
        Ok(workspace) => tune(
            seat,
            Act::Priority {
                workspace,
                role: WORKER.to_owned(),
                on,
            },
        ),
        Err(why) => Posted::Refused(why),
    }
}

/// The one role a phone assigns. Named here because it is this seat's whole
/// answer to the wire's free `role` token (DESIGN §13.2's controls row).
const WORKER: &str = "worker";

/// The focused workspace, or the sentence a control earns for acting with no
/// workspace under it — the same shape the deposit's own guard has. Shared
/// with `seat::asks`, whose three reads are focused the same way.
pub(super) fn focused(focus: &Focus) -> Result<String, String> {
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
pub(super) fn stop(seat: &Seat, focus: &Focus, children: bool) -> Posted {
    let Focus {
        workspace: Some(workspace),
        agent: Some(agent),
    } = focus.clone()
    else {
        return Posted::Refused("stop: no conversation is focused".to_owned());
    };
    let act = Act::Stop {
        workspace,
        agent,
        children,
    };
    match seat.answered(&encode(&Gesture::Act(act))) {
        Ok(Reply::Outcome { ok: true, .. }) => Posted::Took,
        Ok(Reply::Outcome { stderr, .. }) => Posted::Refused(format!("stop refused: {stderr}")),
        Ok(other) => Posted::Refused(kind_err("stop", &other)),
        Err(why) => faulted(&why, "stop", FLIGHT),
    }
}

/// The read that settles a doubted stop or nudge: the conversation's own row,
/// whose `flight` is where every conversation-level gate rides (REMOTE §9.4)
/// and which the next pass re-reads.
const FLIGHT: &str = "The conversation's row says whether a turn is still in flight.";

/// **Nudge the focused conversation** (§8.2, bl-d09e): re-prompt it from
/// where it stands. It deposits nothing — a nudge is a detached
/// `litany advance`, not a message — so a branch that stopped advancing goes
/// on without a line in its transcript saying an operator poked it.
///
/// The receipt is `nudged` and carries nothing: what the nudge did shows up
/// in the next cadence read like any other work. **Which is also the whole
/// recovery for a lost one** — and the reason a doubted nudge must not be
/// repeated: §9.8's own example of a gesture that is not idempotent.
pub(super) fn nudge(seat: &Seat, focus: &Focus) -> Posted {
    let Focus {
        workspace: Some(workspace),
        agent: Some(agent),
    } = focus.clone()
    else {
        return Posted::Refused("nudge: no conversation is focused".to_owned());
    };
    match seat.answered(&encode(&Gesture::Act(Act::Nudge { workspace, agent }))) {
        Ok(Reply::Nudged) => Posted::Took,
        Ok(other) => Posted::Refused(kind_err("nudge", &other)),
        Err(why) => faulted(&why, "nudge", FLIGHT),
    }
}
