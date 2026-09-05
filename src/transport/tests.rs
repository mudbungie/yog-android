//! The transport end to end: a real handshake against a real one-shot mTLS
//! server on loopback — both leaves minted by the same openssl recipe the
//! operator's own provisioning uses — then every refusal on the input that
//! earns it.

use super::{Seat, Wire, server_name};
use crate::codec::reply::Reply;
use crate::test_support::{
    Turn, material, mint_ca, mint_leaf, scratch, serve_once, serve_turns, serve_versioned,
};
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
    assert_eq!(e, Wire::Refused("no such workspace".to_owned()));
    // The class is the fact a redialling caller acts on (bl-8641): an engine
    // that said no says it again on a fresh connection.
    assert!(!e.transport());
    assert_eq!(String::from(e), "no such workspace");
}

/// **The decoder's two errors are two different facts about who failed**
/// (bl-8bd0). A reply this end cannot READ is unusable — no leg and no redial
/// mends it — while an `ok: false` is the engine's own no, which on the follow
/// read is REMOTE §5.1's one-reader guard and IS worth another dial
/// (`host::serve`'s matrix). They shared one class until the redial matrix
/// needed them apart, and `answered` was throwing the distinction away one
/// line after the decoder made it.
#[test]
fn a_reply_this_end_cannot_read_is_unusable_and_not_the_engine_saying_no() {
    let dir = pki();
    // A kind this codec knows, without the field it is made of.
    let malformed = json!({ "ok": true, "kind": "workspaces" });
    let (address, _served) = serve_once(
        &dir,
        "ca",
        "server",
        vec![malformed.to_string().into_bytes()],
    );
    let seat = Seat::open(&material(&dir, "ca", "client", &address)).unwrap();
    let e = seat.answered(&json!({ "op": "workspaces" })).unwrap_err();
    assert!(matches!(e, Wire::Unusable(_)), "{e:?}");
    assert!(!e.transport());
}

#[test]
fn an_empty_stream_is_an_engine_that_never_answered() {
    let dir = pki();
    let (address, _served) = serve_once(&dir, "ca", "server", vec![]);
    let seat = Seat::open(&material(&dir, "ca", "client", &address)).unwrap();
    let e = seat.answered(&json!({ "op": "workspaces" })).unwrap_err();
    assert!(e.sentence().contains("without answering"), "{e:?}");
    assert!(!e.transport());
}

#[test]
fn a_frame_that_is_not_json_refuses_on_receive() {
    let dir = pki();
    let (address, _served) = serve_once(&dir, "ca", "server", vec![b"not json".to_vec()]);
    let seat = Seat::open(&material(&dir, "ca", "client", &address)).unwrap();
    let e = seat.ask(&json!({ "op": "workspaces" })).unwrap_err();
    assert!(e.sentence().contains("frame is not JSON"), "{e:?}");
    // Bytes that arrived intact and said something unreadable: dialling again
    // cannot mend it, so it is a refusal.
    assert!(!e.transport());
}

/// **The lost reply, at the socket** (yog REMOTE §3, bl-07b1). The gesture is
/// written whole and the engine hangs up where the answer belongs, which is
/// the one shape that tells this end nothing about whether the act ran. It is
/// still the channel — the tool host must dial again — but it is no longer
/// merely a failure, and the two questions are answered by two predicates.
#[test]
fn a_channel_that_dies_after_the_gesture_was_written_leaves_the_act_in_doubt() {
    let dir = pki();
    let (address, served) = serve_turns(&dir, "ca", "server", vec![Turn::Hangup]);
    let seat = Seat::open(&material(&dir, "ca", "client", &address)).unwrap();
    let request = json!({ "op": "nudge", "workspace": "home", "agent": "a1" });
    let e = seat.answered(&request).unwrap_err();
    assert!(matches!(e, Wire::Lost(_)), "{e:?}");
    assert!(
        e.in_doubt(),
        "the act was on the wire when the channel died"
    );
    assert!(e.transport(), "the channel is still what failed, so redial");
    // The engine read the gesture before it hung up: this is the window the
    // contract is about, not one where nothing was said.
    assert_eq!(
        served.join().unwrap(),
        vec![request.to_string().into_bytes()]
    );
}

/// **The other side of the same line.** A socket that would not open carried
/// no byte of the act, so there is nothing to be in doubt about — and telling
/// an operator otherwise every time a phone is out of range would make the
/// word worthless where it means something.
#[test]
fn a_dead_address_refuses_on_connect() {
    let dir = pki();
    // Reserved-but-closed: nothing listens on port 1 on loopback.
    let seat = Seat::open(&material(&dir, "ca", "client", "127.0.0.1:1")).unwrap();
    let e = seat.ask(&json!({ "op": "workspaces" })).unwrap_err();
    assert!(e.sentence().starts_with("connect 127.0.0.1:1:"), "{e:?}");
    // The channel class — the one the tool host climbs a ladder against.
    assert!(e.transport());
    assert!(
        !e.in_doubt(),
        "nothing was written, so nothing may have run"
    );
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
    let said = e.sentence();
    assert!(
        said.contains("receive:") || said.contains("send:"),
        "{said}"
    );
    assert!(e.transport());
    // A handshake that never completed carried no application byte either
    // (bl-07b1): the refusal happens inside the write, before a frame of it
    // could be accepted.
    assert!(!e.in_doubt(), "{said}");
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
    // Derived, not typed: "the version this build does not speak" is one more
    // than the one it does, at every version this build will ever be.
    let theirs = u64::from(crate::hello::PROTOCOL) + 1;
    let (address, _served) = serve_versioned(
        &dir,
        "ca",
        "server",
        u32::try_from(theirs).unwrap(),
        vec![vec![reply.to_string().into_bytes()]],
    );
    let seat = Seat::open(&material(&dir, "ca", "client", &address)).unwrap();
    let e = seat.ask(&json!({ "op": "workspaces" })).unwrap_err();
    let said = e.sentence();
    assert!(
        said.starts_with("wire protocol mismatch: this end speaks version ")
            && said.contains(&format!("the peer speaks {theirs}."))
            && said.ends_with("upgrade the older component until both speak one version."),
        "{said}"
    );
    // A version that cannot be spoken to is a refusal: redialling it forever
    // would hide an operator-actionable fact behind "reconnecting…".
    assert!(!e.transport());
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

mod held;
