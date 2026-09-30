//! **The two rungs that cost seconds** (DESIGN §21.3, §21.9): a re-call
//! from the cached presence, then the full rendezvous — split from `ladder`
//! on the seam the backoff already draws, since only these two sit behind
//! it.

use super::{Ladder, Rove, say};
use crate::dht::{Dht, Udp};
use crate::rendezvous::call;
use crate::rendezvous::punch::Punch;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;

impl Ladder {
    /// **One walk or two, then a call and a punch** (yog bl-278f). With the
    /// engine's presence cached the climb is a **re-call**: it writes a
    /// fresh call at the cached endpoints without reading the presence
    /// first. Without it, the full rendezvous reads the presence, caches it,
    /// and calls. Either way the call names the port the climb punches from,
    /// because the engine punches a call once, toward the port it named, and
    /// its NAT lets back nothing else — a punch with no call behind it is
    /// one-sided, which is why the old re-punch rung is gone. A call that
    /// expires unanswered drops the cached presence: the next climb reads it.
    ///
    /// The port is the entry's (`bound`), except for a dial `beside` a line
    /// still out (`gate::Dialling::beside`): that line holds the entry's
    /// port toward the engine's, and TCP carries one connection per pair of
    /// ends, so a beside call names a fresh port of its own, which is
    /// forgotten after its window (bl-c21d).
    ///
    /// **A climb stops at a rung boundary once the process is not awake**
    /// (bl-c00e; `awake`): before the walk and before the call, so nothing is
    /// written in the background. A punch whose call is already written runs
    /// its window out — the engine is punching toward it.
    pub(super) fn climb(&self, rove: &Rove, beside: bool) -> Result<TcpStream, String> {
        let punch = if beside {
            Arc::new(Punch::bind(0)?)
        } else {
            self.bound(rove)?
        };
        self.boundary(rove)?;
        let mut dht = dht(rove).inspect_err(|e| self.voice.say(&say::no_commons(e)))?;
        let cached = self.port.with(&mut |port| port.presence.clone());
        let endpoints = if cached.is_empty() {
            let (at, endpoints) = call::presence(&mut dht, &rove.pairing)
                .inspect_err(|e| self.voice.say(&say::not_found(e)))?;
            self.voice.say(&say::found(at, &endpoints));
            self.port
                .with(&mut |port| port.presence.clone_from(&endpoints));
            endpoints
        } else {
            self.voice.say(&say::recall(&cached));
            cached
        };
        self.boundary(rove)?;
        let last = self.port.with(&mut |port| port.seq);
        let seq = self.clock.unix().max(last + 1);
        // The addresses are read now, as the call names them — not at the
        // dial's start, a walk of the commons ago (bl-792e).
        let (_, mine) = self.notice(rove);
        let called = call::call(&mut dht, &rove.pairing, seq, mine, punch.port())
            .inspect_err(|_| self.voice.say(&say::not_called()))?;
        self.voice.say(&say::called(&called, seq));
        self.port.with(&mut |port| port.seq = seq);
        self.voice.say(&say::punching(&endpoints, rove.window));
        self.landed(punch.punch(endpoints, rove.window), rove.window)
            .ok_or_else(|| {
                self.port.with(&mut |port| port.presence.clear());
                "the punch landed nothing inside its window".to_owned()
            })
    }

    /// A rung boundary: the climb goes on only while the process is awake.
    fn boundary(&self, rove: &Rove) -> Result<(), String> {
        if rove.awake.awake() {
            return Ok(());
        }
        self.voice.say(&say::stopped());
        Err("the climb stopped: the app left the foreground".to_owned())
    }

    /// The entry's punch port, bound on the first climb and kept for the
    /// run (bl-97ed); a port of another network generation is not the
    /// entry's any more, and is let go (`network`).
    fn bound(&self, rove: &Rove) -> Result<Arc<Punch>, String> {
        let generation = rove.network.generation();
        self.port.with(&mut |port| {
            if let Some((punch, _)) = port.bound.as_ref().filter(|(_, g)| *g == generation) {
                return Ok(Arc::clone(punch));
            }
            let punch = Arc::new(Punch::bind(0)?);
            port.bound = Some((Arc::clone(&punch), generation));
            Ok(punch)
        })
    }

    /// Say how a punch ended — the peer's family, or the window running out.
    pub(super) fn landed(&self, tcp: Option<TcpStream>, window: Duration) -> Option<TcpStream> {
        self.voice.say(&match &tcp {
            Some(tcp) => say::landed(tcp.peer_addr().ok().map(|a| a.ip())),
            None => say::expired(window),
        });
        tcp
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
