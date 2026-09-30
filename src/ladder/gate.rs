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
//! **A look in the pool is a line out** (bl-2ba5). Taking a held line hands
//! it over from its holder thread, which wakes on its own tick — up to a
//! quarter second in which the pool is empty and, were nothing counted, no
//! line out and no dial in flight. A caller arriving in that window read
//! the entry as idle and dialled a second line beside the first; measured
//! live as a re-punch three seconds after a punch landed, while the line was
//! still up. So a caller counts itself out *before* it looks ([`Lease`]),
//! and a look that found nothing takes the count back without a change —
//! nothing a waiter waits for appeared.
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
/// `beside` says the dial was admitted past [`LENT`] with a line still out:
/// the entry HAS a line, so what this dial wants is a second one.
pub(crate) struct Dialling {
    gate: Arc<Gate>,
    pub(crate) beside: bool,
}

/// **A punched line out with one caller.** Dropped — after the line went
/// back to the pool, or with the line — it is back, and the waiters look.
pub struct Lease {
    gate: Arc<Gate>,
    /// Whether its return is a change: a look that found nothing is not.
    returned: bool,
}

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
                    return Some(None);
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
                Some(Some(s.lent > 0))
            },
            TICK,
        );
        match dial {
            Some(beside) => Turn::Dial(Dialling {
                gate: Arc::clone(self),
                beside,
            }),
            None => Turn::Look,
        }
    }

    /// A line out with this caller — counted before the caller looks in the
    /// pool, so the look itself is never an idle entry. Going out is not a
    /// change: no waiter waits for a line to leave.
    pub(crate) fn lend(self: &Arc<Self>) -> Lease {
        self.0.with(&mut |s| s.lent += 1);
        Lease {
            gate: Arc::clone(self),
            returned: true,
        }
    }
}

impl Dialling {
    /// The dial landed a punched line: it is out with the dialler.
    pub(crate) fn punched(self) -> Lease {
        self.gate.lend()
    }
}

impl Lease {
    /// The look found the pool empty: the count goes back, and nothing
    /// changed that a waiter was waiting for.
    pub(crate) fn missed(mut self) {
        self.returned = false;
    }
}

impl Drop for Dialling {
    fn drop(&mut self) {
        self.gate.0.with(&mut |s| {
            s.dialling = false;
            s.changes += 1;
        });
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        let returned = u64::from(self.returned);
        self.gate.0.with(&mut |s| {
            s.lent = s.lent.saturating_sub(1);
            s.changes += returned;
        });
    }
}

#[cfg(test)]
mod tests;
