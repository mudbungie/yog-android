//! **What the door answered, filed as this device's material** (yog REMOTE
//! §15.1 step 2; DESIGN §5).
//!
//! The answer is one JSON object — `workspace`, `name`, `grade`, `address`
//! (the list a client dials, in order), `ca` and `cert` — and with the key
//! this device made it is the whole of [`crate::material`]'s four files.
//!
//! **Filing replaces what was there, and only once every file is written.**
//! One device holds one leaf (DESIGN §9), so a registration does not join
//! this device's identities — it becomes the identity. Each file is written
//! whole under a second name beside its own and then renamed over it, so a
//! write that fails leaves the old identity standing and no file is ever
//! half of one certificate and half of another.

use std::path::Path;

use serde_json::Value;

use crate::material::{ADDRESS, ANCHORS, CHAIN, KEY};

/// What the server issued this device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claimed {
    /// The workspace the offer seats this device in, once approved.
    pub workspace: String,
    /// The name the server knows this device by.
    pub name: String,
    pub grade: String,
    /// The endpoints to dial, in order. Never empty.
    pub addresses: Vec<String>,
    /// The operator CA, PEM.
    pub ca: String,
    /// The leaf signed for this device's key, PEM.
    pub cert: String,
}

/// Read the door's answer, or name the first field it does not carry.
pub fn read(body: &[u8]) -> Result<Claimed, String> {
    let answer: Value = serde_json::from_slice(body).unwrap_or(Value::Null);
    let text = |key: &str| {
        answer
            .get(key)
            .and_then(Value::as_str)
            .filter(|said| !said.is_empty())
            .map(str::to_owned)
            .ok_or_else(|| missing(key))
    };
    let addresses: Vec<String> = answer
        .get(ADDRESS)
        .and_then(Value::as_array)
        .map(|listed| listed.iter().filter_map(Value::as_str).collect::<Vec<_>>())
        .map(|listed| crate::material::list(&listed.join("\n")))
        .filter(|listed| !listed.is_empty())
        .ok_or_else(|| missing(ADDRESS))?;
    Ok(Claimed {
        workspace: text("workspace")?,
        name: text("name")?,
        grade: text("grade")?,
        addresses,
        ca: text("ca")?,
        cert: text("cert")?,
    })
}

fn missing(key: &str) -> String {
    format!("the server's answer carried no `{key}`; nothing was filed")
}

/// **File the material** in `dir`, with `key` as its private key.
pub fn file(dir: &Path, claimed: &Claimed, key: &str) -> Result<(), String> {
    let address = claimed.addresses.join("\n");
    let files = [
        (ANCHORS, claimed.ca.as_str()),
        (CHAIN, claimed.cert.as_str()),
        (KEY, key),
        (ADDRESS, address.as_str()),
    ];
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for (leaf, body) in files {
        laid(&staged(dir, leaf), body)?;
    }
    for (leaf, _) in files {
        let path = dir.join(leaf);
        std::fs::rename(staged(dir, leaf), &path)
            .map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(())
}

/// Where a file waits, whole, until all four are written.
fn staged(dir: &Path, leaf: &str) -> std::path::PathBuf {
    dir.join(format!("{leaf}.new"))
}

/// One file, readable by this app alone — the mode rides the `open`, so the
/// private key is never wider even for an instant.
fn laid(path: &Path, body: &str) -> Result<(), String> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    options
        .open(path)
        .and_then(|mut file| {
            std::io::Write::write_all(&mut file, format!("{}\n", body.trim_end()).as_bytes())
        })
        .map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(test)]
mod tests;
