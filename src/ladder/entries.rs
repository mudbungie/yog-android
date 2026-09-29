//! **One ladder per entry, process-wide** (bl-58a0). The seat's asker, the
//! tool host beside it and the attention lane each open their own
//! `transport::Seat` on the same material; before this table each built its
//! own [`Ladder`], so each climbed its own rendezvous into the pairing's one
//! inbox slot. Now a roving entry's ladder is looked up here, and the gate,
//! the held pool, the endpoint cache and the backoff are the entry's.
//!
//! **The key is what the ladder dials**: the entry's address, the pairing it
//! roves by and the commons it walks. **Held weakly**: a ladder no seat holds
//! any more goes away with its held streams, as it always did, and the next
//! seat on that entry starts a fresh one. An entry that does not rove shares
//! nothing — its dial is a plain connect with no state behind it.

use super::{Clock, Ladder, Rove};
use crate::state::Slot;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock, Weak};

type Key = (String, [u8; 32], [u8; 32], Vec<String>);

fn table() -> &'static Slot<HashMap<Key, Weak<Ladder>>> {
    static TABLE: OnceLock<Slot<HashMap<Key, Weak<Ladder>>>> = OnceLock::new();
    TABLE.get_or_init(|| Slot::new(HashMap::new()))
}

/// The entry's ladder: the live one if a seat already holds it, else a new
/// one on `rove` and `clock`.
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
    // Built before the lock, and dropped unused if a seat already holds
    // the entry's: a ladder costs a few allocations and starts no thread.
    let fresh = Arc::new(Ladder::new(address, Some(rove), clock));
    table().with(&mut |table| {
        table.retain(|_, ladder| ladder.strong_count() > 0);
        if let Some(live) = table.get(&key).and_then(Weak::upgrade) {
            return live;
        }
        table.insert(key.clone(), Arc::downgrade(&fresh));
        Arc::clone(&fresh)
    })
}
