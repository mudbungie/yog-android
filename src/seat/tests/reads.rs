//! The standing set: what the seat re-asks, how focus deepens and backs out,
//! and every way an answer can be the wrong one.

use super::{
    Model, cache_in, conv_reply, material, model_against, nothing_set, ops, pki, queue_quiet,
    serve_many, settle, tr_reply, ws_reply,
};
use crate::transport::Seat;
use serde_json::{Value, json};
use std::time::Duration;

#[test]
fn boot_publishes_the_workspace_roster() {
    let (mut model, served) = model_against(vec![vec![ws_reply()]]);
    let snap = settle(&mut model, &|s| !s.workspaces.is_empty());
    assert_eq!(snap.workspaces[0].workspace, "home");
    assert_eq!(snap.focus, crate::seat::Focus::default());
    assert_eq!(snap.error, None);
    assert!(snap.conversations.is_empty() && snap.transcript.is_empty());
    drop(model);
    assert_eq!(ops(&served.join().unwrap()), ["workspaces"]);
}

#[test]
fn focus_deepens_and_backs_out_of_the_standing_set() {
    let (mut model, served) = model_against(vec![
        vec![ws_reply()], // boot
        vec![nothing_set()],
        vec![ws_reply()],    // focus_workspace: refresh…
        vec![conv_reply()],  // …now two questions deep
        vec![ws_reply()],    // focus_conversation: refresh…
        vec![conv_reply()],  // …
        vec![tr_reply()],    // …three questions deep
        vec![queue_quiet()], // …and the queue, which only this depth asks
        vec![ws_reply()],    // back out: the roster alone again
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    let snap = settle(&mut model, &|s| !s.conversations.is_empty());
    assert_eq!(snap.focus.workspace.as_deref(), Some("home"));
    assert_eq!(snap.focus.agent, None);
    assert_eq!(snap.conversations[0].root_id, "a1");
    model.focus_conversation("home".into(), "a1".into());
    let snap = settle(&mut model, &|s| !s.transcript.is_empty());
    assert_eq!(snap.focus.agent.as_deref(), Some("a1"));
    assert_eq!(snap.transcript[0].name, "001");
    model.focus_workspace(None);
    settle(&mut model, &|s| {
        s.focus.workspace.is_none() && !s.workspaces.is_empty()
    });
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        ops(&requests),
        [
            "workspaces",
            "roles",
            "workspaces",
            "conversations",
            "workspaces",
            "conversations",
            "transcript",
            "attention",
            "workspaces"
        ]
    );
    let transcript: Value = serde_json::from_slice(&requests[6]).unwrap();
    assert_eq!(
        transcript,
        json!({ "op": "transcript", "workspace": "home", "agent": "a1" })
    );
}

#[test]
fn the_cadence_refreshes_unprompted() {
    // A two-connection script and a cadence short enough to fire inside the
    // test but long enough to be OBSERVED: `snapshot` drains to the newest,
    // which is right for a frame and means a test cannot wait on a state the
    // next refresh has already replaced. A cadence near the poll interval
    // makes that race certain rather than rare (it failed only under
    // coverage instrumentation, which is the slow reader this guards).
    let dir = pki();
    let (address, served) = serve_many(&dir, "ca", "server", vec![vec![ws_reply()]; 2]);
    let seat = Seat::open(&material(&dir, "ca", "client", &address)).unwrap();
    let mut model = Model::start(seat, Duration::from_millis(400), cache_in(&dir));
    settle(&mut model, &|s| !s.workspaces.is_empty());
    served.join().unwrap();
    // Which sentence the dead engine earns depends on where the dial met
    // the dying listener (refused outright, or reset after accept) — the
    // fact under test is only that an UNPROMPTED refresh ran: no command
    // was ever sent, so reaching the error at all took the timeout arm.
    let snap = settle(&mut model, &|s| s.error.is_some());
    // And the roster the engine did give is still under the banner: a failed
    // pass republishes the last answer rather than blanking the screen
    // (bl-3202).
    assert!(!snap.workspaces.is_empty());
}
