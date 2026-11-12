//! **The address list** (yog REMOTE §15.3; DESIGN §21): this device's
//! endpoints are dialled in order, and the one that worked is tried first for
//! the rest of the run.

use super::{Seat, pki};
use crate::test_support::{material, mint_ca, mint_leaf, scratch, serve_many, serve_once};
use serde_json::{Value, json};
use std::net::TcpListener;
use std::sync::mpsc;

fn answered() -> Value {
    json!({ "ok": true, "kind": "transcript", "rows": [] })
}

fn frames() -> Vec<Vec<u8>> {
    vec![answered().to_string().into_bytes()]
}

fn ask(seat: &Seat) -> Vec<Value> {
    seat.ask(&json!({ "op": "workspaces" })).unwrap()
}

/// A loopback listener that takes each connection, says so, and hangs up
/// without a byte: something that answers and is not the server.
fn stranger() -> (String, mpsc::Receiver<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let at = listener.local_addr().unwrap().to_string();
    let (took, taken) = mpsc::channel();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            if took.send(()).is_err() {
                return;
            }
            drop(stream);
        }
    });
    (at, taken)
}

/// **An endpoint nothing listens at is passed over**, and the next one in the
/// list answers. Nothing listens on port 1 on loopback.
#[test]
fn a_dial_falls_through_a_dead_endpoint_to_the_next_in_the_list() {
    let dir = pki();
    let (address, served) = serve_once(&dir, "ca", "server", frames());
    let listed = format!("127.0.0.1:1\n{address}\n");
    let seat = Seat::open(&material(&dir, "ca", "client", &listed)).unwrap();
    assert_eq!(seat.address(), format!("127.0.0.1:1, {address}"));
    assert_eq!(ask(&seat), vec![answered()]);
    assert!(!served.join().unwrap().is_empty(), "the request arrived");
}

/// **What answers and is not the server is passed over too, and the endpoint
/// that worked is tried first from then on** — in RAM, for the run. The
/// stranger at the head of the list is dialled once and never again.
#[test]
fn the_endpoint_that_worked_is_dialled_first_for_the_rest_of_the_run() {
    let dir = pki();
    let (address, _served) = serve_many(&dir, "ca", "server", vec![frames(), frames()]);
    let (at, taken) = stranger();
    let seat = Seat::open(&material(&dir, "ca", "client", &format!("{at}\n{address}"))).unwrap();
    assert_eq!(ask(&seat), vec![answered()]);
    assert_eq!(taken.try_recv(), Ok(()), "the head of the list was tried");
    assert_eq!(ask(&seat), vec![answered()]);
    assert!(taken.try_recv().is_err(), "and not a second time");
}

/// **A server under another CA is not the server**: its handshake does not
/// verify, and the list goes on.
#[test]
fn an_endpoint_that_cannot_present_the_operators_ca_is_passed_over() {
    let dir = pki();
    let elsewhere = scratch();
    mint_ca(&elsewhere, "ca");
    mint_leaf(&elsewhere, "ca", "server", true);
    let (theirs, _other) = serve_once(&elsewhere, "ca", "server", frames());
    let (address, _served) = serve_once(&dir, "ca", "server", frames());
    let listed = format!("{theirs}\n{address}");
    let seat = Seat::open(&material(&dir, "ca", "client", &listed)).unwrap();
    assert_eq!(ask(&seat), vec![answered()]);
}

/// **A list that reached nothing says why of every line in it**, and nothing
/// crossed.
#[test]
fn a_list_that_reaches_nothing_names_every_endpoint() {
    let dir = pki();
    let seat = Seat::open(&material(&dir, "ca", "client", "127.0.0.1:1\n127.0.0.1:2")).unwrap();
    let e = seat.ask(&json!({ "op": "workspaces" })).unwrap_err();
    assert!(e.transport() && !e.in_doubt(), "{e:?}");
    let said = e.sentence();
    assert!(said.starts_with("connect 127.0.0.1:1: "), "{said}");
    assert!(said.contains("; connect 127.0.0.1:2: "), "{said}");
}

/// A line that is not an endpoint refuses when the seat opens, naming it.
#[test]
fn a_line_that_names_no_server_refuses_the_whole_list() {
    let dir = pki();
    let listed = "server.example:9000\nnot a name:9000";
    let Err(refusal) = Seat::open(&material(&dir, "ca", "client", listed)) else {
        panic!("opened");
    };
    assert!(refusal.contains("not a name:9000"), "{refusal}");
}

/// A name that resolves to nothing is the endpoint's own refusal.
#[test]
fn a_name_that_does_not_resolve_is_named_in_the_refusal() {
    let dir = pki();
    let seat = Seat::open(&material(&dir, "ca", "client", "nowhere.invalid:7737")).unwrap();
    let e = seat.ask(&json!({ "op": "workspaces" })).unwrap_err();
    assert!(
        e.sentence().starts_with("connect nowhere.invalid:7737: "),
        "{e:?}"
    );
}
