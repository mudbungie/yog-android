//! The seat's key material, and what its absence means — the client half of
//! the server's `wire/material.rs` (yog REMOTE §1.4, §8).
//!
//! **This app never mints a certificate.** Provisioning is an act performed
//! through existing trust (DESIGN §5: adb, remote exec, QR), so this module
//! only ever *reads*, and the three answers it can give are the whole trust
//! bootstrap:
//!
//! - **Nothing provisioned** — `Ok(None)`. The wire is simply off: the seat
//!   has nowhere to dial, and the shell says so instead of dialling.
//! - **Partly provisioned** — `Err(missing)`, naming every absent file at
//!   once. Half a trust store is a misconfiguration, and one that silently
//!   degrades is the failure mode this design exists to exclude.
//! - **Provisioned** — `Ok(Some(Material))`: the anchors, the client leaf and
//!   key, and the one address.
//!
//! The directory is the caller's fact (the Android shell hands its app-files
//! dir; a test hands a scratch dir); the file names inside it are this
//! module's, and they are the delivery channels' write contract.

use std::path::{Path, PathBuf};

/// The operator CA this seat verifies the engine against.
pub const ANCHORS: &str = "ca.pem";
/// This seat's certificate chain — the leaf that *is* its identity.
pub const CHAIN: &str = "client.pem";
/// This seat's private key.
pub const KEY: &str = "client.key";
/// The file naming the `host:port` this seat dials.
pub const ADDRESS: &str = "address";

/// **Every file a provisioned device holds**, in the order a screen should
/// read them out. One definition: [`read_dir`] checks exactly this list and
/// the enrollment screen names exactly this list, so a fifth file cannot be
/// required by the reader and unnamed by the screen that asks for it.
pub const WANTED: [&str; 4] = [ANCHORS, CHAIN, KEY, ADDRESS];

/// One seat's provisioned material.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Material {
    /// The operator CA, PEM.
    pub anchors: PathBuf,
    /// This seat's certificate chain, PEM.
    pub chain: PathBuf,
    /// This seat's private key, PEM.
    pub key: PathBuf,
    /// `host:port`, as provisioned.
    pub address: String,
}

/// Read the seat's material out of `dir`. See the module doc for the three
/// answers; the `Err` names every missing file at once, because a remedy that
/// reveals one gap per run is a remedy run four times.
pub fn read_dir(dir: &Path) -> Result<Option<Material>, String> {
    let wanted = WANTED;
    let missing: Vec<&str> = wanted
        .iter()
        .filter(|f| !dir.join(f).is_file())
        .copied()
        .collect();
    if missing.len() == wanted.len() {
        return Ok(None);
    }
    if !missing.is_empty() {
        return Err(format!(
            "half-provisioned at {}: missing {}",
            dir.display(),
            missing.join(", ")
        ));
    }
    // A file that will not read yields no address, and no address is the same
    // refusal an empty one earns: one branch, because "unreadable" and
    // "empty" are one fact about what this seat can be told to dial.
    let address = std::fs::read_to_string(dir.join(ADDRESS))
        .unwrap_or_default()
        .trim()
        .to_owned();
    if address.is_empty() {
        return Err(format!(
            "{} names no address; it must hold one host:port",
            dir.join(ADDRESS).display()
        ));
    }
    Ok(Some(Material {
        anchors: dir.join(ANCHORS),
        chain: dir.join(CHAIN),
        key: dir.join(KEY),
        address,
    }))
}

#[cfg(test)]
mod tests;
