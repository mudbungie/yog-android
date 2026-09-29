//! **One dial in flight per entry** (bl-58a0; yog REMOTE §13.3–§13.4): the
//! gate every caller on a roving entry passes before it climbs.
//!
//! **Why a second concurrent climb is never right.** A pairing's inbox is ONE
//! item — the newest `seq` wins — and the engine reads it once a poll and
//! punches the call it finds. Two ladders climbing one entry at once each
//! write their own call from their own punch port, so one is answered per
//! poll and the other waits out its whole window for nothing; measured live
//! on the phone, where the roster read, the conversations read and the
//! attention fetch each climbed their own. So a dial is one caller's, and
//! every other caller on the entry **waits for it and shares the line it
//! lands**: rung 1 — the held line — is already the entry's, so the line a
//! finished ask hands back is the next caller's.
//!
//! **What a waiter waits on**, re-asked after every change:
//!
//! - a dial in flight — wait; its end is a change.
//! - a punched line out with a caller ([`Lease`]) — wait for it to come back
//!   to the pool, which is a change; one that is dropped instead is a change
//!   too, and then nothing is out and the waiter dials. A line held by a
//!   parked read (the foot's `invocations`, the attention lane) never comes
//!   back, so a waiter that has watched lines out for [`LENT`] with no dial
//!   in flight dials its own — serially still, since the gate admits one.
//! - nothing in flight and nothing out — this caller dials.
//!
//! A dialled socket (the direct rung) is one ask's and is never lent, so it
//! only ends the dial. An entry that does not rove has no gate: its dial is
//! a plain connect, as it always was.

use super::Clock;
use crate::state::Watched;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// How long a waiter watches a punched line out with another caller before
/// it dials its own: an ask is answered well inside it, a parked read never
/// is, and a rendezvous costs three times as much. A default to revisit on
/// evidence.
pub(crate) const LENT: Duration = Duration::from_secs(10);

/// How often a waiter reads the clock while nothing changes.
const TICK: Duration = Duration::from_millis(250);

#[derive(Default)]
struct State {
    dialling: bool,
    lent: usize,
    /// Rises on every change a waiter might be waiting for.
    changes: u64,
}

/// One entry's gate.
pub(crate) struct Gate(Watched<State>);

/// What a caller does next: look in the pool again, or climb.
pub(crate) enum Turn {
    Look,
    Dial(Dialling),
}

/// The one dial in flight. Dropped, it ends the dial having lent nothing.
pub(crate) struct Dialling(Arc<Gate>);

/// **A punched line out with one caller.** Dropped — after the line went
/// back to the pool, or with the line — it is back, and the waiters look.
pub struct Lease(Arc<Gate>);

impl Gate {
    pub(crate) fn new() -> Arc<Gate> {
        Arc::new(Gate(Watched::new(State::default())))
    }

    /// The change count, read before a look in the pool: a change after it
    /// sends the caller to look again rather than to wait.
    pub(crate) fn seen(&self) -> u64 {
        self.0.with(&mut |s| s.changes)
    }

    /// Wait for this caller's turn, as the module doc spells it.
    pub(crate) fn turn(self: &Arc<Self>, seen: u64, clock: &dyn Clock) -> Turn {
        let mut since: Option<Instant> = None;
        let dial = self.0.wait(
            &mut |s| {
                if s.changes != seen {
                    return Some(false);
                }
                if s.dialling {
                    since = None;
                    return None;
                }
                let now = clock.now();
                let out = s.lent > 0 && now.duration_since(*since.get_or_insert(now)) < LENT;
                if out {
                    return None;
                }
                s.dialling = true;
                Some(true)
            },
            TICK,
        );
        if dial {
            Turn::Dial(Dialling(Arc::clone(self)))
        } else {
            Turn::Look
        }
    }

    /// A held line taken out of the pool is out with this caller.
    pub(crate) fn lend(self: &Arc<Self>) -> Lease {
        self.0.with(&mut |s| {
            s.lent += 1;
            s.changes += 1;
        });
        Lease(Arc::clone(self))
    }
}

impl Dialling {
    /// The dial landed a punched line: it is out with the dialler.
    pub(crate) fn punched(self) -> Lease {
        self.0.lend()
    }
}

impl Drop for Dialling {
    fn drop(&mut self) {
        self.0.0.with(&mut |s| {
            s.dialling = false;
            s.changes += 1;
        });
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        self.0.0.with(&mut |s| {
            s.lent = s.lent.saturating_sub(1);
            s.changes += 1;
        });
    }
}

#[cfg(test)]
mod tests;
