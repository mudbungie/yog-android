//! **One ladder per entry, process-wide** (bl-58a0). The seat's asker, the
//! tool host beside it and the attention lane each open their own
//! `transport::Seat` on the same material; before this table each built its
//! own [`Ladder`], so each climbed its own rendezvous into the pairing's one
//! inbox slot. Now a roving entry's ladder is looked up here, and the gate,
//! the held pool and the backoff are the entry's.
//!
//! **The key is what the ladder dials**: the entry's address, the pairing it
//! roves by and the commons it walks. **The ladder is held weakly**: a ladder
//! no seat holds any more goes away with its held streams, as it always did,
//! and the next seat on that entry starts a fresh one. An entry that does not
//! rove shares nothing — its dial is a plain connect with no state behind it.
//!
//! **The punch port and the presence cache are held strongly, for the run**
//! (bl-97ed, bl-c00e; DESIGN §21.9): the call a climb writes names the port
//! it punches from, and a port that is kept is a port the engine's NAT has
//! let in before; the engine's presence is the engine's fact, not this
//! box's, and saves the next climb its walk (the re-call). The [`Port`]
//! outlives every ladder on its entry, and a fresh ladder takes it up where
//! the last one left it.

use super::{Clock, Ladder, Rove};
use crate::rendezvous::punch::Punch;
use crate::state::Slot;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, OnceLock, Weak};

/// The entry's punch port, and the engine's presence as last read.
///
/// **The presence is a re-call's whole input** (yog bl-278f): a climb that
/// holds it writes its call without walking for the presence first. It is
/// kept across a network change — it says where the ENGINE is, and this box
/// moving does not move it — and dropped by a call that expires unanswered,
/// the one evidence that it may be stale, so the next climb reads it again.
#[derive(Default)]
pub(crate) struct Port {
    /// The punch bound for the run, and the network generation it was
    /// bound in: a port is the entry's only in that generation (`network`).
    pub(crate) bound: Option<(Arc<Punch>, u64)>,
    pub(crate) presence: Vec<SocketAddr>,
    /// The last call's `seq`: the next is above it, whichever ladder on
    /// the entry writes it, or the commons refuses it.
    pub(crate) seq: i64,
}

/// An entry's port, shared by every ladder that climbs it.
pub(crate) type Bound = Arc<Slot<Port>>;

type Key = (String, [u8; 32], [u8; 32], Vec<String>);
type Table = Slot<HashMap<Key, (Weak<Ladder>, Bound)>>;

fn table() -> &'static Table {
    static TABLE: OnceLock<Table> = OnceLock::new();
    TABLE.get_or_init(|| Slot::new(HashMap::new()))
}

/// The entry's ladder: the live one if a seat already holds it, else a new
/// one on `rove` and `clock`, climbing from the entry's port.
pub(crate) fn ladder(address: String, rove: Option<Rove>, clock: Arc<dyn Clock>) -> Arc<Ladder> {
    let Some(rove) = rove else {
        return Arc::new(Ladder::new(address, None, clock));
    };
    let key = (
        address.clone(),
        rove.pairing.engine,
        rove.pairing.salt,
        rove.bootstrap.clone(),
    );
    // A ladder costs a few allocations and starts no thread, so it is built
    // under the lock, once, only where the entry has no live one.
    let mut rove = Some(rove);
    table().with(&mut |table| {
        let (live, port) = table
            .entry(key.clone())
            .or_insert_with(|| (Weak::new(), Arc::new(Slot::new(Port::default()))));
        if let Some(live) = live.upgrade() {
            return live;
        }
        let fresh = Arc::new(Ladder::on(
            address.clone(),
            rove.take(),
            Arc::clone(&clock),
            Arc::clone(port),
        ));
        *live = Arc::downgrade(&fresh);
        if let Some(rove) = &fresh.rove {
            rove.network.watch(&fresh);
        }
        fresh
    })
}
