//! The mTLS wrapper's client half — the mirror of the server's `wire/tls.rs`
//! (yog REMOTE §1.3, §4): a rustls configuration built from [`Material`].
//!
//! **Both ends authenticate with certificates, and that is the entire
//! authentication story.** This seat requires the engine's leaf to chain to
//! the operator CA and presents its own; there is no password, token or
//! account anywhere in the channel, so there is nothing in it to phish,
//! rotate or leak, and an unauthenticated peer gets a TLS refusal, never a
//! reply.
//!
//! **The provider is named, never defaulted.** The builder's process-global
//! default is a panic path when none is installed or two are (AGENTS.md rule
//! 4); naming `ring` outright removes the global read and the panic with it —
//! and `ring` is the ruling's own condition (deny.toml bans `aws-lc-sys`).

use crate::material::Material;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::{ClientConfig, RootCertStore};
use std::path::Path;
use std::sync::Arc;

/// The seat's end: verify the server against the operator CA, and present
/// the client leaf — the certificate that *is* this seat's identity.
pub(crate) fn client_config(m: &Material) -> Result<Arc<ClientConfig>, String> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let (chain, key) = identity(&m.chain, &m.key)?;
    let config = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|e| format!("tls versions: {e}"))?
        .with_root_certificates(anchors(&m.anchors)?)
        .with_client_auth_cert(chain, key)
        .map_err(|e| format!("{}: client identity: {e}", m.chain.display()))?;
    Ok(Arc::new(config))
}

/// The operator CA as a trust anchor store. Every certificate in the file is
/// an anchor: an operator who put two in meant two.
fn anchors(path: &Path) -> Result<RootCertStore, String> {
    let mut store = RootCertStore::empty();
    for anchor in
        CertificateDer::pem_file_iter(path).map_err(|e| format!("{}: {e}", path.display()))?
    {
        let anchor = anchor.map_err(|e| format!("{}: {e}", path.display()))?;
        store
            .add(anchor)
            .map_err(|e| format!("{}: {e}", path.display()))?;
    }
    if store.is_empty() {
        return Err(format!("{}: no certificate in it", path.display()));
    }
    Ok(store)
}

/// This seat's chain and key, read from PEM.
fn identity(
    chain: &Path,
    key: &Path,
) -> Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>), String> {
    let certs: Vec<CertificateDer<'static>> = CertificateDer::pem_file_iter(chain)
        .map_err(|e| format!("{}: {e}", chain.display()))?
        .collect::<Result<_, _>>()
        .map_err(|e| format!("{}: {e}", chain.display()))?;
    if certs.is_empty() {
        return Err(format!("{}: no certificate in it", chain.display()));
    }
    let private =
        PrivateKeyDer::from_pem_file(key).map_err(|e| format!("{}: {e}", key.display()))?;
    Ok((certs, private))
}

#[cfg(test)]
mod tests;
