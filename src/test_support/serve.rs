//! The answering servers the suite dials — rustls' server half, which prod
//! here deliberately does not have (the phone never listens; DESIGN §1).

use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::net::TcpListener;
use std::path::Path;
use std::sync::Arc;
use std::thread::JoinHandle;

/// A one-shot mTLS server on loopback: accepts one connection, requires a
/// client certificate chaining to `ca`, reads one frame, answers with each
/// of `replies` then the terminator. Returns the bound address and the join
/// handle carrying the request it read.
pub fn serve_once(
    dir: &Path,
    ca: &str,
    leaf: &str,
    replies: Vec<Vec<u8>>,
) -> (String, JoinHandle<Vec<u8>>) {
    let (address, handle) = serve_many(dir, ca, leaf, vec![replies]);
    let handle = std::thread::spawn(move || {
        let mut requests = handle.join().unwrap();
        requests.pop().unwrap()
    });
    (address, handle)
}

/// The scripted multi-connection server the seat model's tests need: the
/// model dials once per ask, so one test spans a SEQUENCE of connections.
/// Each entry of `scripts` serves one connection — one request read, every
/// scripted reply frame written, then the terminator — and the handle
/// returns every request read, in order.
pub fn serve_many(
    dir: &Path,
    ca: &str,
    leaf: &str,
    scripts: Vec<Vec<Vec<u8>>>,
) -> (String, JoinHandle<Vec<Vec<u8>>>) {
    serve_versioned(dir, ca, leaf, crate::hello::PROTOCOL, scripts)
}

/// [`serve_many`], with the version this engine states made a parameter — the
/// one knob a test needs to stand a skewed engine up (REMOTE §3's fail-closed
/// mismatch). Every other caller gets this build's own `PROTOCOL`, so every
/// existing test also asserts that the seat states its version on **every**
/// connection: the read below refuses a client that did not.
pub fn serve_versioned(
    dir: &Path,
    ca: &str,
    leaf: &str,
    protocol: u32,
    scripts: Vec<Vec<Vec<u8>>>,
) -> (String, JoinHandle<Vec<Vec<u8>>>) {
    let config = config(dir, ca, leaf);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("127.0.0.1:{}", listener.local_addr().unwrap().port());
    let preface = serde_json::json!({ "protocol": protocol })
        .to_string()
        .into_bytes();
    let handle = std::thread::spawn(move || {
        let mut requests = Vec::new();
        for script in scripts {
            let (tcp, _) = listener.accept().unwrap();
            let conn = rustls::ServerConnection::new(Arc::clone(&config)).unwrap();
            let mut tls = rustls::StreamOwned::new(conn, tcp);
            // The engine's half of the preface, stated before this end reads —
            // §3's "both write before either reads", from the other side.
            crate::frame::write_frame(&mut tls, &preface).unwrap();
            let stated = crate::frame::read_frame(&mut tls).unwrap().unwrap();
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&stated).unwrap(),
                serde_json::json!({ "protocol": crate::hello::PROTOCOL }),
                "the seat opened a connection without stating its version"
            );
            requests.push(crate::frame::read_frame(&mut tls).unwrap().unwrap());
            for reply in &script {
                crate::frame::write_frame(&mut tls, reply).unwrap();
            }
            crate::frame::write_end(&mut tls).unwrap();
        }
        requests
    });
    (address, handle)
}

/// The server half's TLS config over the minted files: client certificates
/// required, chaining to `ca`.
fn config(dir: &Path, ca: &str, leaf: &str) -> Arc<rustls::ServerConfig> {
    let mut store = rustls::RootCertStore::empty();
    for anchor in CertificateDer::pem_file_iter(dir.join(format!("{ca}.pem"))).unwrap() {
        store.add(anchor.unwrap()).unwrap();
    }
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let verifier = rustls::server::WebPkiClientVerifier::builder_with_provider(
        Arc::new(store),
        Arc::clone(&provider),
    )
    .build()
    .unwrap();
    let chain: Vec<CertificateDer<'static>> =
        CertificateDer::pem_file_iter(dir.join(format!("{leaf}.pem")))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
    let key = PrivateKeyDer::from_pem_file(dir.join(format!("{leaf}.key"))).unwrap();
    Arc::new(
        rustls::ServerConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_client_cert_verifier(verifier)
            .with_single_cert(chain, key)
            .unwrap(),
    )
}
