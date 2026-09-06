//! **The conversation's machinery** (DESIGN §13.11): what an opening asks,
//! what reaches the snapshot, and the two ways it can be asked for and not
//! answered.
//!
//! What is load-bearing here is that the six answers are ONE value carrying
//! the conversation they are about — so a screen paints its own conversation's
//! records or none — and that the drill-in folds INTO them rather than beside
//! them.

use serde_json::{Value, json};

use super::{conv_reply, nothing_set, ops, settle, tr_reply, ws_reply};

fn agent() -> Vec<u8> {
    json!({ "ok": true, "kind": "agent", "display": "Pennant", "root": "a1",
            "state": "quiescent", "present": true, "refused": false, "tip": "abc" })
    .to_string()
    .into_bytes()
}

fn steps() -> Vec<u8> {
    json!({ "ok": true, "kind": "steps", "orphan": "none",
            "rows": [{ "seq": "001", "framing": "complete", "wound": "none",
                       "attempts": 1, "tokens": { "total": 99 } }] })
    .to_string()
    .into_bytes()
}

fn rail() -> Vec<u8> {
    json!({ "ok": true, "kind": "rail", "rows": [], "cards": [] })
        .to_string()
        .into_bytes()
}

fn governing() -> Vec<u8> {
    json!({ "ok": true, "kind": "governing", "oid": "bb", "short_oid": "bbbb",
            "follows": "default", "diverged_lineages": 0, "files": [] })
    .to_string()
    .into_bytes()
}

fn inbox() -> Vec<u8> {
    json!({ "ok": true, "kind": "inbox", "rows": [] })
        .to_string()
        .into_bytes()
}

fn lineages() -> Vec<u8> {
    json!({ "ok": true, "kind": "lineages",
            "rows": [{ "name": "main", "oid": "abcdef1234", "short_oid": "abcdef1",
                       "committed": 1_700_000_000_i64, "files": [] }] })
    .to_string()
    .into_bytes()
}

fn step() -> Vec<u8> {
    json!({ "ok": true, "kind": "step", "seq": "001",
            "meta": { "kind": "absent" }, "request": { "kind": "absent" },
            "staging": { "kind": "absent" }, "response": [], "tools": [] })
    .to_string()
    .into_bytes()
}

/// The scripts every case here shares: the pass, the focus's preload, and the
/// pass after each gesture.
pub(super) fn opened() -> Vec<Vec<Vec<u8>>> {
    vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
        vec![agent()],
        vec![steps()],
        vec![rail()],
        vec![governing()],
        vec![inbox()],
        vec![lineages()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
    ]
}

/// Focus a conversation and wait for the transcript to land under it.
fn focused(model: &mut super::Model) {
    settle(model, &|s| !s.workspaces.is_empty());
    model.focus_conversation("home".into(), "a1".into());
    settle(model, &|s| {
        s.focus.agent.is_some() && !s.transcript.is_empty()
    });
}

/// **Six asks per opening**, each addressed at the conversation the operator
/// is standing in, and the answers reach the snapshot as one value that says
/// which conversation it is about.
#[test]
fn opening_the_records_asks_the_six_and_they_land_as_one_value() {
    let (mut model, served) = super::model_against(opened());
    focused(&mut model);
    model.open_records();
    let snap = settle(&mut model, &|s| s.records.is_some());
    let records = snap.records.unwrap_or_else(|| unreachable!());
    assert!(records.about("home", "a1"));
    assert_eq!(records.head.display, "Pennant");
    assert_eq!(records.steps.rows.first().map(|row| row.tokens), Some(99));
    assert_eq!(records.governing.follows.as_deref(), Some("default"));
    // **The sixth read names the workspace**, and what it lists is what the
    // governing half's `follows` is one of (§13.14).
    assert_eq!(
        records.lineages.first().map(|row| row.name.clone()),
        Some("main".to_owned())
    );
    assert!(records.drilled.is_none());
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        ops(&requests)[5..11],
        ["agent", "steps", "rail", "governing", "inbox", "lineages"]
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&requests[5]).unwrap(),
        json!({ "op": "agent", "workspace": "home", "agent": "a1" })
    );
}

/// **The drill-in folds into the records it belongs to**, carrying back the
/// sequence it was asked by — so the paint asks the answer which row it is
/// under and nothing in the model holds a second name for it.
#[test]
fn one_steps_drill_in_lands_under_the_sequence_it_names() {
    let mut scripts = opened();
    scripts.extend([
        vec![step()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
    ]);
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.open_records();
    settle(&mut model, &|s| s.records.is_some());
    model.drill_step("001".into());
    let snap = settle(&mut model, &|s| {
        s.records.as_ref().is_some_and(|r| r.drilled.is_some())
    });
    let drilled = snap
        .records
        .and_then(|records| records.drilled)
        .unwrap_or_else(|| unreachable!());
    assert_eq!(drilled.seq, "001");
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&requests[14]).unwrap(),
        json!({ "op": "step", "workspace": "home", "agent": "a1", "seq": "001" })
    );
}

/// **A drill-in whose reads were never made is dropped**: a step's records
/// under no conversation's records are rows with no subject.
#[test]
fn a_drill_in_with_no_records_under_it_is_dropped_rather_than_held() {
    let (mut model, served) = super::model_against(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
        vec![step()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
    ]);
    focused(&mut model);
    model.drill_step("001".into());
    let snap = settle(&mut model, &|s| s.focus.agent.is_some());
    assert!(snap.records.is_none());
    drop(model);
    assert_eq!(ops(&served.join().unwrap())[5], "step");
}

/// **Asked with no conversation focused, it is refused here and crosses
/// nothing**: these six are about a conversation, and there is nothing to ask
/// without one.
#[test]
fn the_records_with_no_conversation_focused_cross_nothing() {
    let (mut model, served) = super::model_against(vec![vec![ws_reply()], vec![ws_reply()]]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.open_records();
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("records: no conversation is focused")
    );
    assert!(snap.records.is_none());
    drop(model);
    assert!(
        ops(&served.join().unwrap())
            .iter()
            .all(|op| op == "workspaces"),
        "nothing but the pass crossed"
    );
}

/// **The first read that fails is the whole gesture's answer, and the
/// sentence names the read that was ASKED** — never the one that answered.
/// Each of the six is walked, because a group that reported the wrong one of
/// itself would send an operator looking at the wrong op.
///
/// What the screen already had is not dropped for it either: `searched`'s
/// rule, which every gesture-driven read here keeps.
#[test]
fn a_wrong_kind_at_any_of_the_six_names_that_read_and_keeps_what_was_there() {
    for (at, named) in [
        (0, "agent"),
        (1, "steps"),
        (2, "rail"),
        (3, "governing"),
        (4, "inbox"),
        (5, "lineages"),
    ] {
        let answers = [agent(), steps(), rail(), governing(), inbox(), lineages()];
        let mut scripts = opened();
        scripts.extend(answers.into_iter().take(at).map(|body| vec![body]));
        scripts.extend([
            vec![tr_reply()],
            vec![ws_reply()],
            vec![conv_reply()],
            vec![tr_reply()],
        ]);
        let (mut model, served) = super::model_against(scripts);
        focused(&mut model);
        model.open_records();
        settle(&mut model, &|s| s.records.is_some());
        model.open_records();
        let snap = settle(&mut model, &|s| s.error.is_some());
        assert_eq!(
            snap.error.as_deref(),
            Some(format!("{named}: the engine answered transcript instead").as_str())
        );
        assert!(
            snap.records
                .is_some_and(|records| records.about("home", "a1")),
            "the answer it had is still the answer it has"
        );
        drop(model);
        served.join().unwrap();
    }
}

/// **The drill-in's own wrong kind is a sentence too**, and the records it
/// would have landed under keep standing.
#[test]
fn a_wrong_kind_for_one_step_names_that_read() {
    let mut scripts = opened();
    scripts.extend([
        vec![tr_reply()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![tr_reply()],
    ]);
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.open_records();
    settle(&mut model, &|s| s.records.is_some());
    model.drill_step("001".into());
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("step: the engine answered transcript instead")
    );
    assert!(
        snap.records
            .is_some_and(|records| records.drilled.is_none())
    );
    drop(model);
    served.join().unwrap();
}
