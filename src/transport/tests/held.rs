//! **The held read** (DESIGN §14.1): the door a lane parks on, and the two
//! ways a read on it ends before the engine's terminator — the seat hanging
//! up from another thread, and the reader answering no.

use super::{Seat, Wire, pki};
use crate::test_support::{Turn, material, serve_once, serve_turns};
use serde_json::json;

/// **A held read is ended from another thread by hanging up** (DESIGN
/// §14.1): the reader is parked on the socket and the shutdown is what wakes
/// it — as a lost stream, which is what a hold that ended under it is. A
/// second hang-up on a socket already gone is nothing.
#[test]
fn a_held_read_is_ended_by_a_hangup_from_another_thread() {
    let dir = pki();
    let frame = json!({ "ok": true, "kind": "attention", "rows": [] });
    let (address, _served) = serve_turns(
        &dir,
        "ca",
        "server",
        vec![Turn::Hold(vec![frame.to_string().into_bytes()])],
    );
    let seat = Seat::open(&material(&dir, "ca", "client", &address)).unwrap();
    let (open, hangup) = seat.hold(&json!({ "op": "attention" })).unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    let reader = std::thread::spawn(move || {
        open.each(&mut |got| {
            tx.send(got).unwrap();
            true
        })
    });
    assert_eq!(rx.recv().unwrap(), frame);
    hangup.hang_up();
    let ended = reader.join().unwrap().unwrap_err();
    assert!(matches!(ended, Wire::Lost(_)), "{ended:?}");
    hangup.hang_up();
}

/// The reader's own `false` ends a read cleanly, with the frames after it
/// unread: the connection is dropped, which is how the engine learns its
/// answer has no reader.
#[test]
fn a_reader_that_answers_false_ends_the_read_where_it_stands() {
    let dir = pki();
    let first = json!({ "ok": true, "kind": "transcript", "rows": [] });
    let second = json!({ "ok": true, "kind": "conversations", "rows": [] });
    let (address, _served) = serve_once(
        &dir,
        "ca",
        "server",
        vec![
            first.to_string().into_bytes(),
            second.to_string().into_bytes(),
        ],
    );
    let seat = Seat::open(&material(&dir, "ca", "client", &address)).unwrap();
    let mut seen = Vec::new();
    let (open, _hangup) = seat.hold(&json!({ "op": "transcript" })).unwrap();
    open.each(&mut |got| {
        seen.push(got);
        false
    })
    .unwrap();
    assert_eq!(seen, vec![first]);
}
