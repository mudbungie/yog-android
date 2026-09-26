//! **The DHT rendezvous client** (yog REMOTE §13.2) — a pure client of the
//! mainline DHT (BEP 5): outbound-UDP iterative lookups and BEP 44 signed
//! mutable get/put, and never a node. No listener, no routing answers, no
//! storage offered — nothing for the commons to dial back, which is REMOTE
//! §13's ruling 2 applied to the rendezvous itself, and DESIGN §1's *"the
//! phone opens no listening socket"* one layer down.
//!
//! **Reimplemented per component, as REMOTE §13.7 ruling 2 recommends**, from
//! the engine's own `src/dht` (yog bl-df31): a DHT client is substrate, not
//! protocol, and at under nine hundred lines a copy costs less than a fifth
//! repository's release machinery. Nothing in it is yog-shaped and nothing
//! in it is phone-shaped either — no world, no boundary, no wire type crosses
//! its interface, and its one dependency is `ring`, which rustls already
//! links (DESIGN §21). What this end uses it for is the client half alone:
//! `get` reads the engine's presence, `put` writes the call.
//!
//! The shape the punched wire consumes is this module root: a caller hands
//! [`Dht::new`] a [`Transport`] (the std UDP socket [`Udp`], or a stand-in),
//! the bootstrap addresses and a [`Config`], then asks [`Dht::lookup`] for
//! the nodes nearest an id, [`Dht::get`] for the newest item under a key and
//! salt, or [`Dht::put`] to store one a [`Keypair`] signed. Everything the
//! commons answers is untrusted until it verifies: an item without a good
//! signature under the asked-for key is not an item.
//!
//! Four files under this root, one concern each: `bencode` the encoding,
//! `krpc` the datagram shapes, `mutable` the signed item, `transport` the
//! socket seam; `lookup` is the walk and `items` the two BEP 44 verbs over
//! it. Synchronous throughout — `std::net` with socket timeouts, no tokio
//! (AGENTS.md rule 8) — and every duration is a [`Config`] field a test can
//! shorten, so the fake DHT the suite runs on loopback UDP answers in
//! milliseconds where the commons answers in seconds.

pub mod bencode;
mod items;
pub mod krpc;
mod lookup;
pub mod mutable;
pub mod transport;

pub use krpc::{Node, NodeId};
pub use mutable::{Keypair, Mutable, target_of};
pub use transport::{Transport, Udp};

use ring::rand::SecureRandom;
use std::net::SocketAddr;
use std::time::Duration;

/// The walk's parameters — stated so a test can shrink them and a caller
/// can widen them. α, K and the cap are BEP 5's; the one-second round is
/// the engine's, measured against the live mainline (yog bl-5d8d).
#[derive(Clone, Debug)]
pub struct Config {
    /// Queries in flight per round.
    pub alpha: usize,
    /// How many closest nodes a walk converges on and a `put` writes to.
    pub k: usize,
    /// How long one round waits for its answers.
    pub round: Duration,
    /// The most queries one walk may send, however the commons answers.
    pub max_queries: usize,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            alpha: 3,
            k: 8,
            round: Duration::from_secs(1),
            max_queries: 64,
        }
    }
}

/// One client: a transport, the nodes it starts from, and the id it queries
/// as. The id is random per client — a client is not a node and nobody
/// routes by it, so nothing is lost by minting a fresh one every run.
pub struct Dht {
    transport: Box<dyn Transport>,
    bootstrap: Vec<SocketAddr>,
    config: Config,
    id: NodeId,
    tid: u16,
}

impl Dht {
    /// A client over `transport`, starting every walk from `bootstrap`.
    pub fn new(
        transport: Box<dyn Transport>,
        bootstrap: Vec<SocketAddr>,
        config: Config,
    ) -> Result<Dht, String> {
        let mut id = [0u8; 20];
        random(&mut id)?;
        Ok(Dht {
            transport,
            bootstrap,
            config,
            id: NodeId(id),
            tid: 0,
        })
    }

    /// The id this client queries as.
    pub fn id(&self) -> NodeId {
        self.id
    }

    /// The nodes nearest `target`, closest first — BEP 5's `find_node` walk.
    pub fn lookup(&mut self, target: NodeId) -> Result<Vec<Node>, String> {
        let out = self.search(target, "find_node")?;
        Ok(out
            .replies
            .into_iter()
            .map(|(n, _)| n)
            .take(self.config.k)
            .collect())
    }

    /// The next transaction id: two bytes, wrapping, never reused within a
    /// walk (a walk sends at most `max_queries`, far under 65 536).
    pub(crate) fn next_tid(&mut self) -> Vec<u8> {
        self.tid = self.tid.wrapping_add(1);
        self.tid.to_be_bytes().to_vec()
    }
}

/// Fill `buf` from the system's randomness.
pub(crate) fn random(buf: &mut [u8]) -> Result<(), String> {
    ring::rand::SystemRandom::new()
        .fill(buf)
        .map_err(|_| "the system offered no randomness".to_string())
}

#[cfg(test)]
pub(crate) mod tests;
