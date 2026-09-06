//! **The attempt and the read that informs it** (DESIGN §13.16): what picking
//! a fork point asks, and what firing one sends.
//!
//! What is load-bearing here is that **the anchored read carries the commit it
//! was asked at into the value**. A `governing` answer echoes no commit — it
//! is the same shape either question earns — so a policy that landed after the
//! operator tapped another notch could paint under the wrong one unless the
//! fold names the point, which is `drilled`'s guarantee bought at the fold.

use serde_json::{Value, json};

use super::records::opened;
use super::{conv_reply, outcome, settle, tr_reply, ws_reply};

fn governing(oid: &str) -> Vec<u8> {
    json!({ "ok": true, "kind": "governing", "oid": oid, "short_oid": "bbbb",
            "follows": "strict", "diverged_lineages": 0, "files": [] })
    .to_string()
    .into_bytes()
}

fn after() -> Vec<Vec<Vec<u8>>> {
    vec![vec![ws_reply()], vec![conv_reply()], vec![tr_reply()]]
}

fn focused(model: &mut super::Model) {
    settle(model, &|s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    settle(model, &|s| {
        s.focus.agent.is_some() && !s.transcript.is_empty()
    });
}

fn frame(requests: &[Vec<u8>], at: usize) -> Value {
    serde_json::from_slice(requests.get(at).unwrap_or_else(|| unreachable!())).unwrap()
}

/// **Picking a notch asks what governs THERE**, and the answer folds into the
/// records under the commit it was asked at.
#[test]
fn picking_a_fork_point_asks_the_config_governing_it() {
    let mut scripts = opened();
    scripts.push(vec![governing("cafe")]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.open_records();
    settle(&mut model, &|s| s.records.is_some());
    model.anchor("abcdef1234567890".into());
    let snap = settle(&mut model, &|s| {
        s.records.as_ref().is_some_and(|r| r.anchored.is_some())
    });
    let records = snap.records.unwrap_or_else(|| unreachable!());
    let (at, governing) = records.anchored.unwrap_or_else(|| unreachable!());
    assert_eq!(
        at, "abcdef1234567890",
        "the ask names the point, not the answer"
    );
    assert_eq!(governing.follows.as_deref(), Some("strict"));
    drop(model);
    let requests = served.join().unwrap();
    // Five for the focus and its preload, six for the records opening, three
    // for the pass that opening woke: the fifteenth frame is the pick's read.
    assert_eq!(
        frame(&requests, 14),
        json!({ "op": "governing", "workspace": "home", "agent": "a1",
                "at": "abcdef1234567890" })
    );
}

/// **An anchor whose records were retired is dropped**, not held — a policy
/// under no conversation's records is an answer with no subject. And a read
/// that failed is one sentence for the banner.
#[test]
fn an_anchor_with_no_records_standing_is_dropped_and_a_failure_is_a_sentence() {
    let mut scripts = vec![
        vec![ws_reply()],
        vec![super::nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
        vec![governing("cafe")],
    ];
    scripts.extend(after());
    scripts.push(vec![tr_reply()]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.anchor("abcdef1234567890".into());
    let snap = settle(&mut model, &|s| s.focus.agent.is_some());
    assert!(snap.records.is_none(), "nothing to fold it into");
    model.anchor("abcdef1234567890".into());
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("governing: the engine answered transcript instead")
    );
    drop(model);
    served.join().unwrap();
}

/// **The gesture is the whole frame**: this seat's one role, no pinned skills,
/// and the picked point as the ref.
#[test]
fn a_fork_names_the_point_the_role_and_no_skills() {
    let mut scripts = vec![
        vec![ws_reply()],
        vec![super::nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
        vec![outcome(true, "")],
    ];
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.fork("config/strict".into(), "try it the other way".into());
    settle(&mut model, &|s| {
        s.error.is_none() && s.focus.agent.is_some()
    });
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        frame(&requests, 5),
        json!({ "op": "fork", "workspace": "home", "parent": "a1",
                "from": "config/strict", "role": "worker", "skills": [],
                "goal": "try it the other way" })
    );
}

/// **A refusal is the engine's own words**, a wrong kind names the act, and
/// with no conversation focused nothing crosses at all.
#[test]
fn a_fork_refused_says_why_and_one_with_no_conversation_crosses_nothing() {
    let mut scripts = vec![
        vec![ws_reply()],
        vec![super::nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
        vec![outcome(false, "no such ref")],
    ];
    scripts.extend(after());
    scripts.push(vec![tr_reply()]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.fork("config/strict".into(), "g".into());
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(snap.error.as_deref(), Some("fork refused: no such ref"));
    model.fork("config/strict".into(), "g".into());
    let snap = settle(&mut model, &|s| {
        s.error.as_deref() != Some("fork refused: no such ref")
    });
    assert_eq!(
        snap.error.as_deref(),
        Some("fork: the engine answered transcript instead")
    );
    drop(model);
    served.join().unwrap();

    let (mut model, served) = super::model_against(vec![vec![ws_reply()], vec![ws_reply()]]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.fork("config/strict".into(), "g".into());
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("fork: no conversation is focused")
    );
    drop(model);
    assert!(
        super::ops(&served.join().unwrap())
            .iter()
            .all(|op| op == "workspaces")
    );
}
