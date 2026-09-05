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

/// **What one scripted connection does** (bl-8641). A reply list can say
/// "answer nothing and terminate cleanly", which is an engine that answered
/// badly; it cannot say "the socket went away mid-answer", which is what a
/// phone that changed networks meets and the one class the tool host redials.
pub enum Turn {
    /// Write these reply frames, then the terminator.
    Answer(Vec<Vec<u8>>),
    /// Read the request and hang up — a FIN where an answer belongs.
    Hangup,
    /// Write these frames and then HOLD the connection — no terminator —
    /// until the seat hangs up. What a follow-class read looks like from the
    /// engine's side, and what a lane in a test parks on.
    Hold(Vec<Vec<u8>>),
    /// Hold the connection and write each frame the test FEEDS, as it feeds
    /// it — the engine's *"a frame whenever the answer changes"*, with the
    /// test as the world. Dropping the sender ends the hold cleanly: the
    /// terminator is written, which is the bound expiring.
    Feed(std::sync::mpsc::Receiver<Vec<u8>>),
}

/// [`serve_many`], with each connection's turn spelled — the entry point for
/// a test that needs a channel to BREAK rather than to refuse.
pub fn serve_turns(
    dir: &Path,
    ca: &str,
    leaf: &str,
    turns: Vec<Turn>,
) -> (String, JoinHandle<Vec<Vec<u8>>>) {
    scripted(dir, ca, leaf, crate::hello::PROTOCOL, turns, Vec::new())
}

/// The scripted engine with **the attention lane served aside** (DESIGN
/// §14.1): a connection whose request is `attention` is answered from `lane`
/// — one turn per dial, in order, each on its own thread — and is not a turn
/// of `turns`, nor a request the handle reports. The lane stands for the
/// seat's whole life, so scripting it positionally would put one line in
/// every script and make every request index a moving target; and it is
/// re-dialled at the pass after it ends, whose timing against a test's
/// gestures is nobody's to script. A lane past its script is held quiet.
pub fn serve_lanes(
    dir: &Path,
    ca: &str,
    leaf: &str,
    turns: Vec<Turn>,
    lane: Vec<Turn>,
) -> (String, JoinHandle<Vec<Vec<u8>>>) {
    scripted(dir, ca, leaf, crate::hello::PROTOCOL, turns, lane)
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
    scripted(
        dir,
        ca,
        leaf,
        protocol,
        scripts.into_iter().map(Turn::Answer).collect(),
        Vec::new(),
    )
}

/// The attention lane's quiet answer: an empty queue, held.
fn quiet_lane() -> Turn {
    let empty = serde_json::json!({ "ok": true, "kind": "attention", "rows": [] })
        .to_string()
        .into_bytes();
    Turn::Hold(vec![empty])
}

/// One connection's turn, played out. A held turn parks on the socket until
/// the seat hangs up, so its caller runs it on a thread of its own.
fn play(mut tls: rustls::StreamOwned<rustls::ServerConnection, std::net::TcpStream>, turn: Turn) {
    let script = match turn {
        Turn::Hangup => return,
        Turn::Answer(script) => script,
        Turn::Hold(script) => {
            for reply in &script {
                crate::frame::write_frame(&mut tls, reply).unwrap();
            }
            // The seat never writes after its request, so this read parks
            // until it hangs up — a FIN, which is the error that ends it.
            while crate::frame::read_frame(&mut tls).is_ok() {}
            return;
        }
        Turn::Feed(frames) => {
            while let Ok(reply) = frames.recv() {
                if crate::frame::write_frame(&mut tls, reply.as_slice()).is_err() {
                    return;
                }
            }
            let _ = crate::frame::write_end(&mut tls);
            return;
        }
    };
    for reply in &script {
        crate::frame::write_frame(&mut tls, reply).unwrap();
    }
    crate::frame::write_end(&mut tls).unwrap();
}

/// The one implementation the three entry points above share.
fn scripted(
    dir: &Path,
    ca: &str,
    leaf: &str,
    protocol: u32,
    turns: Vec<Turn>,
    lane: Vec<Turn>,
) -> (String, JoinHandle<Vec<Vec<u8>>>) {
    let config = config(dir, ca, leaf);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("127.0.0.1:{}", listener.local_addr().unwrap().port());
    let preface = serde_json::json!({ "protocol": protocol })
        .to_string()
        .into_bytes();
    let handle = std::thread::spawn(move || {
        let mut requests = Vec::new();
        let mut turns = turns.into_iter();
        let mut lane = lane.into_iter();
        // The loop serves every scripted turn, and every scripted lane dial:
        // a lane scripted past the last turn is still owed its connection.
        while turns.len() > 0 || lane.len() > 0 {
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
            let request = crate::frame::read_frame(&mut tls).unwrap().unwrap();
            let asked: serde_json::Value = serde_json::from_slice(&request).unwrap();
            if asked["op"] == "attention" {
                let turn = lane.next().unwrap_or_else(quiet_lane);
                std::thread::spawn(move || play(tls, turn));
                continue;
            }
            requests.push(request);
            // A request past the script's last turn is hung up on, which is
            // the refusal a test reads as a `receive:` error.
            match turns.next().unwrap_or(Turn::Hangup) {
                held @ (Turn::Hold(_) | Turn::Feed(_)) => {
                    std::thread::spawn(move || play(tls, held));
                }
                turn => play(tls, turn),
            }
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
