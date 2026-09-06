//! **The held attention lane** (DESIGN §17.6; yog REMOTE §14 rung 2): the
//! read this seat keeps standing from a pocket, and the frame that becomes a
//! wake.
//!
//! **It is rung 1's decision on a faster clock, and it is deliberately not a
//! second one.** The rise rule, the memory file and the words a wake carries
//! are all [`super`]'s, unchanged: what rose above what was last announced,
//! recorded whether or not anything was said. Only the ASK differs — rung 1
//! performs `Query::Workspaces` on the platform's schedule, and this holds
//! `Query::Attention` open, so a change reaches the operator at the engine's
//! own write cadence rather than at a fifteen-minute floor.
//!
//! **The two asks count the same thing, and that is why one memory serves
//! both.** A `workspaces` row's `attention` is the engine's count of agents in
//! that workspace whose §6 predicate fires (yog `answer::chrome::workspace_stats`),
//! and the attention queue is every agent whose predicate fires across every
//! workspace (`answer::queue::queue`) — one rollup and one flattening of ONE
//! derivation. So a queue frame folded per workspace is the very number rung 1
//! stores, and the two rungs cannot double-announce a rise or hide one from
//! each other. That is the whole reason this is a module beside `attention`
//! rather than a mechanism of its own.
//!
//! **A frame is the wake, and the first rise ends the lane.** The engine
//! writes a frame whenever this asker's answer changes (REMOTE §14.1), so
//! reading until something rises IS the wake — no second read, and nothing
//! derived here that the engine did not say. The lane is dropped at that
//! point rather than held through: a lane whose frames replace has nothing to
//! reconcile on the next dial, so redialling costs one connection and buys a
//! caller that is never holding a socket while it posts.
//!
//! **Every failure is silence**, exactly as rung 1's is: no material, an
//! engine that will not answer, a frame this end cannot read. A phone in a
//! pocket must never nag about network.

use std::path::Path;

use super::{Counts, Notice, WIRE};
use crate::codec::reply::{self, Reply};
use crate::codec::{Ask, Gesture, QueueRow, encode};
use crate::transport::Seat;

/// **One lane's life**: dial, hold, and answer the first rise it hears.
/// `None` is silence — the hold ended with nothing new, or nothing this end
/// could use — and the caller rests before dialling again.
pub fn wake(dir: &Path) -> Option<Notice> {
    let material = crate::material::read_dir(&dir.join(WIRE)).ok().flatten()?;
    let seat = Seat::open(&material).ok()?;
    let (open, _hangup) = seat.hold(&encode(&Gesture::Ask(Ask::Attention))).ok()?;
    let mut woken = None;
    let _ = open.each(&mut |frame| {
        // A frame of another kind — or one this build cannot read — ends the
        // lane rather than being guessed at (REMOTE §3's third rule); the
        // rest before the next dial is what keeps that from spinning.
        let Ok(Ok(Reply::Attention(rows))) = reply::decode(&frame) else {
            return false;
        };
        let now = queued(&rows);
        woken = super::risen(&now, &super::read_seen(dir));
        super::write_seen(dir, &now);
        // Stop at the first rise; go on listening while nothing has risen.
        woken.is_none()
    });
    woken
}

/// The queue folded into the fact rung 1 keeps: how many conversations each
/// workspace is waiting on. A workspace with no row is absent rather than
/// zero, which is `counts`' own rule one shape along — and it is what makes
/// an emptied queue read as a FALL rather than as no answer.
fn queued(rows: &[QueueRow]) -> Counts {
    let mut counts = Counts::new();
    for row in rows {
        *counts.entry(row.workspace.clone()).or_insert(0) += 1;
    }
    counts
}

#[cfg(test)]
mod tests;
