//! **The search read** (yog DESIGN §8.5, bl-4c2b): the envelope it puts on
//! the wire, the answer it holds, and the two ways an answer stops standing.
//!
//! The load-bearing assertions are about the ABSENCE of a wire crossing. A
//! cleared needle asks nothing — the answer being dropped is this seat's own
//! copy, and an operator with an unreachable engine must still be able to
//! leave a search — and a failed search drops nothing, because losing the
//! hits the engine did give over one it did not is the same defect twice.

use serde_json::{Value, json};

use super::{outcome, settle, ws_reply};
use crate::codec::Address;

/// One conversation hit, as `corpus/reply/search.json` spells one.
fn hit_reply(needle: &str) -> Vec<u8> {
    json!({ "ok": true, "kind": "search", "needle": needle,
            "rows": [{ "at": "conversation", "workspace": "home", "agent": "a1",
                       "field": "text", "offset": 12, "excerpt": "the gate" }],
            "unreadable": [] })
    .to_string()
    .into_bytes()
}

/// The engine's own spelling of *no search*: an answer with no needle.
fn no_search() -> Vec<u8> {
    json!({ "ok": true, "kind": "search", "needle": "", "rows": [], "unreadable": [] })
        .to_string()
        .into_bytes()
}

/// The envelope, and the hit landing in the snapshot as an address this seat
/// can focus.
#[test]
fn the_needle_crosses_the_wire_and_its_hits_reach_the_snapshot() {
    let (mut model, served) = super::model_against(vec![
        vec![ws_reply()],
        vec![hit_reply("gate")],
        vec![ws_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.search("gate".into());
    let snap = settle(&mut model, &|s| s.search.is_some());
    let found = snap.search.unwrap();
    assert_eq!(found.needle, "gate");
    assert_eq!(
        found.hits[0].at,
        Address::Conversation {
            workspace: "home".into(),
            agent: "a1".into()
        }
    );
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&requests[1]).unwrap(),
        json!({ "op": "search", "text": "gate" })
    );
}

/// **A search that matched nothing is not no search** (upstream bl-648a). It
/// is an answer carrying its own needle, and the screen it selects says so.
#[test]
fn an_answer_with_no_hits_still_stands() {
    let empty = json!({ "ok": true, "kind": "search", "needle": "gate",
                        "rows": [], "unreadable": ["p: balls unlistable"] })
    .to_string()
    .into_bytes();
    let (mut model, _s) =
        super::model_against(vec![vec![ws_reply()], vec![empty], vec![ws_reply()]]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.search("gate".into());
    let found = settle(&mut model, &|s| s.search.is_some()).search.unwrap();
    assert!(found.hits.is_empty());
    assert_eq!(found.unreadable, ["p: balls unlistable"]);
}

/// The clear: an empty needle drops the answer and asks the engine nothing.
/// The connection count is the assertion — a clear that crossed the wire
/// would be a search an unreachable engine could refuse to let go of.
#[test]
fn an_empty_needle_clears_the_answer_without_asking_anything() {
    let (mut model, served) = super::model_against(vec![
        vec![ws_reply()],
        vec![hit_reply("gate")],
        vec![ws_reply()],
        vec![ws_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.search("gate".into());
    settle(&mut model, &|s| s.search.is_some());
    model.search("   ".into());
    settle(&mut model, &|s| s.search.is_none());
    drop(model);
    assert_eq!(
        super::ops(&served.join().unwrap()),
        ["workspaces", "search", "workspaces", "workspaces"]
    );
}

/// The engine's own clear reads the same way: an answer with an empty needle
/// is no answer, and the seat holds none.
#[test]
fn an_answer_with_an_empty_needle_is_no_answer() {
    let (mut model, _s) = super::model_against(vec![
        vec![ws_reply()],
        vec![hit_reply("gate")],
        vec![ws_reply()],
        vec![no_search()],
        vec![ws_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.search("gate".into());
    settle(&mut model, &|s| s.search.is_some());
    model.search("gate".into());
    settle(&mut model, &|s| s.search.is_none());
}

/// A search answered with another shape names what came back — and the hits
/// already on the glass stay there.
#[test]
fn a_search_answered_with_the_wrong_shape_keeps_the_standing_hits() {
    let (mut model, _s) = super::model_against(vec![
        vec![ws_reply()],
        vec![hit_reply("gate")],
        vec![ws_reply()],
        vec![outcome(true, "")],
        vec![ws_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.search("gate".into());
    settle(&mut model, &|s| s.search.is_some());
    model.search("gate".into());
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("search: the engine answered outcome instead")
    );
    assert_eq!(snap.search.unwrap().needle, "gate");
}
