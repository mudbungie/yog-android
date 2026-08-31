//! The preface, both directions, over in-memory framing — the transport's own
//! tests carry it across a real handshake.

use super::{PROTOCOL, confirm, state};
use crate::frame;
use serde_json::json;
use std::io::Cursor;

/// One framed body as the bytes a peer would have written.
fn framed(body: &[u8]) -> Vec<u8> {
    let mut buf = Vec::new();
    frame::write_frame(&mut buf, body).unwrap();
    buf
}

/// A peer stating `protocol`, framed.
fn peer(protocol: u64) -> Vec<u8> {
    framed(json!({ "protocol": protocol }).to_string().as_bytes())
}

#[test]
fn this_build_states_one_framed_object_and_nothing_else() {
    let mut buf = Vec::new();
    state(&mut buf).unwrap();
    assert_eq!(buf, peer(u64::from(PROTOCOL)));
}

#[test]
fn an_engine_of_this_version_is_confirmed() {
    assert_eq!(confirm(&mut Cursor::new(peer(u64::from(PROTOCOL)))), Ok(()));
}

/// The sentence is the upgrade prompt: it names this end's version, the
/// peer's, and the remedy — so an operator reads what to do, not a code.
#[test]
fn a_version_skew_names_both_versions_and_the_remedy() {
    let e = confirm(&mut Cursor::new(peer(2))).unwrap_err();
    assert_eq!(
        e,
        "wire protocol mismatch: this end speaks version 1, the peer speaks 2. \
         There is no negotiation — upgrade the older component until both \
         speak one version."
    );
}

/// REMOTE §3: *"A peer that states no version is refused exactly as a peer of
/// the wrong one."* Five ways to say nothing, one sentence — and it is the
/// same sentence, with the peer's half spelled `no version`.
#[test]
fn every_way_of_stating_nothing_is_the_one_refusal() {
    let unstated = "the peer speaks no version.";
    let ways: Vec<Vec<u8>> = vec![
        // A peer that hung up before the preface.
        Vec::new(),
        // A header promising more bytes than ever arrive.
        vec![0, 0, 0, 8, b'{'],
        // The terminator where a preface belongs.
        framed(b""),
        // Bytes that are not JSON at all.
        framed(b"not json"),
        // JSON that is not an object.
        framed(b"7"),
        // An object with no such key.
        framed(br#"{"ok":true}"#),
        // The key present but not an unsigned integer — an unversioned build
        // that grew a `protocol` field of another shape is still unversioned.
        framed(br#"{"protocol":"one"}"#),
        // The pre-version era, exactly: a gesture envelope where a preface
        // belongs. Diagnosed, never special-cased.
        framed(br#"{"op":"workspaces"}"#),
    ];
    for bytes in ways {
        let e = confirm(&mut Cursor::new(bytes)).unwrap_err();
        assert!(e.contains(unstated), "{e}");
    }
}
