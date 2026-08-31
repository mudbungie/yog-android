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
///
/// **The skewed peer is derived, never typed.** A literal here is a literal
/// that reads as the wrong version for exactly one release: this test named
/// the standing version as the peer's the day PROTOCOL moved to 2, and passed
/// only because it happened to be testing the opposite of what it said.
#[test]
fn a_version_skew_names_both_versions_and_the_remedy() {
    let mine = u64::from(PROTOCOL);
    let e = confirm(&mut Cursor::new(peer(mine + 1))).unwrap_err();
    assert_eq!(e, super::mismatch(Some(mine + 1)));
    assert_eq!(
        e,
        format!(
            "wire protocol mismatch: this end speaks version {mine}, the peer \
             speaks {}. There is no negotiation — upgrade the older component \
             until both speak one version.",
            mine + 1
        )
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
