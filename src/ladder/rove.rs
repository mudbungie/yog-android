//! What a roving entry climbs with, and the clock it climbs on — split from
//! the ladder so the rungs are the policy and this is only its inputs.

use super::say::{self, Say};
use super::{Awake, Network};
use crate::dht::Config;
use crate::rendezvous::Pairing;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Time, injected — so the silence bound and the backoff walk in a test
/// without waiting for them.
pub trait Clock: Send + Sync {
    fn now(&self) -> Instant;
    /// Epoch seconds, for the call's rising `seq`.
    fn unix(&self) -> i64;
}

/// The device's own clock.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }

    fn unix(&self) -> i64 {
        crate::roster::now_unix()
    }
}

/// What a roving entry climbs with: the pairing, and the commons to walk.
/// Every field a test can point at a fake node and shorten.
pub struct Rove {
    pub pairing: Pairing,
    /// Bootstrap nodes as names, resolved on the dialling thread.
    pub bootstrap: Vec<String>,
    pub config: Config,
    /// How long a punch keeps sending SYNs.
    pub window: Duration,
    /// Where this box sends from and when that changed — what the call
    /// names and what every cache is keyed by (`ladder::network`, DESIGN
    /// §21.11); the device's is [`Network::process`], the suite's one it
    /// reports to and whose fallback it moves.
    pub network: Arc<Network>,
    /// Where the rungs' lines go (`ladder::say`); the device's is
    /// [`say::logcat`], the suite's a sink it reads back.
    pub say: Say,
    /// Whether the process may climb now (`ladder::awake`); the device's is
    /// [`Awake::process`], the suite's one it toggles.
    pub awake: Arc<Awake>,
}

/// The engine reads its inbox every fifteen seconds and punches for twenty
/// after it reads a call, so a client window that stopped at twenty would
/// miss the engine's whole window when the poll came late. The sum — the
/// figure thrall states too (thrall bl-0a8b) — and a default to revisit on
/// evidence (yog REMOTE §13.7 ruling 3).
const WINDOW: Duration = Duration::from_secs(35);

impl Rove {
    /// The production shape over `pairing`: the engine's own four mainline
    /// routers (a silent router should cost a quarter of the roster, not
    /// half — yog bl-9408), the engine's walk, and [`WINDOW`].
    pub fn mainline(pairing: Pairing) -> Rove {
        Rove {
            pairing,
            bootstrap: [
                "router.bittorrent.com:6881",
                "dht.transmissionbt.com:6881",
                "router.utorrent.com:6881",
                "dht.aelitis.com:6881",
            ]
            .map(str::to_owned)
            .to_vec(),
            config: Config::default(),
            window: WINDOW,
            network: Network::process(),
            say: say::logcat(),
            awake: Awake::process(),
        }
    }
}
