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
//! a stream that served, and the addresses this box sends from moving —
//! which also drops every held stream (dead mappings after an address
//! change), so the next dial re-punches at the cached endpoints first and
//! rendezvouses afresh only if that fails.
//!
//! **The TLS is the same TLS whichever rung answered.** This module hands
//! back a socket; `transport::Seat` runs the ordinary inner mTLS over it,
//! verifying the same engine name off the same `address`, and the engine
//! reads the same leaf. §4 is untouched.

mod backoff;
pub mod held;
mod rove;

pub use held::Held;
pub use rove::{Clock, Rove, SystemClock};

use crate::dht::{Dht, Udp};
use crate::rendezvous::{call, punch};
use crate::state::Slot;
use backoff::Backoff;
use std::net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;

/// How long a direct dial is given before the ladder moves on. Only spent
/// where there is a rung to move on to; an entry with an address alone
/// dials as it always has.
const DIRECT: Duration = Duration::from_secs(5);

/// What a dial got: a stream with its preface already spent, or a socket
/// the caller runs the whole wire over — and whether it was punched, which
/// is what decides if a finished ask keeps it.
pub enum Conn {
    Held(Box<Held>),
    Fresh { tcp: TcpStream, punched: bool },
}

/// One entry's ladder.
pub struct Ladder {
    address: String,
    rove: Option<Rove>,
    clock: Arc<dyn Clock>,
    /// The addresses the last dial saw this box send from.
    seen: Slot<Option<Vec<IpAddr>>>,
    cache: Slot<Vec<SocketAddr>>,
    backoff: Slot<Backoff>,
    held: held::Pool,
    last_seq: AtomicI64,
}

impl Ladder {
    pub fn new(address: String, rove: Option<Rove>, clock: Arc<dyn Clock>) -> Ladder {
        Ladder {
            address,
            rove,
            clock,
            seen: Slot::new(None),
            cache: Slot::new(Vec::new()),
            backoff: Slot::new(Backoff::new()),
            held: held::Pool::new(),
            last_seq: AtomicI64::new(0),
        }
    }

    /// Climb. The `Err` is the sentence of the rung that stopped the climb —
    /// the direct dial's where nothing roves, the rendezvous's where it does.
    pub fn connect(&self) -> Result<Conn, String> {
        let Some(rove) = &self.rove else {
            let tcp = TcpStream::connect(&self.address)
                .map_err(|e| format!("connect {}: {e}", self.address))?;
            return Ok(Conn::Fresh {
                tcp,
                punched: false,
            });
        };
        let mine = (rove.addresses)();
        self.notice_network(&mine);
        if let Some(held) = self.held.take() {
            return Ok(Conn::Held(Box::new(held)));
        }
        let direct = match direct(&self.address) {
            Ok(tcp) => {
                self.settle();
                return Ok(Conn::Fresh {
                    tcp,
                    punched: false,
                });
            }
            Err(said) => said,
        };
        let now = self.clock.now();
        if let Some(said) = self.backoff.with(&mut |b| b.resting(now)) {
            return Err(format!("{direct}; {said}"));
        }
        match self.climb(rove, mine) {
            Ok(tcp) => {
                self.settle();
                Ok(Conn::Fresh { tcp, punched: true })
            }
            Err(said) => {
                self.backoff.with(&mut |b| b.failed(now, said.clone()));
                Err(format!("{direct}; {said}"))
            }
        }
    }

    /// A finished ask hands its punched stream back for the next one.
    pub fn keep(&self, held: Held) {
        self.settle();
        self.held.keep(held, Arc::clone(&self.clock));
    }

    /// How many punched streams are being held.
    pub fn held(&self) -> usize {
        self.held.len()
    }

    /// The two seconds-scale rungs: a re-punch at what worked last time,
    /// then the full rendezvous.
    fn climb(&self, rove: &Rove, mine: Vec<IpAddr>) -> Result<TcpStream, String> {
        let punch = punch::Punch::bind(0)?;
        let cached = self.cache.with(&mut |c| c.clone());
        if !cached.is_empty()
            && let Some(tcp) = punch.punch(cached, rove.window)
        {
            return Ok(tcp);
        }
        let mut dht = dht(rove)?;
        let endpoints = call::presence(&mut dht, &rove.pairing)?;
        let seq = self
            .clock
            .unix()
            .max(self.last_seq.load(Ordering::Relaxed) + 1);
        let mine = mine
            .into_iter()
            .map(|ip| SocketAddr::new(ip, punch.port()))
            .collect();
        call::call(&mut dht, &rove.pairing, seq, mine)?;
        self.last_seq.store(seq, Ordering::Relaxed);
        self.cache.with(&mut |c| c.clone_from(&endpoints));
        punch
            .punch(endpoints, rove.window)
            .ok_or_else(|| "the punch landed nothing inside its window".to_owned())
    }

    /// The backoff returns to its floor: something connected or served.
    fn settle(&self) {
        self.backoff.with(&mut Backoff::settle);
    }

    /// **A network change is the addresses moving** (DESIGN §21.3): the
    /// set this box would send from differs from the last dial's. That drops
    /// every held stream — a dead mapping after an address change — and
    /// clears the backoff; the cache stays, because the endpoints that
    /// worked are the first thing to try on the new network, and the
    /// rendezvous is the rung after them. The first dial has nothing to
    /// compare against and changes nothing.
    fn notice_network(&self, mine: &[IpAddr]) {
        let moved = self.seen.with(&mut |seen| {
            let moved = seen.as_deref().is_some_and(|was| was != mine);
            *seen = Some(mine.to_vec());
            moved
        });
        if moved {
            self.held.clear();
            self.settle();
        }
    }
}

/// A DHT client over a fresh UDP socket, once the bootstrap resolves.
fn dht(rove: &Rove) -> Result<Dht, String> {
    let bootstrap: Vec<SocketAddr> = rove
        .bootstrap
        .iter()
        .filter_map(|name| name.to_socket_addrs().ok())
        .flatten()
        .collect();
    if bootstrap.is_empty() {
        return Err("no bootstrap node resolved".to_owned());
    }
    let udp = Udp::bind("0.0.0.0:0".parse().map_err(|e| format!("{e}"))?)
        .map_err(|e| format!("rendezvous: {e}"))?;
    Dht::new(Box::new(udp), bootstrap, rove.config.clone())
}

/// The direct rung: every address the entry resolves to, each given
/// [`DIRECT`], in the sentence today's dial already earns.
fn direct(address: &str) -> Result<TcpStream, String> {
    let mut refused = format!("connect {address}: no address resolved");
    for at in address
        .to_socket_addrs()
        .map_err(|e| format!("connect {address}: {e}"))?
    {
        match TcpStream::connect_timeout(&at, DIRECT) {
            Ok(tcp) => return Ok(tcp),
            Err(e) => refused = format!("connect {address}: {e}"),
        }
    }
    Err(refused)
}

#[cfg(test)]
mod tests;
