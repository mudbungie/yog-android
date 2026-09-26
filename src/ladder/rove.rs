//! What a roving entry climbs with, and the clock it climbs on — split from
//! the ladder so the rungs are the policy and this is only its inputs.

use crate::dht::Config;
use crate::rendezvous::{Pairing, punch};
use std::net::IpAddr;
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
    /// The addresses this box would send from — what the call names and
    /// what a network change moves (DESIGN §21.3). A function so the suite
    /// can move it; the device's is [`punch::local_ips`].
    pub addresses: fn() -> Vec<IpAddr>,
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
            addresses: punch::local_ips,
        }
    }
}
