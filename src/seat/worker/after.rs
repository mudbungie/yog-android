//! **What a fold does AFTER an act** — the re-reads that make a gesture's
//! effect visible, split from the read folds beside them (bl-99fd) when the
//! fork's anchor took `fold.rs` toward the wall. The seam is the one the file
//! already read as, in its own words: *"a preload, a re-read and a re-pane are
//! not the operator's gesture, they are what makes a gesture's effect
//! visible."*
//!
//! **The acts themselves are not here** (bl-2cad): an act is posted by the
//! poster, on its own thread, and `worker::post` hands what it earned to the
//! folds below. Nothing in this file says anything to the server but a read.
//!
//! **Every one of them swallows its failure**, and that is the whole of what
//! they share: losing what the server already gave over an answer it did not
//! is the defect §13.2's grace exists to prevent, so an act whose follow-up
//! read fails leaves the rows exactly where they stood.

use super::super::pass::Standing;
use super::super::posted::Posted;
use super::super::{Focus, asks};
use crate::transport::Seat;

/// **Read what the workspace is set to, and say nothing if it cannot be
/// read** (bl-e9f9). This is a preload, not an answer to a gesture the
/// operator made: its absence means the controls seed from nothing, which is
/// exactly where they stood before this read existed. So every way it can
/// fail is swallowed — including the one that will be common for a while, an
/// server that predates the read and refuses the op in band by name. A
/// banner for that would be this app telling an operator off for running the
/// server they have.
pub(super) fn preload(seat: &Seat, focus: &Focus, standing: &mut Standing) {
    if let Ok((workspace, envelope)) = asks::roles(seat, focus) {
        standing.options.assigned(&workspace, envelope);
        standing.reads += 1;
    }
}

/// **Re-read the pane after an act on it**, swallowing the failure, for
/// `reread`'s reason exactly: this is not the operator's gesture, it is what
/// makes the gesture's effect visible, and its absence leaves the rows where
/// they were. The view re-asked is the one the pane is holding — the act was
/// fired from it, and a pane holding another view's answer is unpaintable
/// anyway (§13.9).
pub(super) fn repane(seat: &Seat, focus: &Focus, standing: &mut Standing) {
    let Some(view) = standing.pane.as_ref().map(crate::codec::Pane::view) else {
        return;
    };
    if let Ok(pane) = asks::balls(seat, focus, view) {
        standing.pane = Some(pane);
    }
}

/// **What follows one admin act: the branch a `marks` write answered with**
/// (§13.17), and the listing a settle changed. `marked` is that write's
/// workspace beside the branch the server re-read; `settled` is whether the
/// act was the proposal verdict.
/// Four of the six need no read after them — what a config write did is read
/// by tapping the destination again, and what a deletion did is that its
/// subject is gone, which the next pass re-reads — `marks` answers with the
/// server's own re-read, so there is nothing to ask twice, and `proposal` is
/// the one that needs one: a settle takes the staging branch with it, so the
/// listing an operator is looking at still carries a row that no longer
/// exists (the trail acts' rule, one surface along).
pub(super) fn administered(
    seat: &Seat,
    focus: &Focus,
    standing: &mut Standing,
    marked: Option<(String, String)>,
    settled: bool,
    posted: Posted,
) -> Option<String> {
    if let Some((workspace, branch)) = marked {
        standing.marks = Some(crate::codec::Marks { workspace, branch });
    }
    if settled && let Ok(staged) = asks::proposals(seat, focus, None) {
        // The whole reading goes with the row it was of: what a settle leaves
        // is a listing, and a diff of a proposal that is gone is not a reading
        // anybody asked for.
        standing.proposals = Some(staged);
    }
    posted.note()
}

/// **The mint, and the read behind it** (§13.18): the URL `offer` answered is
/// held beside the pending rows it now appears among. It is here rather than
/// in `fold` because the URL is not a read's answer — no read gives it back.
pub(super) fn offered(
    seat: &Seat,
    focus: &Focus,
    standing: &mut Standing,
    name: String,
    posted: Posted,
    offered: Option<crate::codec::Offered>,
) -> Option<String> {
    let said = pending(seat, focus, standing, posted.note());
    if let (Some(offered), Some(held)) = (offered, standing.offers.as_mut()) {
        held.minted = Some((name, offered));
    }
    said
}

/// **Re-read the pending offers** into the surface, keeping the URL it shows
/// where the workspace is the one it was minted for. `said` is what the act
/// before this read had to say, which outranks the read's own refusal.
pub(super) fn pending(
    seat: &Seat,
    focus: &Focus,
    standing: &mut Standing,
    said: Option<String>,
) -> Option<String> {
    match super::super::asks::offers(seat, focus) {
        Ok((workspace, rows)) => {
            let minted = standing
                .offers
                .take()
                .filter(|held| held.about(&workspace))
                .and_then(|held| held.minted);
            standing.offers = Some(crate::codec::Pending {
                workspace,
                minted,
                rows,
            });
            said
        }
        Err(why) => said.or(Some(why)),
    }
}

/// **The sign-in's answer, adopted** (§13.19) — `offered`'s shape at the
/// other act whose answer is a value rather than a receipt. The standing it
/// answered with is the run's whole output so far, which is exactly what the
/// lane's first frame would carry, so nothing waits a cadence to paint.
///
/// **Starting a run watches its tail, here and nowhere else** (§24 item 5;
/// lernie §4.24's *one control fires two ops*). The watch is set at the
/// act's receipt rather than sent beside the act as a second command: a
/// `Watch` racing the pass would dial a lane on a pair with no run yet, and
/// the pass's order is a script the suite asserts. Each outcome decides its
/// own: a receipt is adopted (`started`, which watches); an act IN DOUBT
/// watches too, because the tail is the read that settles it and the doubt
/// sentence says so (`acts::login::SETTLES`); a refusal watches nothing — a
/// run that never started has said nothing.
///
/// **A code's receipt is not a new run's** (REMOTE §8.3, edition 31): the
/// finish is the same run's second command line, so the lane already held
/// goes on being followed and only the receipt's standing fact is taken
/// (`pass::login::handed`).
pub(super) fn signed(
    standing: &mut Standing,
    provider: String,
    finishing: bool,
    posted: Posted,
    view: Option<crate::codec::LoginView>,
) -> Option<String> {
    match view {
        Some(view) if finishing => standing.signing.handed(provider, view),
        Some(view) => standing.signing.started(provider, view),
        None if matches!(posted, Posted::InDoubt(_)) => standing.signing.watch(Some(provider)),
        None => {}
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
