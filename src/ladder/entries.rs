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
//! **The punch port is held strongly, for the run** (bl-97ed, DESIGN §21.9):
//! the engine punches a call once, toward the port that call named, and its
//! NAT then holds a mapping toward exactly that port — so a re-punch is
//! answerable only from it. The [`Port`] outlives every ladder on its entry,
//! and a fresh ladder takes it up where the last one left it.

use super::{Clock, Ladder, Rove};
use crate::rendezvous::punch::Punch;
use crate::state::Slot;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, OnceLock, Weak};

/// The entry's punch port and the engine endpoints the last call from it
/// found — one value, because the endpoints are worth re-punching only from
/// the port the call named. A network change drops both (`notice_network`).
///
/// **A re-punch is spent by trying it** (`armed`). A re-punch can land a
/// TCP stream the engine never serves — its listener completes the
/// handshake whenever the SYN gets in, but it accepts only inside a new
/// call's window (DESIGN §21.9) — and a cache that outlived that would
/// re-punch into the same dead end on every redial and never write a call.
/// So a re-punch disarms it, a line served and kept re-arms it, and a call
/// written arms it afresh.
pub(crate) struct Port {
    pub(crate) punch: Arc<Punch>,
    pub(crate) cached: Vec<SocketAddr>,
    pub(crate) armed: bool,
}

/// An entry's port, shared by every ladder that climbs it.
pub(crate) type Bound = Arc<Slot<Option<Port>>>;

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
            .or_insert_with(|| (Weak::new(), Arc::new(Slot::new(None))));
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
        fresh
    })
}
