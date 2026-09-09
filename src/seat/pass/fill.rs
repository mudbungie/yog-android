//! **The standing questions a pass asks, as deep as the focus goes** — split
//! from `seat::pass` when the queue read made it four (§13.7, bl-b39d), on
//! the seam that file's own doc already draws: what a pass MEANS is there,
//! and WHICH questions one asks is here. They change for unrelated reasons —
//! the grace, the cache write and the published snapshot are the pass's, and
//! a new read is only ever a new arm in this walk.
//!
//! **Three again since bl-8e3c.** The queue read left this walk when
//! `attention` became follow-class (REMOTE §14.1): the intake this seat
//! dials holds it, so a pass that asked it would wait a hold. It is the
//! attention LANE's now (`seat::lane`, DESIGN §14.1), standing beside the
//! pass and writing the same holder.

use crate::cache::Envelopes;
use crate::codec::Ask;
use crate::codec::reply::Reply;
use crate::transport::{Seat, Wire};

use super::{Focus, Snapshot, kind_err, wired};

/// The standing questions, as deep as the focus goes. The first failure
/// stops the walk: an unreachable engine is one sentence, not three.
///
/// **The failure keeps its class** (bl-eec1). The pass's grace waits out a
/// channel and never an answer, so what it needs back is a [`Wire`] and not a
/// sentence — and a reply of the wrong kind is `Unusable` here for the reason
/// that variant already states: *"asking again asks the same question and
/// gets the same answer"*.
pub(super) fn fill(
    seat: &Seat,
    focus: &Focus,
    snap: &mut Snapshot,
    kept: &mut Envelopes,
) -> Result<(), Wire> {
    let (reply, envelope) = wired(seat, &Ask::Workspaces)?;
    snap.workspaces = match reply {
        Reply::Workspaces { rows, .. } => rows,
        other => return Err(Wire::Unusable(kind_err("workspaces", &other))),
    };
    kept.workspaces = Some(envelope);
    let Some(workspace) = focus.workspace.clone() else {
        return Ok(());
    };
    let ask = Ask::Conversations {
        workspace: workspace.clone(),
    };
    let (reply, envelope) = wired(seat, &ask)?;
    snap.conversations = match reply {
        Reply::Conversations(rows) => rows,
        other => return Err(Wire::Unusable(kind_err("conversations", &other))),
    };
    kept.conversations = Some(envelope);
    let Some(agent) = focus.agent.clone() else {
        return Ok(());
    };
    let (reply, envelope) = wired(seat, &Ask::Transcript { workspace, agent })?;
    snap.transcript = match reply {
        Reply::Transcript(rows) => rows,
        other => return Err(Wire::Unusable(kind_err("transcript", &other))),
    };
    kept.transcript = Some(envelope);
    Ok(())
}
