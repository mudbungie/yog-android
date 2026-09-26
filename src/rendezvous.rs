//! **The rendezvous material** (yog REMOTE §13.2, engine end bl-4263; the
//! phone's half of yog bl-0247): two files beside `ca.pem`, and everything
//! both ends derive from them.
//!
//! **Minted out of channel, like every other fact in `material`** (REMOTE
//! §1.4). The engine mints an ed25519 rendezvous seed and a random pairing
//! salt; what crosses into this device's entry is the **public** half of the
//! key and the salt, hex, exactly as `ca.pem` crosses. This module only ever
//! reads, and the three answers it gives are `material`'s three one rung
//! over: nothing (`Ok(None)` — an entry with an address and no rendezvous
//! material is today's entry, byte for byte, REMOTE §13.4), half (`Err`,
//! naming the remedy), or the [`Pairing`].
//!
//! **One salt, four derivations, all HKDF-SHA256** under the salt `yog
//! rendezvous`, one info label each, 32 bytes out — so the commons stores
//! nothing that names the pairing. `presence salt` and `inbox salt` are the
//! DHT salts the two items are filed under; `seal key` is the
//! ChaCha20-Poly1305 key both are sealed with; `inbox key` is a second
//! ed25519 **seed**, so the inbox is signed under a keypair derived from the
//! salt and this device mints no keypair of its own for it. That is why the
//! entry holds two facts and not three: a phone keypair would be a stored
//! fact the engine never reads, and the inbox's is computed. Presence lives
//! at `(engine public key, presence salt)`; the inbox at `(inbox public key,
//! inbox salt)`. The labels are the engine's, byte for byte.

use crate::dht::Keypair;
use ring::hkdf::{HKDF_SHA256, Salt};
use std::path::Path;

pub mod item;
pub mod punch;

pub(crate) mod call;

/// The engine's rendezvous PUBLIC key, hex — the key its presence is signed
/// under. The engine's own file of the same stem holds the seed; a client
/// holds the half it can verify with.
pub const KEY: &str = "rendezvous.pub";
/// The pairing salt, hex — the secret this device and the engine share,
/// under the engine's own file name.
pub const SALT: &str = "pairing.salt";

/// **The two roving files**, in the order a screen should read them out —
/// the optional pair beside [`crate::material::WANTED`], both or neither.
pub const ROVING: [&str; 2] = [KEY, SALT];

/// What the two files hold, decoded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pairing {
    /// The engine's ed25519 public key.
    pub engine: [u8; 32],
    /// The pairing salt.
    pub salt: [u8; 32],
}

impl Pairing {
    /// The keypair the inbox is signed with, which both ends derive.
    pub(crate) fn inbox_keypair(&self) -> Result<Keypair, String> {
        Keypair::from_seed(self.derive(b"inbox key"))
    }

    /// The DHT salt the presence item is filed under.
    pub(crate) fn presence_salt(&self) -> Vec<u8> {
        self.derive(b"presence salt").to_vec()
    }

    /// The DHT salt the inbox item is filed under.
    pub(crate) fn inbox_salt(&self) -> Vec<u8> {
        self.derive(b"inbox salt").to_vec()
    }

    /// The AEAD key both items are sealed under.
    pub(crate) fn seal_key(&self) -> [u8; 32] {
        self.derive(b"seal key")
    }

    /// HKDF-SHA256 over the pairing salt, one label per derived fact.
    fn derive(&self, label: &[u8]) -> [u8; 32] {
        let mut out = [0u8; 32];
        // HKDF-SHA256 expands to exactly 32 bytes for a 32-byte buffer, so
        // neither step can refuse; the fallback is unreachable and named.
        if let Ok(okm) = Salt::new(HKDF_SHA256, b"yog rendezvous")
            .extract(&self.salt)
            .expand(&[label], HKDF_SHA256)
        {
            let _ = okm.fill(&mut out);
        }
        out
    }
}

/// Read the pairing out of `dir`: `Ok(None)` is nothing provisioned (no
/// roving for this entry), `Err` is half of it — the same misconfiguration a
/// half-provisioned trust store is, named the same way.
pub fn read_dir(dir: &Path) -> Result<Option<Pairing>, String> {
    let (engine, salt) = (hex_file(&dir.join(KEY)), hex_file(&dir.join(SALT)));
    match (engine, salt) {
        (None, None) => Ok(None),
        (Some(engine), Some(salt)) => Ok(Some(Pairing { engine, salt })),
        _ => Err(format!(
            "half-provisioned at {}: {KEY} and {SALT} are 32 bytes of hex each and \
             land together — re-enroll this device from an engine that roves",
            dir.display()
        )),
    }
}

/// Exactly 32 bytes of hex in `path`, or nothing — a file that will not read
/// or does not decode is no material, the same as an absent one.
fn hex_file(path: &Path) -> Option<[u8; 32]> {
    let text = std::fs::read_to_string(path).ok()?;
    let bytes = unhex(text.trim())?;
    <[u8; 32]>::try_from(bytes).ok()
}

/// Lowercase hex — the mint's spelling, kept for the suite to write fixtures in.
#[cfg(test)]
pub(crate) fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::new(), |mut out, b| {
        let _ = write!(out, "{b:02x}");
        out
    })
}

/// The inverse of [`hex`]; `None` on any byte that is not one.
pub(crate) fn unhex(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    text.as_bytes()
        .chunks(2)
        .map(|pair| {
            let pair = std::str::from_utf8(pair).ok()?;
            u8::from_str_radix(pair, 16).ok()
        })
        .collect()
}

#[cfg(test)]
mod tests;
