//! **The two rungs that cost seconds** (DESIGN §21.3): a re-punch at the
//! RAM-cached endpoints, then the full rendezvous — split from `ladder` on
//! the seam the backoff already draws, since only these two sit behind it.

use super::{Ladder, Rove, say};
use crate::dht::{Dht, Udp};
use crate::rendezvous::{call, punch};
use std::net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::atomic::Ordering;
use std::time::Duration;

impl Ladder {
    /// The two seconds-scale rungs: a re-punch at what worked last time,
    /// then the full rendezvous. `repunch` is false for a dial beside a line
    /// still out (`gate::Dialling::beside`): the cached endpoints are the
    /// ones that line was punched at, the engine punches a call's nonce
    /// once (yog REMOTE §13.4, *already punched — no punch*), and so a
    /// second line takes a fresh call or nothing.
    pub(super) fn climb(
        &self,
        rove: &Rove,
        mine: Vec<IpAddr>,
        repunch: bool,
    ) -> Result<TcpStream, String> {
        let punch = punch::Punch::bind(0)?;
        let cached = self.cache.with(&mut |c| c.clone());
        if repunch && !cached.is_empty() {
            self.voice.say(&say::repunch(&cached, rove.window));
            if let Some(tcp) = self.landed(punch.punch(cached, rove.window), rove.window) {
                return Ok(tcp);
            }
        }
        let mut dht = dht(rove).inspect_err(|e| self.voice.say(&say::no_commons(e)))?;
        let (at, endpoints) = call::presence(&mut dht, &rove.pairing)
            .inspect_err(|e| self.voice.say(&say::not_found(e)))?;
        self.voice.say(&say::found(at, &endpoints));
        let seq = self
            .clock
            .unix()
            .max(self.last_seq.load(Ordering::Relaxed) + 1);
        let called = call::call(&mut dht, &rove.pairing, seq, mine, punch.port())
            .inspect_err(|_| self.voice.say(&say::not_called()))?;
        self.voice.say(&say::called(&called, seq));
        self.last_seq.store(seq, Ordering::Relaxed);
        self.cache.with(&mut |c| c.clone_from(&endpoints));
        self.voice.say(&say::punching(&endpoints, rove.window));
        self.landed(punch.punch(endpoints, rove.window), rove.window)
            .ok_or_else(|| "the punch landed nothing inside its window".to_owned())
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
