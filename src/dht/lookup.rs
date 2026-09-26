//! The iterative lookup (BEP 5), the one walk every read and write of the
//! commons makes: ask the closest nodes you know, learn closer ones from
//! their answers, repeat until the K closest have all been asked. Rounds are
//! synchronous — α queries out, then one bounded wait for whatever answers —
//! and a node that stays silent is simply never asked again. Bounded twice
//! over against a hostile commons: a round ends at its deadline whatever is
//! still pending, and the walk ends at `max_queries` however many "closer"
//! nodes the answers keep inventing.

use super::Dht;
use super::bencode::{Dict, bytes, entry};
use super::krpc::{self, Message, Node, NodeId};
use std::collections::{BTreeMap, BTreeSet};
use std::net::SocketAddr;
use std::time::Instant;

/// What a walk found: every reply, closest first, and every error a node
/// answered instead of a reply.
#[derive(Debug, Default)]
pub(crate) struct Outcome {
    pub(crate) replies: Vec<(Node, Dict)>,
    pub(crate) errors: Vec<String>,
}

/// The transactions one round is waiting on, by transaction id.
pub(crate) type Pending = BTreeMap<Vec<u8>, SocketAddr>;

impl Dht {
    /// Walk toward `target` asking `q` of every node on the way — except the
    /// bootstrap, which is asked `find_node` whatever `q` is. The mainline's
    /// routers answer `find_node` and never BEP 44's `get` (yog bl-f6e1,
    /// REMOTE §13.7 ruling 3), so a walk that asked them `q` was dark in one
    /// round; the bootstrap is a door into the keyspace, and every node it
    /// opens onto is asked `q`. One walk, not a lookup and then a second one.
    ///
    /// The bootstrap is a door and never a result (yog bl-9408): its answers
    /// seed the pool, but it is not a node near the target and nothing it
    /// says is in the [`Outcome`]. So a walk whose learned nodes were all
    /// silent — measured one walk in five from yog's deployed engine box, the
    /// one answering router naming a single node eight times — is the same
    /// `Err` as a silent bootstrap: a dark commons. Not an empty `Ok`, because
    /// an empty `Ok` already means *nodes near the target answered and held
    /// nothing*, and the ladder would read the engine as absent rather than
    /// the commons as dark. `Err` is that, or the socket itself failing; a
    /// walk whose learned nodes drew only errors is an `Ok` with none.
    ///
    /// A node is one `(id, address)`: a repeated entry is learned once, and
    /// one id at two addresses is two nodes to ask.
    pub(crate) fn search(&mut self, target: NodeId, q: &str) -> Result<Outcome, String> {
        if self.bootstrap.is_empty() {
            return Err("no bootstrap node to ask".into());
        }
        let mut asked: BTreeSet<SocketAddr> = BTreeSet::new();
        let mut pool: BTreeMap<([u8; 20], SocketAddr), Node> = BTreeMap::new();
        let mut out = Outcome::default();
        let mut sent = 0usize;
        let args = Dict::from([entry("target", bytes(&target.0))]);
        let mut picks = self.bootstrap.clone();
        let mut seeding = true;
        loop {
            let verb = if seeding { "find_node" } else { q };
            let mut pending = Pending::new();
            for addr in picks {
                asked.insert(addr);
                sent += 1;
                self.ask(&mut pending, addr, verb, args.clone());
            }
            self.collect(&mut pending, &mut |addr, message| match message {
                Message::Reply { r, .. } => {
                    let Some(id) = r
                        .get(b"id".as_slice())
                        .and_then(|v| v.as_bytes())
                        .and_then(NodeId::parse)
                    else {
                        return;
                    };
                    let node = Node { id, addr };
                    let this = (!seeding).then_some(node);
                    for near in krpc::nodes_of(&r).into_iter().chain(this) {
                        pool.insert((near.id.distance(&target), near.addr), near);
                    }
                    if !seeding {
                        out.replies.push((node, r));
                    }
                }
                Message::Error { code, message, .. } if !seeding => {
                    out.errors.push(format!("{addr}: {code} {message}"));
                }
                Message::Error { .. } => {}
            })?;
            seeding = false;
            picks = pool
                .values()
                .take(self.config.k)
                .filter(|n| !asked.contains(&n.addr))
                .take(self.config.alpha)
                .map(|n| n.addr)
                .collect();
            if picks.is_empty() || sent >= self.config.max_queries {
                break;
            }
        }
        if out.replies.is_empty() && out.errors.is_empty() {
            return Err(format!("no DHT node answered {q} for {target}"));
        }
        out.replies.sort_by_key(|(n, _)| n.id.distance(&target));
        Ok(out)
    }

    /// Send one query and, if the send itself went, remember the transaction.
    pub(crate) fn ask(&mut self, pending: &mut Pending, addr: SocketAddr, q: &str, args: Dict) {
        let tid = self.next_tid();
        let datagram = krpc::query(&tid, &self.id, q, args);
        if self.transport.send(addr, &datagram).is_ok() {
            pending.insert(tid, addr);
        }
    }

    /// One round: read datagrams until every pending transaction has answered
    /// or the round's deadline passes. Only an answer to a transaction this
    /// round sent reaches `on`; everything else on the socket is noise.
    pub(crate) fn collect(
        &mut self,
        pending: &mut Pending,
        on: &mut dyn FnMut(SocketAddr, Message),
    ) -> Result<(), String> {
        let deadline = Instant::now() + self.config.round;
        while !pending.is_empty() {
            let wait = deadline.saturating_duration_since(Instant::now());
            if wait.is_zero() {
                break;
            }
            let Some((_, bytes)) = self
                .transport
                .recv(wait)
                .map_err(|e| format!("DHT socket: {e}"))?
            else {
                break;
            };
            let Some(message) = krpc::parse(&bytes) else {
                continue;
            };
            let (Message::Reply { tid, .. } | Message::Error { tid, .. }) = &message;
            if let Some(addr) = pending.remove(tid) {
                on(addr, message);
            }
        }
        Ok(())
    }
}
