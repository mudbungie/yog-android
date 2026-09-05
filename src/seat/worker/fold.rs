//! **What one answer does to the standing** — the folds the loop next door
//! spends, split from it (bl-f36e) when the ball pane's act took `worker.rs`
//! past the 300 wall. The seam is the one the file already read as: `run` is
//! the loop and `wait` is its clock, and everything here is *a read came back,
//! now what does the standing hold*.
//!
//! Three of these swallow their failure on purpose and each says why at its
//! own site: a preload, a re-read and a re-pane are not the operator's
//! gesture, they are what makes a gesture's effect visible, and losing what
//! the engine already gave over one it did not is the defect §13.2's grace
//! exists to prevent.

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

/// **Re-read the trail after an act on it**, swallowing the failure: this is
/// not the operator's gesture, it is what makes the gesture's effect visible,
/// and its absence leaves the rows exactly as they were — which is where they
/// stood before the act was fired. `preload`'s rule, on the other pair.
pub(super) fn reread(seat: &Seat, standing: &mut Standing) {
    if let Ok(rows) = asks::ops(seat) {
        standing.trail = rows;
    }
}

/// Fold one selector read into the standing options, or hand back the
/// sentence it failed with. One body for both reads, because the only
/// difference between them is which slot the envelope lands in.
pub(super) fn learned(
    read: Result<(String, serde_json::Value), String>,
    provider: Option<String>,
    standing: &mut Standing,
) -> Option<String> {
    match read {
        Ok((workspace, envelope)) => {
            standing
                .options
                .learned(&workspace, provider.as_deref(), envelope);
            None
        }
        Err(why) => Some(why),
    }
}

/// **One search, held** (bl-4c2b) — `learned`'s shape for the other read a
/// gesture makes, and here for the same reason: what the fold does with an
/// answer is the worker's business, and what a PASS means is `seat::pass`'s.
///
/// The answer replaces whatever was held. A failure does NOT: the operator is
/// still looking at the hits they have, and dropping an answer the engine gave
/// because of one it did not is the defect §13.2's grace exists to prevent.
pub(super) fn searched(
    read: Result<Option<crate::codec::Found>, String>,
    standing: &mut Standing,
) -> Option<String> {
    match read {
        Ok(found) => {
            standing.found = found;
            None
        }
        Err(why) => Some(why),
    }
}

/// **The trail's read, folded.** The answer replaces what it answers and a
/// failure keeps what was there, which is `searched`'s rule and is here for
/// its reason: losing an answer the engine gave over one it did not is the
/// defect §13.2's grace exists to prevent.
pub(super) fn opened(
    read: Result<Vec<crate::codec::OpRow>, String>,
    standing: &mut Standing,
) -> Option<String> {
    match read {
        Ok(rows) => {
            standing.trail = rows;
            None
        }
        Err(why) => Some(why),
    }
}

/// The ball pane's read, folded on `opened`'s terms exactly: opening the
/// surface is the ask, the answer replaces what was held, and a failure keeps
/// what was there.
pub(super) fn paned(
    read: Result<crate::codec::Pane, String>,
    standing: &mut Standing,
) -> Option<String> {
    match read {
        Ok(pane) => {
            standing.pane = Some(pane);
            None
        }
        Err(why) => Some(why),
    }
}
