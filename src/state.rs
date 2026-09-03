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

use std::sync::{Mutex, OnceLock, PoisonError};

use crate::host::{Host, Standing};

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
