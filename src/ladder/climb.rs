//! **The two rungs that cost seconds** (DESIGN §21.3): a re-punch at the
//! RAM-cached endpoints, then the full rendezvous — split from `ladder` on
//! the seam the backoff already draws, since only these two sit behind it.

use super::entries::Port;
use super::{Ladder, Rove, say};
use crate::dht::{Dht, Udp};
use crate::rendezvous::call;
use crate::rendezvous::punch::Punch;
use std::net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

impl Ladder {
    /// The two seconds-scale rungs: a re-punch at what worked last time,
    /// then the full rendezvous, both from the entry's one port (`bound`):
    /// the engine's NAT answers a re-punch only from the port the call it
    /// punched named. `repunch` is false for a dial beside a line still out
    /// (`gate::Dialling::beside`): the cached endpoints are the ones that
    /// line was punched at, the engine punches a call's nonce once (yog
    /// REMOTE §13.4, *already punched — no punch*), and so a second line
    /// takes a fresh call or nothing — from a fresh port, because the line
    /// still out holds the entry's port toward the engine's, and TCP carries
    /// one connection per pair of ends. **That port is the beside call's
    /// alone, and is forgotten** (bl-c21d): the entry's port stays the one
    /// whose call landed, whose mapping the engine's NAT holds — replacing
    /// it with the beside port was measured live leaving no later re-punch a
    /// mapping to ride.
    pub(super) fn climb(
        &self,
        rove: &Rove,
        mine: Vec<IpAddr>,
        repunch: bool,
    ) -> Result<TcpStream, String> {
        let (punch, cached) = if repunch {
            self.bound()?
        } else {
            (Arc::new(Punch::bind(0)?), Vec::new())
        };
        if !cached.is_empty() {
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
        // The call named this punch's port, so these endpoints are worth
        // re-punching from it — the pair is stored as one (`entries::Port`).
        // A beside call's port is not the entry's, and is never stored.
        if repunch {
            self.port.with(&mut |port| {
                *port = Some(Port {
                    punch: Arc::clone(&punch),
                    cached: endpoints.clone(),
                    armed: true,
                });
            });
        }
        self.voice.say(&say::punching(&endpoints, rove.window));
        self.landed(punch.punch(endpoints, rove.window), rove.window)
            .ok_or_else(|| "the punch landed nothing inside its window".to_owned())
    }

    /// The entry's punch port, bound on the first climb and kept for the
    /// run (bl-97ed), with the endpoints to re-punch — the last call's, if
    /// the re-punch is armed, and it is disarmed by being handed out
    /// (`entries::Port`). A port bound here has no call behind it yet, so it
    /// caches nothing.
    fn bound(&self) -> Result<(Arc<Punch>, Vec<SocketAddr>), String> {
        self.port.with(&mut |port| {
            if let Some(port) = port {
                let cached = if port.armed {
                    port.cached.clone()
                } else {
                    Vec::new()
                };
                port.armed = false;
                return Ok((Arc::clone(&port.punch), cached));
            }
            let punch = Arc::new(Punch::bind(0)?);
            *port = Some(Port {
                punch: Arc::clone(&punch),
                cached: Vec::new(),
                armed: false,
            });
            Ok((punch, Vec::new()))
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
