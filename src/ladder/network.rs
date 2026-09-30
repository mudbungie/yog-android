//! **Where this box sends from, and when that changed** (bl-792e; DESIGN
//! §21.11): the one source of the call's own addresses and of the network
//! generation every cache on the ladder is keyed by.
//!
//! Measured live: a ladder that learned of a network change by diffing the
//! routing table at its next dial learned it 34–53 s late, and the call it
//! wrote first after the drop still named the old network's addresses. So
//! the change is **the platform's fact**: `ConnectivityManager`'s
//! default-network callback, in `dev.yog.App`, reports the default network's
//! own addresses through the native `App.network` the moment it moves, and
//! the report both **bumps the generation** and **names the addresses** —
//! the default network's link, never every interface's. A ladder watching
//! the network hears the bump at once: its held lines drop and its rest
//! clears then, not at its next dial (`Ladder::notice`), and its punch port
//! is a port of the generation it was bound in (`entries::Port`).
//!
//! **The routing-table read is the fallback, not a second source.** Until
//! the platform reports — the host suite, a process a job started — the
//! addresses are [`punch::local_ips`]' UDP "connect", and a read that differs
//! from the last is a change of generation exactly as a report is. Once the
//! platform has reported, it is the one source.
//!
//! **Only an address a peer could reach is ever named** ([`reachable`]):
//! a ULA, a link-local, the shared (carrier-grade and overlay) space and the
//! translator's own range are never reachable from outside, whatever yog
//! bl-f612 rules for an overlay.

use super::Ladder;
use crate::rendezvous::punch;
use crate::state::Slot;
use std::net::IpAddr;
use std::sync::{Arc, OnceLock, Weak};

#[derive(Default)]
struct State {
    generation: u64,
    /// The platform's last report, once there has been one.
    platform: Option<Vec<IpAddr>>,
    /// The last addresses read from either source.
    last: Option<Vec<IpAddr>>,
}

impl State {
    /// Take `mine` as the addresses now: a set that differs from the last is
    /// a new generation, and the first set ever read is not a change.
    fn observe(&mut self, mine: &[IpAddr]) -> bool {
        let moved = self.last.as_deref().is_some_and(|was| was != mine);
        self.generation += u64::from(moved);
        self.last = Some(mine.to_vec());
        moved
    }
}

/// A box's network, as far as a ladder cares.
pub struct Network {
    state: Slot<State>,
    fallback: fn() -> Vec<IpAddr>,
    ladders: Slot<Vec<Weak<Ladder>>>,
}

impl Network {
    /// A network read from `fallback` until a platform report arrives.
    pub fn new(fallback: fn() -> Vec<IpAddr>) -> Arc<Network> {
        Arc::new(Network {
            state: Slot::new(State::default()),
            fallback,
            ladders: Slot::new(Vec::new()),
        })
    }

    /// The process's own — what the device's ladders climb on and what the
    /// platform's reports reach.
    pub fn process() -> Arc<Network> {
        static PROCESS: OnceLock<Arc<Network>> = OnceLock::new();
        Arc::clone(PROCESS.get_or_init(|| Network::new(punch::local_ips)))
    }

    /// **The platform's report**: the default network's addresses, one per
    /// line as the platform spells them — empty when there is no default
    /// network. A line that is not an address (a link-local's `%zone`) is
    /// dropped, and so is every address [`reachable`] refuses. A report that
    /// moves the set tells every ladder watching, at once.
    pub fn changed(&self, addresses: &str) {
        let mine = reachable(
            addresses
                .lines()
                .filter_map(|l| l.trim().parse().ok())
                .collect(),
        );
        let moved = self.state.with(&mut |s| {
            s.platform = Some(mine.clone());
            s.observe(&mine)
        });
        if moved {
            let live: Vec<Arc<Ladder>> = self.ladders.with(&mut |ladders| {
                ladders.retain(|l| l.strong_count() > 0);
                ladders.iter().filter_map(Weak::upgrade).collect()
            });
            for ladder in live {
                ladder.noticed();
            }
        }
    }

    /// The generation now, and the addresses this box sends from in it —
    /// the platform's, or the fallback's read, which moves the generation
    /// when it differs from the last.
    pub(crate) fn now(&self) -> (u64, Vec<IpAddr>) {
        let reported = self.state.with(&mut |s| s.platform.is_some());
        let read = if reported {
            Vec::new()
        } else {
            reachable((self.fallback)())
        };
        self.state.with(&mut |s| {
            let mine = s.platform.clone().unwrap_or_else(|| read.clone());
            s.observe(&mine);
            (s.generation, mine)
        })
    }

    /// The generation as last observed, reading nothing.
    pub(crate) fn generation(&self) -> u64 {
        self.state.with(&mut |s| s.generation)
    }

    /// Tell `ladder` of every change from now on, for as long as it lives.
    pub(crate) fn watch(&self, ladder: &Arc<Ladder>) {
        self.ladders.with(&mut |ladders| {
            ladders.retain(|l| l.strong_count() > 0);
            ladders.push(Arc::downgrade(ladder));
        });
    }
}

/// How far an address reaches — what the ladder's lines say of a v6, and
/// what [`reachable`] judges by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Scope {
    Loopback,
    LinkLocal,
    /// `fc00::/7`: the overlay's and every other private v6.
    Ula,
    /// `100.64/10`: carrier-grade NAT, and the overlay's v4.
    Shared,
    /// `192.0.0/24`: the 464XLAT translator's own side (RFC 7335).
    Translator,
    /// RFC 1918.
    Private,
    Global,
}

impl Scope {
    pub(crate) fn of(ip: IpAddr) -> Scope {
        match ip {
            IpAddr::V6(v6) => {
                let [head, ..] = v6.segments();
                if v6.is_loopback() {
                    Scope::Loopback
                } else if head & 0xffc0 == 0xfe80 {
                    Scope::LinkLocal
                } else if head & 0xfe00 == 0xfc00 {
                    Scope::Ula
                } else {
                    Scope::Global
                }
            }
            IpAddr::V4(v4) => {
                let [a, b, c, _] = v4.octets();
                if v4.is_loopback() {
                    Scope::Loopback
                } else if v4.is_link_local() {
                    Scope::LinkLocal
                } else if a == 100 && b & 0xc0 == 64 {
                    Scope::Shared
                } else if (a, b, c) == (192, 0, 0) {
                    Scope::Translator
                } else if v4.is_private() {
                    Scope::Private
                } else {
                    Scope::Global
                }
            }
        }
    }

    /// The class's word in a line.
    pub(crate) fn word(self) -> &'static str {
        match self {
            Scope::Loopback => "loopback",
            Scope::LinkLocal => "link-local",
            Scope::Ula => "ula",
            Scope::Shared => "shared",
            Scope::Translator => "translator",
            Scope::Private => "private",
            Scope::Global => "global",
        }
    }
}

/// The addresses a peer outside could ever reach, sorted and each once — so
/// two reads of one network in another order are one set.
pub(crate) fn reachable(ips: Vec<IpAddr>) -> Vec<IpAddr> {
    let unreachable = [
        Scope::LinkLocal,
        Scope::Ula,
        Scope::Shared,
        Scope::Translator,
    ];
    let mut kept: Vec<IpAddr> = ips
        .into_iter()
        .filter(|ip| !unreachable.contains(&Scope::of(*ip)))
        .collect();
    kept.sort_unstable();
    kept.dedup();
    kept
}
