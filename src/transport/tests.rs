//! The transport end to end: a real handshake against a real one-shot mTLS
//! server on loopback — both leaves minted by the same openssl recipe the
//! operator's own provisioning uses — then every refusal on the input that
//! earns it.

use super::{Seat, server_name};
use crate::codec::reply::Reply;
use crate::test_support::{material, mint_ca, mint_leaf, scratch, serve_once, serve_versioned};
use serde_json::json;

fn pki() -> std::path::PathBuf {
    let dir = scratch();
    mint_ca(&dir, "ca");
    mint_leaf(&dir, "ca", "server", true);
    mint_leaf(&dir, "ca", "client", false);
    dir
}

#[test]
fn one_ask_crosses_mtls_and_reads_the_stream() {
    let dir = pki();
    let reply = json!({ "ok": true, "kind": "transcript", "rows": [] });
    let (address, served) = serve_once(&dir, "ca", "server", vec![reply.to_string().into_bytes()]);
    let seat = Seat::open(&material(&dir, "ca", "client", &address)).unwrap();
    assert_eq!(seat.address(), address);
    let request = json!({ "op": "workspaces" });
    let stream = seat.ask(&request).unwrap();
    assert_eq!(stream, vec![reply]);
    // The server read the very envelope this seat framed.
    assert_eq!(served.join().unwrap(), request.to_string().into_bytes());
}

#[test]
fn answered_decodes_the_last_frame() {
    let dir = pki();
    let replies = vec![
        json!({ "ok": true, "kind": "transcript", "rows": [] })
            .to_string()
            .into_bytes(),
        json!({ "ok": true, "kind": "conversations", "rows": [] })
            .to_string()
            .into_bytes(),
    ];
    let (address, _served) = serve_once(&dir, "ca", "server", replies);
    let seat = Seat::open(&material(&dir, "ca", "client", &address)).unwrap();
    let reply = seat.answered(&json!({ "op": "conversations" })).unwrap();
    assert_eq!(reply, Reply::Conversations(vec![]));
}

#[test]
fn a_carried_refusal_is_the_engines_sentence() {
    let dir = pki();
    let refusal = json!({ "ok": false, "error": "no such workspace" });
    let (address, _served) =
        serve_once(&dir, "ca", "server", vec![refusal.to_string().into_bytes()]);
    let seat = Seat::open(&material(&dir, "ca", "client", &address)).unwrap();
    let e = seat.answered(&json!({ "op": "workspaces" })).unwrap_err();
    assert_eq!(e, "no such workspace");
}

#[test]
fn an_empty_stream_is_an_engine_that_never_answered() {
    let dir = pki();
    let (address, _served) = serve_once(&dir, "ca", "server", vec![]);
    let seat = Seat::open(&material(&dir, "ca", "client", &address)).unwrap();
    let e = seat.answered(&json!({ "op": "workspaces" })).unwrap_err();
    assert!(e.contains("without answering"), "{e}");
}

#[test]
fn a_frame_that_is_not_json_refuses_on_receive() {
    let dir = pki();
    let (address, _served) = serve_once(&dir, "ca", "server", vec![b"not json".to_vec()]);
    let seat = Seat::open(&material(&dir, "ca", "client", &address)).unwrap();
    let e = seat.ask(&json!({ "op": "workspaces" })).unwrap_err();
    assert!(e.contains("frame is not JSON"), "{e}");
}

#[test]
fn a_dead_address_refuses_on_connect() {
    let dir = pki();
    // Reserved-but-closed: nothing listens on port 1 on loopback.
    let seat = Seat::open(&material(&dir, "ca", "client", "127.0.0.1:1")).unwrap();
    let e = seat.ask(&json!({ "op": "workspaces" })).unwrap_err();
    assert!(e.starts_with("connect 127.0.0.1:1:"), "{e}");
}

#[test]
fn a_server_off_the_operators_ca_never_completes_the_handshake() {
    let dir = pki();
    // A second, rival CA signs the server this seat is told to trust nothing
    // from: the handshake dies inside rustls and no reply byte is ever read.
    mint_ca(&dir, "rival");
    mint_leaf(&dir, "rival", "rogue", true);
    let (address, served) = serve_once(
        &dir,
        "rival",
        "rogue",
        vec![
            json!({ "ok": true, "kind": "transcript", "rows": [] })
                .to_string()
                .into_bytes(),
        ],
    );
    let seat = Seat::open(&material(&dir, "ca", "client", &address)).unwrap();
    let e = seat.ask(&json!({ "op": "workspaces" })).unwrap_err();
    assert!(e.contains("receive:") || e.contains("send:"), "{e}");
    // The server side dies on its own half of the failed handshake.
    assert!(served.join().is_err());
}

/// REMOTE §3's fail-closed mismatch, across a real handshake: an engine of
/// another protocol is refused **before a reply frame is decoded**, so the
/// answer it went on to write is never read as this build's vocabulary. The
/// engine had a perfectly good `transcript` waiting; the seat never sees it.
#[test]
fn a_skewed_engine_is_refused_before_its_answer_is_read() {
    let dir = pki();
    let reply = json!({ "ok": true, "kind": "transcript", "rows": [] });
    let (address, _served) = serve_versioned(
        &dir,
        "ca",
        "server",
        2,
        vec![vec![reply.to_string().into_bytes()]],
    );
    let seat = Seat::open(&material(&dir, "ca", "client", &address)).unwrap();
    let e = seat.ask(&json!({ "op": "workspaces" })).unwrap_err();
    assert_eq!(
        e,
        "wire protocol mismatch: this end speaks version 1, the peer speaks 2. \
         There is no negotiation — upgrade the older component until both \
         speak one version."
    );
}

#[test]
fn the_server_name_is_read_off_the_address() {
    assert!(matches!(
        server_name("127.0.0.1:7737").unwrap(),
        rustls::pki_types::ServerName::IpAddress(_)
    ));
    assert!(matches!(
        server_name("[2001:db8::1]:7737").unwrap(),
        rustls::pki_types::ServerName::IpAddress(_)
    ));
    assert!(matches!(
        server_name("engine.example.com:7737").unwrap(),
        rustls::pki_types::ServerName::DnsName(_)
    ));
    let e = server_name(":7737").unwrap_err();
    assert!(e.contains("not a server name"), "{e}");
}
