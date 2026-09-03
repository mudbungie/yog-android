//! The model end to end, over the real transport against the scripted
//! multi-connection server: every gesture triggers exactly one refresh pass,
//! so with a very long cadence each test's connection count is a script, not
//! a race. The requests the server read back are asserted too — the model's
//! side of the wire is pinned, not assumed.
//!
//! This file is the scaffolding every case shares — the PKI, the scripted
//! server, the settle loop and the canned replies. The cases themselves are
//! split on the seam the model has: [`reads`] is the standing set the seat
//! re-asks, and the two acts it posts have a file each — [`deposit`] and
//! [`start`].

pub(super) use super::Model;
use super::Snapshot;
pub(super) use crate::test_support::{Turn, material, serve_many, serve_turns};
use crate::test_support::{mint_ca, mint_leaf, scratch};
use crate::transport::Seat;
use serde_json::{Value, json};
use std::time::Duration;

/// Long enough that no unprompted refresh fires inside a test.
pub(super) const REST: Duration = Duration::from_hours(1);

pub(super) fn pki() -> std::path::PathBuf {
    let dir = scratch();
    mint_ca(&dir, "ca");
    mint_leaf(&dir, "ca", "server", true);
    mint_leaf(&dir, "ca", "client", false);
    dir
}

pub(super) fn model_against(
    scripts: Vec<Vec<Vec<u8>>>,
) -> (Model, std::thread::JoinHandle<Vec<Vec<u8>>>) {
    let dir = pki();
    let (address, served) = serve_many(&dir, "ca", "server", scripts);
    let seat = Seat::open(&material(&dir, "ca", "client", &address)).unwrap();
    (Model::start(seat, REST, cache_in(&dir)), served)
}

/// [`model_against`], with each connection's turn spelled — the entry point
/// for a test that needs a channel to BREAK under a gesture rather than to
/// refuse it (bl-07b1: a lost reply is neither a refusal nor a failure).
pub(super) fn model_turns(turns: Vec<Turn>) -> (Model, std::thread::JoinHandle<Vec<Vec<u8>>>) {
    let dir = pki();
    let (address, served) = serve_turns(&dir, "ca", "server", turns);
    let seat = Seat::open(&material(&dir, "ca", "client", &address)).unwrap();
    (Model::start(seat, REST, cache_in(&dir)), served)
}

/// A throwaway cache path beside a test's PKI. Every model writes one
/// (bl-de96), so every test gets its own rather than sharing a boot state —
/// and the tests that are ABOUT the cache name the path themselves.
pub(super) fn cache_in(dir: &std::path::Path) -> std::path::PathBuf {
    dir.join("cache").join("seat.json")
}

/// Poll until a snapshot satisfies `pass` — the worker publishes on its own
/// thread, so the test waits the way a frame would, just faster.
///
/// The predicate is a trait OBJECT rather than a generic: every caller lives
/// in a sibling file, and a monomorphized helper's lines are attributed to
/// the instantiation rather than to the definition, which left this function
/// reading as uncovered while every one of its callers exercised it.
pub(super) fn settle(model: &mut Model, pass: &dyn Fn(&Snapshot) -> bool) -> Snapshot {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        let snap = model.snapshot();
        if pass(&snap) {
            return snap;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "no matching snapshot; last: {snap:?}"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

pub(super) fn ws_reply() -> Vec<u8> {
    ws_named("home")
}

/// A roster of one named workspace. Named rather than fixed so a test can
/// tell one pass's answer from another's — which is how a grace test proves
/// WHICH pass it is reading (bl-3202).
pub(super) fn ws_named(workspace: &str) -> Vec<u8> {
    json!({ "ok": true, "kind": "workspaces",
            "rows": [{ "workspace": workspace, "kind": "named", "attention": 0,
                       "agents": 1, "running": false }] })
    .to_string()
    .into_bytes()
}

/// A conversation row that is WRITING: the `flight` the engine puts on the
/// row is the whole gate the live lane reads (bl-4822).
pub(super) fn conv_flying() -> Vec<u8> {
    json!({ "ok": true, "kind": "conversations",
            "rows": [{ "root_id": "a1", "display": "d", "display_only": false,
                       "state": "in-flight", "uncertain": false, "preview": "",
                       "age_secs": 0, "last_active_unix": 1_700_000_042_i64, "flight": "inference", "attention": 0,
                       "members": 1, "direct": 0, "stoppable": true,
                       "stop_children": false, "depth": 0, "tone": "live" }] })
    .to_string()
    .into_bytes()
}

pub(super) fn conv_reply() -> Vec<u8> {
    json!({ "ok": true, "kind": "conversations",
            "rows": [{ "root_id": "a1", "display": "d", "display_only": false,
                       "state": "quiescent", "uncertain": false, "preview": "",
                       "age_secs": 0, "last_active_unix": 1_700_000_042_i64, "attention": 0, "members": 1, "direct": 0,
                       "stoppable": false, "stop_children": false, "depth": 0,
                       "tone": "plain" }] })
    .to_string()
    .into_bytes()
}

pub(super) fn tr_reply() -> Vec<u8> {
    json!({ "ok": true, "kind": "transcript",
            "rows": [{ "name": "001", "raw": "", "kind": "raw" }] })
    .to_string()
    .into_bytes()
}

pub(super) fn prepared() -> Vec<u8> {
    json!({ "ok": true, "kind": "prepared",
            "prepared": { "workspace": "home", "binding": null, "lineage": null,
                          "goal": "look", "origin": "conversation" } })
    .to_string()
    .into_bytes()
}

pub(super) fn outcome(ok: bool, stderr: &str) -> Vec<u8> {
    json!({ "ok": ok, "kind": "outcome", "exit": i32::from(!ok),
            "stdout": "", "stderr": stderr })
    .to_string()
    .into_bytes()
}

/// A workspace with nothing assigned: the empty list, which is an answer and
/// not a refusal (bl-e9f9). Every script whose test focuses a workspace
/// answers the preload with it.
pub(super) fn nothing_set() -> Vec<u8> {
    json!({ "ok": true, "kind": "roles", "rows": [] })
        .to_string()
        .into_bytes()
}

pub(super) fn ops(requests: &[Vec<u8>]) -> Vec<String> {
    requests
        .iter()
        .map(|r| {
            let v: Value = serde_json::from_slice(r).unwrap();
            v["op"].as_str().unwrap().to_owned()
        })
        .collect()
}

mod deposit;
mod doubt;
mod grace;
mod live;
mod loaded;
mod pick;
mod reads;
mod resume;
mod start;
mod tuning;
mod turn;
