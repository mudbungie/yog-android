//! **A server that answers by what it is asked, not by when** (bl-1782). A
//! pushing tool host speaks on several connections at once — one read held
//! open, and a post per job from whichever thread finished — so the order its
//! connections arrive in is nobody's to script. Each turn here is keyed on
//! the request's `op` and spent by the first connection asking that op.

use super::serve::{Turn, config, play};
use serde_json::Value;
use std::net::TcpListener;
use std::path::Path;
use std::sync::{Arc, mpsc};
use std::thread::JoinHandle;

/// Serve `turns`, each spent by the first connection whose request names its
/// op, stating `edition` in the preface. Every request is told to the
/// returned receiver the moment it is read — the one way a test can wait on
/// a connection it did not open — and the handle returns them all, in arrival
/// order, once every turn is spent. A request no turn is left for is hung up
/// on.
pub fn serve_by_op(
    dir: &Path,
    ca: &str,
    leaf: &str,
    edition: u32,
    turns: Vec<(&'static str, Turn)>,
) -> (String, mpsc::Receiver<Value>, JoinHandle<Vec<Value>>) {
    let config = config(dir, ca, leaf);
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = format!("127.0.0.1:{}", listener.local_addr().unwrap().port());
    let preface = serde_json::json!({ "protocol": crate::hello::PROTOCOL, "edition": edition })
        .to_string()
        .into_bytes();
    let (told, heard) = mpsc::channel();
    let handle = std::thread::spawn(move || {
        let mut turns = turns;
        let mut requests = Vec::new();
        while !turns.is_empty() {
            let (tcp, _) = listener.accept().unwrap();
            let conn = rustls::ServerConnection::new(Arc::clone(&config)).unwrap();
            let mut tls = rustls::StreamOwned::new(conn, tcp);
            crate::frame::write_frame(&mut tls, &preface).unwrap();
            crate::frame::read_frame(&mut tls).unwrap().unwrap();
            let request = crate::frame::read_frame(&mut tls).unwrap().unwrap();
            let asked: Value = serde_json::from_slice(&request).unwrap();
            let _ = told.send(asked.clone());
            let turn = turns
                .iter()
                .position(|(op, _)| asked["op"] == *op)
                .map_or(Turn::Hangup, |at| turns.remove(at).1);
            requests.push(asked);
            std::thread::spawn(move || play(tls, turn));
        }
        requests
    });
    (address, heard, handle)
}
