//! **Whether this process may climb at all** (bl-c21d; DESIGN §21.10): the
//! one predicate a roving ladder parks on.
//!
//! Measured live: a backgrounded app has no DNS and no path — the entry's
//! name does not resolve and no bootstrap node does — so a ladder that
//! climbed there spent the re-punch (§21.9's arm) on nothing and rested, and
//! the next dial in front went straight to a fresh call. So a climb waits
//! until the process is **awake**, which is the platform's fact and never
//! this app's guess at it. Three things make it so, each reported by the
//! component the platform tells:
//!
//! - **the activity is in front** — `MainActivity`'s resume and pause, through
//!   `dev.yog.App`, the same place `open` asks (bl-f34f). A process nobody
//!   has told otherwise is awake: the host suite, and a process a job or a
//!   service started, where no activity will ever say.
//! - **the pocket service holds the process** (DESIGN §18) — a foreground
//!   service is above the platform's network threshold (§18.6), and the foot
//!   and the attention lane are exactly the climbs a pocketed phone makes.
//! - **a job is running** — the scheduled fetch (§17) holds a [`Hold`] for
//!   the length of its one run; the platform grants a job its network.
//!
//! A parked caller drops nothing and starts nothing: what the ladder holds
//! stays held, and the caller waits here until the process wakes. **A wake
//! after sleep is a return**, counted, and the ladder's first caller after
//! one clears the rest and re-arms the re-punch (`Ladder::wake`): the line
//! the platform killed in the background is evidence about the platform, not
//! about the engine's mapping, so the first climb back rides the port the
//! last call named before it pays for a call.

use super::{Ladder, Rove, say};
use crate::state::Watched;
use std::sync::atomic::Ordering;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

/// How often a parked caller re-reads the state with nothing changing — a
/// guard only, since every change wakes it.
const TICK: Duration = Duration::from_mins(1);

#[derive(Default)]
struct State {
    /// The activity paused and has not resumed.
    behind: bool,
    /// The pocket service is holding the process.
    service: bool,
    /// Runs the platform granted, each a [`Hold`].
    holds: usize,
    /// How many times the process woke after sleeping.
    returns: u64,
}

impl State {
    fn awake(&self) -> bool {
        !self.behind || self.service || self.holds > 0
    }
}

/// A process's lifecycle, as far as a climb cares.
pub struct Awake(Watched<State>);

/// A run the platform granted network: awake while held.
pub struct Hold(Arc<Awake>);

impl Awake {
    pub fn new() -> Arc<Awake> {
        Arc::new(Awake(Watched::new(State::default())))
    }

    /// The process's own — what the device's ladders climb on and what the
    /// platform's reports reach.
    pub fn process() -> Arc<Awake> {
        static PROCESS: OnceLock<Arc<Awake>> = OnceLock::new();
        Arc::clone(PROCESS.get_or_init(Awake::new))
    }

    /// The activity resumed (`true`) or paused (`false`).
    pub fn front(&self, front: bool) {
        self.set(&mut |s| s.behind = !front);
    }

    /// The pocket service started holding the process, or stopped.
    pub fn service(&self, held: bool) {
        self.set(&mut |s| s.service = held);
    }

    /// A granted run, awake until the [`Hold`] drops.
    pub fn hold(self: &Arc<Self>) -> Hold {
        self.set(&mut |s| s.holds += 1);
        Hold(Arc::clone(self))
    }

    /// Whether a climb may start now.
    pub fn awake(&self) -> bool {
        self.0.with(&mut |s| s.awake())
    }

    /// How many returns so far.
    pub(crate) fn returns(&self) -> u64 {
        self.0.with(&mut |s| s.returns)
    }

    /// Wait until awake, calling `parked` once if there is waiting to do;
    /// the count of returns then.
    pub(crate) fn park(&self, parked: &mut dyn FnMut()) -> u64 {
        let mut now = |s: &mut State| s.awake().then_some(s.returns);
        if let Some(returns) = self.0.with(&mut now) {
            return returns;
        }
        parked();
        self.0.wait(&mut now, TICK)
    }

    fn set(&self, change: &mut dyn FnMut(&mut State)) {
        self.0.with(&mut |s| {
            let was = s.awake();
            change(s);
            s.returns += u64::from(!was && s.awake());
        });
    }
}

impl Drop for Hold {
    fn drop(&mut self) {
        self.0.set(&mut |s| s.holds = s.holds.saturating_sub(1));
    }
}

impl Ladder {
    /// Park while the process sleeps; on the first look after a return,
    /// clear the rest and re-arm the entry's re-punch — the next climb then
    /// leaves from the port the last call named before it writes a call.
    pub(super) fn wake(&self, rove: &Rove) {
        let returns = rove
            .awake
            .park(&mut || self.voice.once("park", &say::parked()));
        if self.returns.swap(returns, Ordering::Relaxed) == returns {
            return;
        }
        self.voice.forget("park");
        self.settle();
        self.port.with(&mut |port| {
            if let Some(port) = port {
                port.armed = true;
            }
        });
        self.voice.say(&say::returned());
    }
}
