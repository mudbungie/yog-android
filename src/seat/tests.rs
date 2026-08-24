//! The model end to end, over the real transport against the scripted
//! multi-connection server: every gesture triggers exactly one refresh pass,
//! so with a very long cadence each test's connection count is a script, not
//! a race. The requests the server read back are asserted too — the model's
//! side of the wire is pinned, not assumed.

use super::{Model, Snapshot};
use crate::test_support::{material, mint_ca, mint_leaf, scratch, serve_many};
use crate::transport::Seat;
use serde_json::{Value, json};
use std::time::Duration;

/// Long enough that no unprompted refresh fires inside a test.
const REST: Duration = Duration::from_hours(1);

fn pki() -> std::path::PathBuf {
    let dir = scratch();
    mint_ca(&dir, "ca");
    mint_leaf(&dir, "ca", "server", true);
    mint_leaf(&dir, "ca", "client", false);
    dir
}

fn model_against(scripts: Vec<Vec<Vec<u8>>>) -> (Model, std::thread::JoinHandle<Vec<Vec<u8>>>) {
    let dir = pki();
    let (address, served) = serve_many(&dir, "ca", "server", scripts);
    let seat = Seat::open(&material(&dir, "ca", "client", &address)).unwrap();
    (Model::start(seat, REST), served)
}

/// Poll until a snapshot satisfies `pass` — the worker publishes on its own
/// thread, so the test waits the way a frame would, just faster.
fn settle<F: Fn(&Snapshot) -> bool>(model: &mut Model, pass: F) -> Snapshot {
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

fn ws_reply() -> Vec<u8> {
    json!({ "ok": true, "kind": "workspaces",
            "rows": [{ "workspace": "home", "kind": "named", "attention": 0,
                       "agents": 1, "running": false }] })
    .to_string()
    .into_bytes()
}

fn conv_reply() -> Vec<u8> {
    json!({ "ok": true, "kind": "conversations",
            "rows": [{ "root_id": "a1", "display": "d", "display_only": false,
                       "state": "quiescent", "uncertain": false, "preview": "",
                       "age_secs": 0, "attention": 0, "members": 1, "direct": 0,
                       "stoppable": false, "stop_children": false, "depth": 0,
                       "tone": "plain" }] })
    .to_string()
    .into_bytes()
}

fn tr_reply() -> Vec<u8> {
    json!({ "ok": true, "kind": "transcript",
            "rows": [{ "name": "001", "raw": "", "kind": "raw" }] })
    .to_string()
    .into_bytes()
}

fn outcome(ok: bool, stderr: &str) -> Vec<u8> {
    json!({ "ok": ok, "kind": "outcome", "exit": i32::from(!ok),
            "stdout": "", "stderr": stderr })
    .to_string()
    .into_bytes()
}

fn ops(requests: &[Vec<u8>]) -> Vec<String> {
    requests
        .iter()
        .map(|r| {
            let v: Value = serde_json::from_slice(r).unwrap();
            v["op"].as_str().unwrap().to_owned()
        })
        .collect()
}

#[test]
fn boot_publishes_the_workspace_roster() {
    let (mut model, served) = model_against(vec![vec![ws_reply()]]);
    let snap = settle(&mut model, |s| !s.workspaces.is_empty());
    assert_eq!(snap.workspaces[0].workspace, "home");
    assert_eq!(snap.focus, super::Focus::default());
    assert_eq!(snap.error, None);
    assert!(snap.conversations.is_empty() && snap.transcript.is_empty());
    drop(model);
    assert_eq!(ops(&served.join().unwrap()), ["workspaces"]);
}

#[test]
fn focus_deepens_and_backs_out_of_the_standing_set() {
    let (mut model, served) = model_against(vec![
        vec![ws_reply()],   // boot
        vec![ws_reply()],   // focus_workspace: refresh…
        vec![conv_reply()], // …now two questions deep
        vec![ws_reply()],   // focus_conversation: refresh…
        vec![conv_reply()], // …
        vec![tr_reply()],   // …three questions deep
        vec![ws_reply()],   // back out: the roster alone again
    ]);
    settle(&mut model, |s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    let snap = settle(&mut model, |s| !s.conversations.is_empty());
    assert_eq!(snap.focus.workspace.as_deref(), Some("home"));
    assert_eq!(snap.focus.agent, None);
    assert_eq!(snap.conversations[0].root_id, "a1");
    model.focus_conversation("home".into(), "a1".into());
    let snap = settle(&mut model, |s| !s.transcript.is_empty());
    assert_eq!(snap.focus.agent.as_deref(), Some("a1"));
    assert_eq!(snap.transcript[0].name, "001");
    model.focus_workspace(None);
    settle(&mut model, |s| {
        s.focus.workspace.is_none() && !s.workspaces.is_empty()
    });
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        ops(&requests),
        [
            "workspaces",
            "workspaces",
            "conversations",
            "workspaces",
            "conversations",
            "transcript",
            "workspaces"
        ]
    );
    let transcript: Value = serde_json::from_slice(&requests[5]).unwrap();
    assert_eq!(
        transcript,
        json!({ "op": "transcript", "workspace": "home", "agent": "a1" })
    );
}

#[test]
fn a_deposit_posts_the_composer_and_refreshes() {
    let (mut model, served) = model_against(vec![
        vec![ws_reply()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
        vec![outcome(true, "")], // the deposit's receipt
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
    ]);
    settle(&mut model, |s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    settle(&mut model, |s| !s.transcript.is_empty());
    model.deposit("hello".into());
    // The post-deposit refresh publishes with no error: the receipt was ok.
    settle(&mut model, |s| {
        !s.transcript.is_empty() && s.error.is_none()
    });
    drop(model);
    let requests = served.join().unwrap();
    let message: Value = serde_json::from_slice(&requests[4]).unwrap();
    assert_eq!(
        message,
        json!({ "op": "message", "workspace": "home", "agent": "a1", "content": "hello" })
    );
}

#[test]
fn a_refused_deposit_reaches_the_banner() {
    let (mut model, _served) = model_against(vec![
        vec![ws_reply()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
        vec![outcome(false, "gate red")],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
    ]);
    settle(&mut model, |s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    settle(&mut model, |s| !s.transcript.is_empty());
    model.deposit("hello".into());
    let snap = settle(&mut model, |s| s.error.is_some());
    assert_eq!(snap.error.as_deref(), Some("deposit refused: gate red"));
}

#[test]
fn an_unfocused_deposit_and_a_dead_engine_share_the_banner() {
    // One scripted connection; once it is served the listener is gone, so
    // the post-deposit refresh fails too — both sentences join the banner.
    let (mut model, served) = model_against(vec![vec![ws_reply()]]);
    settle(&mut model, |s| !s.workspaces.is_empty());
    served.join().unwrap();
    model.deposit("hello".into());
    let snap = settle(&mut model, |s| s.error.is_some());
    let banner = snap.error.unwrap();
    assert!(
        banner.starts_with("deposit: no conversation is focused; connect"),
        "banner: {banner}"
    );
    assert!(snap.workspaces.is_empty());
}

#[test]
fn wrong_reply_kinds_name_the_kind() {
    // workspaces answered with conversations.
    let (mut model, _s) = model_against(vec![vec![conv_reply()]]);
    let snap = settle(&mut model, |s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("workspaces: the engine answered conversations instead")
    );
    drop(model);

    // conversations answered with an outcome.
    let (mut model, _s) = model_against(vec![
        vec![ws_reply()],
        vec![ws_reply()],
        vec![outcome(true, "")],
    ]);
    settle(&mut model, |s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    let snap = settle(&mut model, |s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("conversations: the engine answered outcome instead")
    );
    drop(model);

    // transcript answered with workspaces.
    let (mut model, _s) = model_against(vec![
        vec![ws_reply()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![ws_reply()],
    ]);
    settle(&mut model, |s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    let snap = settle(&mut model, |s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("transcript: the engine answered workspaces instead")
    );
    drop(model);

    // the deposit's receipt answered with a transcript.
    let (mut model, _s) = model_against(vec![
        vec![ws_reply()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
        vec![tr_reply()], // wrong: the receipt slot
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
    ]);
    settle(&mut model, |s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    settle(&mut model, |s| !s.transcript.is_empty());
    model.deposit("hello".into());
    let snap = settle(&mut model, |s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("deposit: the engine answered transcript instead")
    );
}

#[test]
fn the_cadence_refreshes_unprompted() {
    // A short cadence and a two-connection script: the second refresh is
    // driven by the timeout arm alone (no command is ever sent), and once
    // the script is exhausted the third pass reports the dead engine.
    let dir = pki();
    let (address, served) = serve_many(&dir, "ca", "server", vec![vec![ws_reply()]; 2]);
    let seat = Seat::open(&material(&dir, "ca", "client", &address)).unwrap();
    let mut model = Model::start(seat, Duration::from_millis(10));
    settle(&mut model, |s| !s.workspaces.is_empty());
    served.join().unwrap();
    // Which sentence the dead engine earns depends on where the dial met
    // the dying listener (refused outright, or reset after accept) — the
    // fact under test is only that an UNPROMPTED refresh ran: no command
    // was ever sent, so reaching the error at all took the timeout arm.
    let snap = settle(&mut model, |s| s.error.is_some());
    assert!(snap.workspaces.is_empty());
}
