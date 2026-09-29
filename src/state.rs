//! **The process's one tool host** (DESIGN §18, bl-8bd0) — the crate's only
//! lock, and AGENTS.md rule 7's named home for it.
//!
//! **Why the host moved out of the frame.** Until this rung the `Host` handle
//! was a field of `shell::boot::Running`, so its lifetime was the *activity's*:
//! `android_main` returns when the activity is destroyed, the handle drops, and
//! the worker ends at its next publish. That is precisely the lifetime a
//! pocketed foot must not have — the whole of §18 is a service holding this
//! process open *past* the activity, and a host that died with the screen would
//! leave the service holding a lane nobody serves.
//!
//! **It also dissolves a race the frame-owned handle had.** An activity that is
//! destroyed and created again — the ordinary android relaunch — built a
//! *second* `Host` on the same certificate while the first worker was still
//! parked on its `invocations` read, and REMOTE §5.1's one-reader guard refuses
//! that second read naming this very device. One live host per process, held
//! here, is the invariant that makes the question unaskable.
//!
//! **"At most one LIVE host", and `alive` is the whole of the test.** A host
//! whose worker has returned is a host that is over — [`Health::Stopped`] is a
//! refusal no redial mends (`crate::transport::Wire`) — so [`hold`] replaces it
//! rather than refusing. Without that, a foot that met a refusal once could
//! never be started again inside the process it stopped in, and the operator's
//! own remedy (open the app) would do nothing at all.
//!
//! **One more resident since the punched wire (DESIGN §21), and not a second
//! chokepoint.** [`Slot`] is the one lock type every other module may hold —
//! a mutex behind a closure door, so the crate's lock inventory is still this
//! file and the ladder's shared RAM (the addresses it last saw, the endpoint
//! cache, the backoff, the held streams) is `Slot`s rather than a rule-7
//! carve-out apiece.
//!
//! **And [`Watched`], the one lock a caller may wait on** (bl-58a0): the
//! ladder's per-entry gate, where every caller on an entry waits while one
//! dial is in flight and while the line it landed is out. The same closure
//! door as [`Slot`], so still nothing is held across a wait.

use std::sync::{Condvar, Mutex, OnceLock, PoisonError};
use std::time::Duration;

use crate::host::{Host, Standing};

/// **The lock a module outside this file may hold.** A closure door rather
/// than a guard, so a caller can never hold it across a wait — every use is
/// a few lines under the lock and nothing blocks inside it.
pub(crate) struct Slot<T>(Mutex<T>);

impl<T> Slot<T> {
    pub(crate) fn new(value: T) -> Self {
        Self(Mutex::new(value))
    }

    /// Run `f` under the lock. A poisoned lock is taken as it stands —
    /// a thread that died mid-update left a value, and a value is better
    /// than a panic on every later dial.
    pub(crate) fn with<R>(&self, f: &mut dyn FnMut(&mut T) -> R) -> R {
        let mut held = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        f(&mut held)
    }
}

/// **A [`Slot`] a caller can wait on.** Every pass through [`with`](Self::with)
/// wakes every waiter, so a waiter re-asks its question after ANY change
/// rather than after the one it guessed would matter; and between askings the
/// lock is released for at most a tick, so a deadline read off an injected
/// clock is read at least that often.
pub(crate) struct Watched<T> {
    value: Mutex<T>,
    changed: Condvar,
}

impl<T> Watched<T> {
    pub(crate) fn new(value: T) -> Self {
        Self {
            value: Mutex::new(value),
            changed: Condvar::new(),
        }
    }

    /// Run `f` under the lock and wake every waiter — who can only look once
    /// the lock is released, so the wake may go first and they still read
    /// what `f` left.
    pub(crate) fn with<R>(&self, f: &mut dyn FnMut(&mut T) -> R) -> R {
        let mut held = self.value.lock().unwrap_or_else(PoisonError::into_inner);
        self.changed.notify_all();
        f(&mut held)
    }

    /// Ask `f` under the lock until it answers, waiting between askings for
    /// a change or for `tick`, whichever comes first.
    pub(crate) fn wait<R>(&self, f: &mut dyn FnMut(&mut T) -> Option<R>, tick: Duration) -> R {
        let mut held = self.value.lock().unwrap_or_else(PoisonError::into_inner);
        loop {
            let waited = match f(&mut held) {
                Some(answer) => return answer,
                None => self.changed.wait_timeout(held, tick),
            };
            (held, _) = waited.unwrap_or_else(PoisonError::into_inner);
        }
    }
}

/// The one slot. `OnceLock` rather than a `LazyLock` initializer because the
/// slot's *contents* are what varies; the mutex itself is created once and
/// never replaced.
fn slot() -> &'static Mutex<Option<Host>> {
    static HELD: OnceLock<Mutex<Option<Host>>> = OnceLock::new();
    HELD.get_or_init(|| Mutex::new(None))
}

/// **Take up the process's host**, unless a live one already stands. The
/// answer is whether `host` was taken up: `false` means one was already
/// serving and this one is dropped on the way out, which stops its worker at
/// the next loop boundary.
pub fn hold(host: Host) -> bool {
    let mut held = slot().lock().unwrap_or_else(PoisonError::into_inner);
    if held.as_mut().is_some_and(Host::alive) {
        return false;
    }
    *held = Some(host);
    true
}

/// **Whether this process already holds a LIVE host** — the question a caller
/// asks before it BUILDS one (§18.8). [`hold`] answers it too, but only after
/// a host exists: a `Host` starts its worker the moment it is made, so a
/// second one made to be refused has already dialled and advertised, and
/// REMOTE §5.1's one-reader guard would refuse this device in its own name.
/// §18.1 made that question unaskable and this is what keeps it so.
pub fn holding() -> bool {
    slot()
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_mut()
        .is_some_and(Host::alive)
}

/// What the process's host stands at, or `None` when it holds none — a cold
/// device, or one whose material would not build a foot. The frame paints this
/// and the pocket's notification is written from it, which is the single home
/// this rung needed the standing to have.
pub fn standing() -> Option<Standing> {
    slot()
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_mut()
        .map(Host::standing)
}

#[cfg(test)]
mod tests;
