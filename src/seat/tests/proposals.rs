//! **The learning loop's veto, end to end** (DESIGN §13.17, REMOTE §9.22):
//! what the listing asks, what the reading beside it asks, and what a settle
//! sends — and the one thing a settle owes afterwards, which is the listing
//! read again.
//!
//! What is load-bearing here is the same fact the two admin reads carry: the
//! answer echoes no workspace, so the ask names it and the value comes back
//! paired — a listing under another workspace's name would be the wrong claim
//! about somebody's config.

use serde_json::{Value, json};

use super::{conv_reply, nothing_set, settle, tr_reply, ws_reply};
use crate::codec::AdminAct;
use crate::codec::proposals::Verdict;

fn row(id: &str) -> Value {
    json!({ "id": id, "lineages": ["default"], "parent": "9f2c1ab4", "fresh": true,
            "diffstat": "1 file changed, 6 insertions(+)",
            "subject": "notes: record what the span taught" })
}

fn listing(ids: &[&str]) -> Vec<u8> {
    json!({ "ok": true, "kind": "proposals",
            "rows": ids.iter().map(|id| row(id)).collect::<Vec<_>>() })
    .to_string()
    .into_bytes()
}

fn whole(id: &str) -> Vec<u8> {
    json!({ "ok": true, "kind": "proposals", "rows": [row(id)],
            "whole": "commit 71011c3d\n\n+ a line\n" })
    .to_string()
    .into_bytes()
}

fn focused_scripts() -> Vec<Vec<Vec<u8>>> {
    vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
    ]
}

fn after() -> Vec<Vec<Vec<u8>>> {
    vec![vec![ws_reply()], vec![conv_reply()]]
}

fn focused(model: &mut super::Model) {
    settle(model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(model, &|s| !s.conversations.is_empty());
}

fn frame(requests: &[Vec<u8>], at: usize) -> Value {
    serde_json::from_slice(requests.get(at).unwrap_or_else(|| unreachable!())).unwrap()
}

/// **The bare listing is aimed, and the id is absent rather than null** — a
/// key written as null would be a third thing to read.
#[test]
fn the_listing_is_aimed_and_names_no_proposal() {
    let mut scripts = focused_scripts();
    scripts.push(vec![listing(&["r1", "r2"])]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.list_proposals(None);
    let snap = settle(&mut model, &|s| s.proposals.is_some());
    let staged = snap.proposals.unwrap_or_else(|| unreachable!());
    assert!(staged.about("home"));
    assert_eq!(staged.rows.len(), 2);
    assert_eq!(staged.whole, None);
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        frame(&requests, 4),
        json!({ "op": "proposals", "workspace": "home" })
    );
}

/// **Naming one asks the same op at its second depth** and the reading rides
/// beside the listing rather than instead of it.
#[test]
fn naming_a_proposal_asks_the_same_op_and_answers_it_whole() {
    let mut scripts = focused_scripts();
    scripts.push(vec![whole("r1")]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.list_proposals(Some("r1".into()));
    let snap = settle(&mut model, &|s| s.proposals.is_some());
    let staged = snap.proposals.unwrap_or_else(|| unreachable!());
    assert_eq!(staged.rows.len(), 1);
    assert!(staged.whole.is_some_and(|text| text.contains("+ a line")));
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        frame(&requests, 4),
        json!({ "op": "proposals", "workspace": "home", "id": "r1" })
    );
}

/// With nothing focused the read refuses in the words every aimed read here
/// refuses in, and an answer of another kind names the read and keeps what was
/// there.
#[test]
fn an_unaimed_listing_refuses_and_a_wrong_kind_names_it() {
    let (mut model, served) = super::model_against(vec![vec![ws_reply()], vec![ws_reply()]]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.list_proposals(None);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(snap.error.as_deref(), Some("no workspace is focused"));
    drop(model);
    served.join().unwrap();

    let mut scripts = focused_scripts();
    scripts.push(vec![listing(&["r1"])]);
    scripts.extend(after());
    scripts.push(vec![tr_reply()]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.list_proposals(None);
    settle(&mut model, &|s| s.proposals.is_some());
    model.list_proposals(None);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("proposals: the engine answered transcript instead")
    );
    assert!(snap.proposals.is_some_and(|held| held.rows.len() == 1));
    drop(model);
    served.join().unwrap();
}

/// **A settle names both words and is followed by the listing read again**
/// (the trail acts' rule, one surface along): the branch it took is gone, so a
/// screen standing on the old listing would still be offering a row that no
/// longer exists.
#[test]
fn a_settle_names_both_words_and_the_listing_is_read_again() {
    let mut scripts = focused_scripts();
    scripts.push(vec![listing(&["r1", "r2"])]);
    scripts.extend(after());
    scripts.push(vec![super::outcome(true, "")]);
    scripts.push(vec![listing(&["r2"])]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.list_proposals(None);
    settle(&mut model, &|s| s.proposals.is_some());
    model.admin(AdminAct::Proposal {
        workspace: "home".into(),
        id: "r1".into(),
        verdict: Verdict::Accept,
    });
    let snap = settle(&mut model, &|s| {
        s.proposals
            .as_ref()
            .is_some_and(|held| held.rows.len() == 1)
    });
    assert_eq!(snap.error, None);
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        frame(&requests, 7),
        json!({ "op": "proposal", "workspace": "home", "id": "r1", "verdict": "accept" })
    );
    assert_eq!(
        frame(&requests, 8),
        json!({ "op": "proposals", "workspace": "home" }),
        "the settle is invisible until the listing is asked again"
    );
}

/// **A refused settle is litany's own sentence**, which is where a stale
/// proposal, an ambiguous one and an unknown id come back — this seat has no
/// second opinion about any of them.
#[test]
fn a_refused_settle_says_the_engines_words() {
    let mut scripts = focused_scripts();
    scripts.push(vec![super::outcome(
        false,
        "proposal r1 is stale: default now stands at 3ac70e11",
    )]);
    scripts.push(vec![listing(&["r1"])]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.admin(AdminAct::Proposal {
        workspace: "home".into(),
        id: "r1".into(),
        verdict: Verdict::Reject,
    });
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("proposal refused: proposal r1 is stale: default now stands at 3ac70e11")
    );
    drop(model);
    served.join().unwrap();
}
