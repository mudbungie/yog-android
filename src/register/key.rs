//! **The key this device makes for itself, and the word both ends show** (yog REMOTE
//! §15.1 steps 2 and 3).
//!
//! A registering device generates a P-256 pair and sends only the public half.
//! The private half is written beside the leaf and goes nowhere else — which
//! is the whole of what the offer changed about enrollment: nothing that is
//! the device ever travels.
//!
//! **The confirmation word** is six characters of SHA-256 over the public key
//! (DER `SubjectPublicKeyInfo`) followed by the operator CA (DER, the first
//! certificate of what the server handed back): the digest's first thirty
//! bits, five at a time, most significant first, each an index into
//! [`ALPHABET`]. The server computes the same from what it signed and what it
//! holds, so an impostor between the two cannot make them agree.

use ring::rand::SystemRandom;
use ring::signature::{ECDSA_P256_SHA256_ASN1_SIGNING, EcdsaKeyPair, KeyPair};
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};

/// The alphabet a code and a word are spelled in: capitals and digits without
/// `I`, `O`, `0` and `1`, because a person reads them.
pub const ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

/// Everything a P-256 `SubjectPublicKeyInfo` says ahead of the point itself:
/// the algorithm (`id-ecPublicKey`, `prime256v1`) and the bit string's head.
/// With the 65-byte uncompressed point after it, the 91 bytes REMOTE §15.1
/// names.
const SPKI_HEAD: [u8; 26] = [
    0x30, 0x59, 0x30, 0x13, 0x06, 0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01, 0x06, 0x08, 0x2a,
    0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07, 0x03, 0x42, 0x00,
];

/// What a device whose random source or key generation failed is told.
const NO_KEY: &str = "this device could not generate a P-256 key";

/// A key pair made on this device.
pub struct Made {
    /// The private key, PKCS#8 DER. Written beside the leaf and nowhere else.
    pub pkcs8: Vec<u8>,
    /// The public key, DER `SubjectPublicKeyInfo`.
    pub spki: Vec<u8>,
}

/// Make one, from the system's random source.
pub fn make() -> Result<Made, String> {
    let rng = SystemRandom::new();
    let alg = &ECDSA_P256_SHA256_ASN1_SIGNING;
    let pkcs8 = EcdsaKeyPair::generate_pkcs8(alg, &rng).map_err(|_| NO_KEY.to_owned())?;
    let spki = spki(pkcs8.as_ref()).ok_or_else(|| NO_KEY.to_owned())?;
    Ok(Made {
        pkcs8: pkcs8.as_ref().to_vec(),
        spki,
    })
}

/// The public half of a P-256 private key (PKCS#8 DER), as the
/// `SubjectPublicKeyInfo` the word is read from. `None` for any other key —
/// a leaf carried here by hand under another algorithm has no word.
pub fn spki(pkcs8: &[u8]) -> Option<Vec<u8>> {
    let alg = &ECDSA_P256_SHA256_ASN1_SIGNING;
    let pair = EcdsaKeyPair::from_pkcs8(alg, pkcs8, &SystemRandom::new()).ok()?;
    let mut spki = SPKI_HEAD.to_vec();
    spki.extend_from_slice(pair.public_key().as_ref());
    Some(spki)
}

/// [`spki`] for a private key as its PEM file holds it.
pub fn spki_of(key_pem: &str) -> Option<Vec<u8>> {
    let key = PrivateKeyDer::from_pem_slice(key_pem.as_bytes()).ok()?;
    spki(key.secret_der())
}

/// DER as the PEM block `label` names, 64 columns to the line.
pub fn pem(label: &str, der: &[u8]) -> String {
    let body = base64(der);
    let lines: Vec<_> = body
        .as_bytes()
        .chunks(64)
        .map(String::from_utf8_lossy)
        .collect();
    format!(
        "-----BEGIN {label}-----\n{}\n-----END {label}-----\n",
        lines.join("\n")
    )
}

/// Standard base64, padded. Hand-rolled because the dependency set is closed
/// and the decode half is already `rustls-pki-types`'.
fn base64(bytes: &[u8]) -> String {
    const SYMBOLS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let symbol =
        |index: u32| char::from(SYMBOLS.get((index & 63) as usize).copied().unwrap_or(b'='));
    let mut text = String::new();
    for chunk in bytes.chunks(3) {
        let word = chunk
            .iter()
            .chain([0, 0].iter())
            .take(3)
            .fold(0u32, |word, byte| (word << 8) | u32::from(*byte));
        text.push(symbol(word >> 18));
        text.push(symbol(word >> 12));
        text.push(if chunk.len() > 1 {
            symbol(word >> 6)
        } else {
            '='
        });
        text.push(if chunk.len() > 2 { symbol(word) } else { '=' });
    }
    text
}

/// **The confirmation word** for a public key and the CA the server handed
/// back, or the refusal a `ca` with no certificate in it earns.
pub fn word(spki: &[u8], ca_pem: &str) -> Result<String, String> {
    let ca = CertificateDer::pem_slice_iter(ca_pem.as_bytes())
        .next()
        .and_then(Result::ok)
        .ok_or("the server's answer carried no CA certificate")?;
    let mut hashed = ring::digest::Context::new(&ring::digest::SHA256);
    hashed.update(spki);
    hashed.update(ca.as_ref());
    let digest = hashed.finish();
    let bits = digest
        .as_ref()
        .iter()
        .take(4)
        .fold(0u32, |bits, byte| (bits << 8) | u32::from(*byte));
    Ok((0..6)
        .map(|n| {
            let index = (bits >> (27 - 5 * n)) & 31;
            char::from(ALPHABET.get(index as usize).copied().unwrap_or(b'?'))
        })
        .collect())
}

#[cfg(test)]
pub(crate) mod tests;
