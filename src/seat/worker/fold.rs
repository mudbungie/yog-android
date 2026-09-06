//! **What one answer does to the standing** — the folds the loop next door
//! spends, split from it (bl-f36e) when the ball pane's act took `worker.rs`
//! past the 300 wall. The seam is the one the file already read as: `run` is
//! the loop and `wait` is its clock, and everything here is *a read came back,
//! now what does the standing hold*.
//!
//! **What is here is one shape said many times**: an answer replaces what was
//! held, and a failure keeps what was there — `searched`'s rule, at every site
//! below. The folds that RE-READ after an act, which swallow their failure
//! instead, are `worker::after` (bl-99fd), split off on the seam this file
//! stated in its own words: they are not the operator's gesture, they are what
//! makes a gesture's effect visible.

use super::super::pass::Standing;

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

/// **The records screen's five reads, folded** — `paned`'s terms exactly, and
/// here for its reason: the answer replaces what was held and a failure keeps
/// what was there, because losing an answer the engine gave over one it did
/// not is the defect §13.2's grace exists to prevent.
pub(super) fn recorded(
    read: Result<crate::codec::Records, String>,
    standing: &mut Standing,
) -> Option<String> {
    match read {
        Ok(records) => {
            standing.records = Some(records);
            None
        }
        Err(why) => Some(why),
    }
}

/// **One step's drill-in, folded into the records it belongs to.** It is not
/// a value of its own: a step's records under no conversation's are rows with
/// no subject, so a drill-in that lands after the five were retired is
/// dropped rather than held. The answer carries its own `seq`, so the paint
/// asks it which row it is under and nothing here remembers a second name.
pub(super) fn drilled(
    read: Result<crate::codec::Step, String>,
    standing: &mut Standing,
) -> Option<String> {
    match read {
        Ok(step) => {
            if let Some(records) = standing.records.as_mut() {
                records.drilled = Some(step);
            }
            None
        }
        Err(why) => Some(why),
    }
}

/// **The governing config at a picked fork point, folded into the records it
/// belongs to** — `drilled`'s rule exactly, one read along: a policy under no
/// conversation's records is an answer with no subject, so an anchor that
/// lands after the six were retired is dropped rather than held. The answer
/// echoes no commit, so the commit it was asked at is carried in beside it.
pub(super) fn anchored(
    read: Result<crate::codec::Governing, String>,
    at: String,
    standing: &mut Standing,
) -> Option<String> {
    match read {
        Ok(governing) => {
            if let Some(records) = standing.records.as_mut() {
                records.anchored = Some((at, governing));
            }
            None
        }
        Err(why) => Some(why),
    }
}

/// **The candidates listing, folded** — `paned`'s terms again: the answer
/// replaces what was held and a failure keeps what was there.
pub(super) fn spread(
    read: Result<crate::codec::Spread, String>,
    standing: &mut Standing,
) -> Option<String> {
    match read {
        Ok(spread) => {
            standing.candidates = Some(spread);
            None
        }
        Err(why) => Some(why),
    }
}

/// **The worktree listing, folded** — `paned`'s terms again, and the value
/// carries the path it was asked at, so a preview lands under its own row and
/// never under one tapped since.
pub(super) fn filed(
    read: Result<crate::codec::Files, String>,
    standing: &mut Standing,
) -> Option<String> {
    match read {
        Ok(files) => {
            standing.files = Some(files);
            None
        }
        Err(why) => Some(why),
    }
}

/// **The work diff, folded** — `filed`'s terms exactly, one subject along.
pub(super) fn worked(
    read: Result<crate::codec::Work, String>,
    standing: &mut Standing,
) -> Option<String> {
    match read {
        Ok(work) => {
            standing.work = Some(work);
            None
        }
        Err(why) => Some(why),
    }
}

/// **The machines roster, folded** — `paned`'s terms again: the answer
/// replaces what was held and a failure keeps what was there.
pub(super) fn machined(
    read: Result<crate::codec::Machines, String>,
    standing: &mut Standing,
) -> Option<String> {
    match read {
        Ok(machines) => {
            standing.clients = Some(machines);
            None
        }
        Err(why) => Some(why),
    }
}
