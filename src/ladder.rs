//! **The dial ladder** (yog REMOTE §13.4; the phone's half of yog bl-0247):
//! the four ways this device reaches its engine, tried in the order they
//! cost, on every dial. *The live held connection; the entry's direct
//! `address` where one exists; a re-punch at the RAM-cached endpoints, which
//! costs no DHT round trip; the full rendezvous.* What worked stays RAM for
//! the run — never disk — which is the runtime half of REMOTE §8's `:0`
//! discipline.
//!
//! **Severable by material** (REMOTE §13.4). An entry whose material holds
//! no [`Pairing`] has a ladder of two rungs — a held stream never exists for
//! it, and the direct dial is today's dial, byte for byte, down to the
//! sentence a socket that would not open earns. Only an entry that roves
//! ever touches the DHT, and only at the moment it wants a connection: the
//! phone pays nothing for the machinery when idle, because only the engine
//! polls.
//!
//! **Redial is the feature** (DESIGN §18.6, now load-bearing; §21.3). A
//! dropped channel re-enters here on the caller's own ladder of naps
//! (`host::serve`), and the two rungs that cost seconds — a punch window, a
//! walk of the commons — are behind one `Backoff`: a climb that failed at
//! both is not climbed again until it expires, doubling to a minute, so a
//! phone with no network settles instead of walking a dark commons per dial.
//! Three things reset it, each the evidence it wants: a rung that connected,
//! a stream that served, and the network changing (`network`: the platform's
//! report, or the addresses this box sends from moving) — which also drops
//! every held stream (dead mappings after a change) and the punch port with
//! its cached endpoints, so the next dial rendezvouses afresh from a fresh
//! port.
//!
//! **One climb per entry at a time** (bl-58a0): every seat on an entry
//! holds the same ladder (`entries`), and every caller passes its gate
//! before it climbs — one dial in flight, the rest waiting on it and sharing
//! the line it lands. `gate` says why a second concurrent climb is never
//! right.
//!
//! **The TLS is the same TLS whichever rung answered.** This module hands
//! back a socket; `transport::Seat` runs the ordinary inner mTLS over it,
//! verifying the same engine name off the same `address`, and the engine
//! reads the same leaf. §4 is untouched.

mod awake;
mod backoff;
mod climb;
mod entries;
mod gate;
pub mod held;
mod network;
mod rove;
pub mod say;

pub use awake::{Awake, Hold};
pub(crate) use entries::ladder as entry;
pub use gate::Lease;
pub use held::Held;
pub use network::Network;
pub use rove::{Clock, Rove, SystemClock};
pub use say::Say;

use crate::state::Slot;
use backoff::Backoff;
use gate::{Gate, Turn};
use say::Voice;
use std::net::{IpAddr, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::time::Duration;

/// How long a direct dial is given before the ladder moves on. Only spent
/// where there is a rung to move on to; an entry with an address alone
/// dials as it always has.
const DIRECT: Duration = Duration::from_secs(5);

/// What a dial got: a stream with its preface already spent, or a socket
/// the caller runs the whole wire over. A punched line — held or fresh — is
/// out on a [`Lease`] until it goes back to the pool or is dropped; a
/// dialled socket is one ask's and is never kept.
pub enum Conn {
    Held(Box<Held>, Lease),
    Punched(TcpStream, Lease),
    Dialled(TcpStream),
}

/// One entry's ladder.
pub struct Ladder {
    address: String,
    rove: Option<Rove>,
    clock: Arc<dyn Clock>,
    /// The network generation this ladder last acted on (`network`).
    seen: AtomicU64,
    /// The entry's punch port and what a call from it found (`entries`).
    port: entries::Bound,
    backoff: Slot<Backoff>,
    held: held::Pool,
    gate: Arc<Gate>,
    last_seq: AtomicI64,
    /// The process's returns this ladder has seen (`awake`).
    returns: AtomicU64,
    /// What the rungs say (`say`): the rove's sink, or nothing.
    voice: Voice,
}

impl Ladder {
    pub fn new(address: String, rove: Option<Rove>, clock: Arc<dyn Clock>) -> Ladder {
        Ladder::on(address, rove, clock, Arc::new(Slot::new(None)))
    }

    /// A ladder climbing from `port` — the entry's, shared with every
    /// ladder before and after this one on it (`entries`).
    pub(crate) fn on(
        address: String,
        rove: Option<Rove>,
        clock: Arc<dyn Clock>,
        port: entries::Bound,
    ) -> Ladder {
        let sink = rove
            .as_ref()
            .map_or_else(say::quiet, |r| Arc::clone(&r.say));
        let returns = rove.as_ref().map_or(0, |r| r.awake.returns());
        let seen = rove.as_ref().map_or(0, |r| r.network.generation());
        Ladder {
            address,
            rove,
            clock,
            seen: AtomicU64::new(seen),
            port,
            backoff: Slot::new(Backoff::new()),
            held: held::Pool::new(),
            gate: Gate::new(),
            last_seq: AtomicI64::new(0),
            returns: AtomicU64::new(returns),
            voice: Voice::new(sink),
        }
    }

    /// Climb. The `Err` is the sentence of the rung that stopped the climb —
    /// the direct dial's where nothing roves, the rendezvous's where it does.
    pub fn connect(&self) -> Result<Conn, String> {
        let Some(rove) = &self.rove else {
            let tcp = TcpStream::connect(&self.address)
                .map_err(|e| format!("connect {}: {e}", self.address))?;
            return Ok(Conn::Dialled(tcp));
        };
        // Nothing climbs while the process sleeps (`awake`, bl-c21d).
        self.wake(rove);
        self.notice(rove);
        // Rung 1, then the gate: a caller that is not the one dialling looks
        // in the pool again after every change, until a line is there for it
        // or the dial is its own.
        // The caller counts itself out before it looks (`gate`, bl-2ba5).
        let dialling = loop {
            let seen = self.gate.seen();
            let lease = self.gate.lend();
            if let Some(held) = self.held.take() {
                return Ok(Conn::Held(Box::new(held), lease));
            }
            lease.missed();
            if let Turn::Dial(dialling) = self.gate.turn(seen, self.clock.as_ref()) {
                break dialling;
            }
        };
        let direct = match direct(&self.address, &self.voice) {
            Ok(tcp) => {
                self.settle();
                return Ok(Conn::Dialled(tcp));
            }
            Err(said) => said,
        };
        let now = self.clock.now();
        if let Some(said) = self.backoff.with(&mut |b| b.resting(now)) {
            self.voice.once("rest", &say::resting());
            return Err(format!("{direct}; {said}"));
        }
        self.voice.forget("rest");
        match self.climb(rove, !dialling.beside) {
            Ok(tcp) => {
                self.settle();
                Ok(Conn::Punched(tcp, dialling.punched()))
            }
            Err(said) => {
                self.backoff.with(&mut |b| b.failed(now, said.clone()));
                Err(format!("{direct}; {said}"))
            }
        }
    }

    /// A finished ask hands its punched stream back for the next one — and
    /// a line served is the evidence that re-arms the re-punch
    /// (`entries::Port`).
    pub fn keep(&self, held: Held) {
        self.settle();
        self.port.with(&mut |port| {
            if let Some(port) = port {
                port.armed = true;
            }
        });
        self.voice.say(&say::kept());
        self.held
            .keep(held, Arc::clone(&self.clock), self.voice.sink());
    }

    /// How many punched streams are being held.
    pub fn held(&self) -> usize {
        self.held.len()
    }

    /// The backoff returns to its floor: something connected or served.
    fn settle(&self) {
        self.backoff.with(&mut Backoff::settle);
    }

    /// **A network change** (DESIGN §21.3, §21.11): the generation moved
    /// since this ladder last looked (`network`). That drops every held
    /// stream — a dead mapping after a change — and clears the backoff; the
    /// punch port goes with the endpoints cached beside it by being of the
    /// old generation (`entries::Port`): the engine's NAT holds a mapping
    /// toward the address and port the last call named, and this box no
    /// longer sends from that address. Called on every dial, just before a
    /// call names the addresses, and at once when the platform reports;
    /// answers the generation now and the addresses in it.
    fn notice(&self, rove: &Rove) -> (u64, Vec<IpAddr>) {
        let (generation, mine) = rove.network.now();
        if self.seen.swap(generation, Ordering::Relaxed) != generation {
            let held = self.held.len();
            self.held.clear();
            self.settle();
            self.voice.say(&say::moved(&mine, held));
        }
        (generation, mine)
    }

    /// The platform said the network changed (`Network::changed`).
    pub(crate) fn noticed(&self) {
        if let Some(rove) = &self.rove {
            self.notice(rove);
        }
    }
}

/// The direct rung: every address the entry resolves to, each given
/// [`DIRECT`], in the sentence today's dial already earns — and said once
/// per change of outcome, the families tried and how each ended.
fn direct(address: &str, voice: &Voice) -> Result<TcpStream, String> {
    let mut refused = format!("connect {address}: no address resolved");
    let mut tried = Vec::new();
    let resolved = address.to_socket_addrs().map_err(|e| {
        voice.once("direct", &say::unresolved());
        format!("connect {address}: {e}")
    })?;
    for at in resolved {
        match TcpStream::connect_timeout(&at, DIRECT) {
            Ok(tcp) => {
                tried.push((at.ip(), "connected"));
                voice.once("direct", &say::direct(&tried));
                return Ok(tcp);
            }
            Err(e) => {
                tried.push((at.ip(), say::outcome(&e)));
                refused = format!("connect {address}: {e}");
            }
        }
    }
    voice.once("direct", &say::direct(&tried));
    Err(refused)
}

#[cfg(test)]
mod tests;
