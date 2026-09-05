//! **The standing questions a pass asks, as deep as the focus goes** — split
//! from `seat::pass` when the queue read made it four (§13.7, bl-b39d), on
//! the seam that file's own doc already draws: what a pass MEANS is there,
//! and WHICH questions one asks is here. They change for unrelated reasons —
//! the grace, the cache write and the published snapshot are the pass's, and
//! a new read is only ever a new arm in this walk.

use crate::cache::Envelopes;
use crate::codec::Ask;
use crate::codec::reply::Reply;
use crate::transport::Seat;

use super::{Focus, Snapshot, answer, kind_err};

/// The standing questions, as deep as the focus goes. The first failure
/// stops the walk: an unreachable engine is one sentence, not three.
pub(super) fn fill(
    seat: &Seat,
    focus: &Focus,
    snap: &mut Snapshot,
    kept: &mut Envelopes,
) -> Result<(), String> {
    let (reply, envelope) = answer(seat, &Ask::Workspaces)?;
    snap.workspaces = match reply {
        Reply::Workspaces { rows, .. } => rows,
        other => return Err(kind_err("workspaces", &other)),
    };
    kept.workspaces = Some(envelope);
    let Some(workspace) = focus.workspace.clone() else {
        return Ok(());
    };
    let ask = Ask::Conversations {
        workspace: workspace.clone(),
    };
    let (reply, envelope) = answer(seat, &ask)?;
    snap.conversations = match reply {
        Reply::Conversations(rows) => rows,
        other => return Err(kind_err("conversations", &other)),
    };
    kept.conversations = Some(envelope);
    let Some(agent) = focus.agent.clone() else {
        return Ok(());
    };
    let (reply, envelope) = answer(seat, &Ask::Transcript { workspace, agent })?;
    snap.transcript = match reply {
        Reply::Transcript(rows) => rows,
        other => return Err(kind_err("transcript", &other)),
    };
    kept.transcript = Some(envelope);
    // **The decision queue, at the depth that spends it** (§13.7). It names no
    // workspace and no conversation — it is the whole world's queue — but it
    // is asked here rather than at every depth because the only screen that
    // paints it is the one a conversation is open on, and a phone's radio is
    // not free. Its rows address themselves, so nothing about being read under
    // one focus binds it to that focus.
    let (reply, envelope) = answer(seat, &Ask::Attention)?;
    snap.queue = match reply {
        Reply::Attention(rows) => rows,
        other => return Err(kind_err("attention", &other)),
    };
    kept.attention = Some(envelope);
    Ok(())
}
