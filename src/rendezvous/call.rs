//! The two BEP 44 verbs a rendezvous spends, in the client's order (yog
//! REMOTE §13.2): read the engine's presence, then write the call. Split
//! from the ladder so the DHT traffic is one small file the fake commons can
//! be pointed at, and the rungs stay a policy over it.

use super::Pairing;
use super::item::{Call, Presence};
use crate::dht::Dht;
use std::net::SocketAddr;

/// The engine's punch endpoints, read off its presence item: `get` under
/// its rendezvous key and the presence salt, opened under the seal key. An
/// engine that has published nothing, or something this pairing cannot
/// open, is one this device cannot find — named as such rather than as an
/// empty list, because the ladder's next rung is the same either way and
/// the sentence is what an operator reads.
pub(crate) fn presence(dht: &mut Dht, pairing: &Pairing) -> Result<Vec<SocketAddr>, String> {
    let item = dht
        .get(pairing.engine, pairing.presence_salt())?
        .ok_or_else(|| "the engine has published no presence".to_owned())?;
    let presence = Presence::open(&pairing.seal_key(), &item.value)
        .ok_or_else(|| "the engine's presence will not open under this pairing".to_owned())?;
    if presence.endpoints.is_empty() {
        return Err("the engine's presence names no endpoint".to_owned());
    }
    Ok(presence.endpoints)
}

/// Write the call — *punch me at these endpoints* — under the derived inbox
/// keypair and salt, sealed, at `seq`. A fresh random nonce per call, so the
/// engine (which remembers the last nonce it punched) punches exactly once
/// for it; the nonce is handed back for the suite to read off the commons.
pub(crate) fn call(
    dht: &mut Dht,
    pairing: &Pairing,
    seq: i64,
    endpoints: Vec<SocketAddr>,
) -> Result<u64, String> {
    let mut nonce = [0u8; 8];
    crate::dht::random(&mut nonce)?;
    let nonce = u64::from_be_bytes(nonce);
    let sealed = Call { nonce, endpoints }.seal(&pairing.seal_key())?;
    let item = pairing
        .inbox_keypair()?
        .sign(pairing.inbox_salt(), seq, sealed)?;
    dht.put(item)?;
    Ok(nonce)
}
