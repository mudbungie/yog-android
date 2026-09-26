//! The two BEP 44 verbs a rendezvous spends, in the client's order (yog
//! REMOTE §13.2): read the engine's presence, then write the call. Split
//! from the ladder so the DHT traffic is one small file the fake commons can
//! be pointed at, and the rungs stay a policy over it.

use super::Pairing;
use super::item::{Call, Presence};
use crate::dht::Dht;
use std::net::{IpAddr, SocketAddr};

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
/// keypair and salt, sealed, at `seq`. The endpoints are `mine` (the
/// route-local addresses) and then every address the last walk's nodes
/// agreed they saw this device at ([`Dht::observed`]) that is not one of
/// them, all at the punch `port` (yog REMOTE §13.2, the engine's own
/// presence rule, yog bl-efae). On cellular the observed address is the only
/// one the engine can reach (DESIGN §21). Only the ADDRESS is taken: the
/// observed port is the DHT socket's UDP mapping, not the punch port's TCP
/// one, so port preservation is trusted — a carrier that rewrites it is the
/// case this does not reach (yog REMOTE §13.8).
///
/// A fresh random nonce per call, so the engine (which remembers the last
/// nonce it punched) punches exactly once for it; the nonce is handed back
/// for the suite to read off the commons.
pub(crate) fn call(
    dht: &mut Dht,
    pairing: &Pairing,
    seq: i64,
    mine: Vec<IpAddr>,
    port: u16,
) -> Result<u64, String> {
    let mut ips = mine;
    for ip in dht.observed().iter().map(SocketAddr::ip) {
        if !ips.contains(&ip) {
            ips.push(ip);
        }
    }
    let endpoints = ips
        .into_iter()
        .map(|ip| SocketAddr::new(ip, port))
        .collect();
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

#[cfg(test)]
mod tests;
