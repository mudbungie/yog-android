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
//! **One more resident, and not a second chokepoint** (DESIGN §21; yog
//! REMOTE §15.3): [`worked`], the endpoint of each identity's address list
//! that answered earlier in this run. It is here rather than on a seat
//! because a seat is opened per pass and per lane — what worked outlives
//! every one of them and is shared across all of them. RAM and never disk, so
//! a phone that moved between networks starts from the list's own order on
//! its next start.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock, PoisonError};

use crate::host::{Host, Standing};

/// What worked, per identity — keyed by the leaf it is about.
fn table() -> &'static Mutex<HashMap<String, String>> {
    static WORKED: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    WORKED.get_or_init(Mutex::default)
}

/// The endpoint that answered for `key` earlier in this run, if one did.
pub(crate) fn worked(key: &str) -> Option<String> {
    let table = table().lock().unwrap_or_else(PoisonError::into_inner);
    table.get(key).cloned()
}

/// `address` answered for `key`: it is tried first for the rest of the run.
pub(crate) fn works(key: &str, address: &str) {
    let mut table = table().lock().unwrap_or_else(PoisonError::into_inner);
    table.insert(key.to_owned(), address.to_owned());
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
