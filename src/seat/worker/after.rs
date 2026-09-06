//! **What a fold does AFTER an act** — the re-reads that make a gesture's
//! effect visible, split from the read folds beside them (bl-99fd) when the
//! fork's anchor took `fold.rs` toward the wall. The seam is the one the file
//! already read as, in its own words: *"a preload, a re-read and a re-pane are
//! not the operator's gesture, they are what makes a gesture's effect
//! visible."*
//!
//! **Every one of them swallows its failure**, and that is the whole of what
//! they share: losing what the engine already gave over an answer it did not
//! is the defect §13.2's grace exists to prevent, so an act whose follow-up
//! read fails leaves the rows exactly where they stood.

use super::super::pass::Standing;
use super::super::{Focus, asks};
use crate::transport::Seat;

/// **Read what the workspace is set to, and say nothing if it cannot be
/// read** (bl-e9f9). This is a preload, not an answer to a gesture the
/// operator made: its absence means the controls seed from nothing, which is
/// exactly where they stood before this read existed. So every way it can
/// fail is swallowed — including the one that will be common for a while, an
/// engine that predates the read and refuses the op in band by name. A
/// banner for that would be this app telling an operator off for running the
/// engine they have.
pub(super) fn preload(seat: &Seat, focus: &Focus, standing: &mut Standing) {
    if let Ok((workspace, envelope)) = asks::roles(seat, focus) {
        standing.options.assigned(&workspace, envelope);
        standing.reads += 1;
    }
}

/// **One ball act, and the read that shows what it did** (§13.9). The pane is
/// not a standing read, so nothing re-asks it on its own and a filing, a claim
/// or a close is invisible until the view it happened in is read again — which
/// is `reread`'s rule, on the pane.
///
/// **The `--as` stamp is the focused workspace's own name**, or the empty
/// string where nothing is focused — which the engine refuses in its own
/// words, and is the right end for that sentence to come from. The pane that
/// fires these is only ever painted under a focused workspace, so the empty
/// case is unreachable from the glass and is not a second refusal here.
pub(super) fn balled(
    seat: &Seat,
    focus: &Focus,
    standing: &mut Standing,
    project: String,
    act: crate::codec::BallAct,
) -> Option<String> {
    let name = focus.workspace.clone().unwrap_or_default();
    let note = super::super::acts::ball(seat, project, name, act).note();
    repane(seat, focus, standing);
    note
}

/// **Re-read the pane after an act on it**, swallowing the failure, for
/// `reread`'s reason exactly: this is not the operator's gesture, it is what
/// makes the gesture's effect visible, and its absence leaves the rows where
/// they were. The view re-asked is the one the pane is holding — the act was
/// fired from it, and a pane holding another view's answer is unpaintable
/// anyway (§13.9).
fn repane(seat: &Seat, focus: &Focus, standing: &mut Standing) {
    let Some(view) = standing.pane.as_ref().map(crate::codec::Pane::view) else {
        return;
    };
    if let Ok(pane) = asks::balls(seat, focus, view) {
        standing.pane = Some(pane);
    }
}

/// **One admin act, and the branch a `marks` write answers with** (§13.17).
/// Four of the five need no read after them — what a config write did is read
/// by tapping the destination again, and what a deletion did is that its
/// subject is gone, which the next pass re-reads — and the fifth answers with
/// the engine's own re-read, so there is nothing to ask twice.
pub(super) fn administered(
    seat: &Seat,
    standing: &mut Standing,
    act: crate::codec::AdminAct,
) -> Option<String> {
    let workspace = match &act {
        crate::codec::AdminAct::Marks { workspace, .. } => Some(workspace.clone()),
        _ => None,
    };
    let (posted, landed) = super::super::acts::admin(seat, act);
    if let (Some(workspace), Some(branch)) = (workspace, landed) {
        standing.marks = Some(crate::codec::Marks { workspace, branch });
    }
    posted.note()
}

/// **The mint, and the material it answered** (§13.18). It is here beside the
/// other acts that write to the standing rather than in `fold`, because what
/// it holds is not a read's answer: no read can fetch it back, the engine
/// having shredded the key as it answered.
pub(super) fn minted(
    seat: &Seat,
    focus: &Focus,
    standing: &mut Standing,
    name: String,
    grade: crate::leaf::Grade,
) -> Option<String> {
    let (posted, envelope) = super::super::acts::enroll(seat, focus, name, grade);
    if let Some(envelope) = envelope {
        standing.minted = Some(envelope);
    }
    posted.note()
}

/// **Re-read the trail after an act on it**, swallowing the failure: this is
/// not the operator's gesture, it is what makes the gesture's effect visible,
/// and its absence leaves the rows exactly as they were — which is where they
/// stood before the act was fired. `preload`'s rule, on the other pair.
pub(super) fn reread(seat: &Seat, standing: &mut Standing) {
    if let Ok(rows) = asks::ops(seat) {
        standing.trail = rows;
    }
}

/// **The read that shows what a candidate act did** (§13.12). The listing is
/// derived when asked and nothing stands over it, so a fan, a delivery or a
/// retirement is invisible until it is asked again — `balled`'s rule, on the
/// other aimed screen. One fold for both commands, because what follows an act
/// here is the same read whichever act it was.
pub(super) fn listed(
    seat: &Seat,
    focus: &Focus,
    standing: &mut Standing,
    posted: crate::seat::posted::Posted,
) -> Option<String> {
    let note = posted.note();
    if let Ok(spread) = asks::science(seat, focus) {
        standing.candidates = Some(spread);
    }
    note
}
