//! **The three acts on an obligation** (DESIGN §13.12), split from the read
//! they hang on (bl-2f17) when the pair took one file past the 300 cap. The
//! seam is the screen's own: what the listing SAYS is next door, and this is
//! what is done to a row of it.
//!
//! What is load-bearing here is the CHAIN: a fan is three gestures — stage,
//! spread, then one firing per candidate — and the requests are asserted, not
//! assumed, because nothing else says the seat fires what it materialized.

use serde_json::{Value, json};

use super::super::{
    Turn, conv_reply, model_against, model_turns, nothing_set, ops, outcome, prepared, settle,
    tr_reply, ws_reply,
};
use super::{after, delivered, fanned, focused, focused_scripts, retired, science, started};
use crate::codec::CandidateAct;

/// **A delivery crosses with the row's own handle**, and the listing is read
/// again straight after — which is also the read that settles a lost one.
/// A delivery that landed no commit says so: silence would report a delivery
/// that did not happen.
#[test]
fn a_delivery_carries_the_handle_and_a_landing_of_nothing_is_said() {
    let mut scripts = focused_scripts();
    scripts.extend([vec![delivered(true)], vec![science()]]);
    scripts.extend(after());
    scripts.extend([vec![delivered(false)], vec![science()]]);
    scripts.extend(after());
    let (mut model, served) = model_against(scripts);
    focused(&mut model);
    let act = CandidateAct::Deliver {
        handle: "at-1".into(),
        summary: "the winner".into(),
    };
    model.candidate_act("p".into(), "bl-1".into(), act.clone());
    let snap = settle(&mut model, &|s| s.candidates.is_some());
    assert!(snap.error.is_none());
    model.candidate_act("p".into(), "bl-1".into(), act);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("delivered onto main: nothing landed — the source ref moved nothing")
    );
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&requests[4]).unwrap(),
        json!({ "op": "deliver", "project": "p", "ball": "bl-1",
                "handle": "at-1", "summary": "the winner" })
    );
    assert_eq!(ops(&requests)[5], "science");
}

/// **Which way the retention went is the engine's answer**, and the seat
/// paints it rather than predicting a policy it has not read.
#[test]
fn a_retirement_says_whether_the_source_ref_went_with_the_worktree() {
    let mut scripts = focused_scripts();
    scripts.extend([vec![retired(true)], vec![science()]]);
    scripts.extend(after());
    scripts.extend([vec![retired(false)], vec![science()]]);
    scripts.extend(after());
    scripts.extend([vec![tr_reply()], vec![science()]]);
    scripts.extend(after());
    let (mut model, served) = model_against(scripts);
    focused(&mut model);
    let act = CandidateAct::Retire {
        handle: "at-1".into(),
    };
    model.candidate_act("p".into(), "bl-1".into(), act.clone());
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("retired: the worktree is released and its source ref is discarded")
    );
    model.candidate_act("p".into(), "bl-1".into(), act.clone());
    let snap = settle(&mut model, &|s| {
        s.error.as_deref().is_some_and(|why| why.ends_with("kept"))
    });
    assert!(snap.candidates.is_some());
    model.candidate_act("p".into(), "bl-1".into(), act);
    let snap = settle(&mut model, &|s| {
        s.error
            .as_deref()
            .is_some_and(|why| why.contains("instead"))
    });
    assert_eq!(
        snap.error.as_deref(),
        Some("retire: the engine answered transcript instead")
    );
    drop(model);
    served.join().unwrap();
}

/// **A fan is three gestures**: stage, spread, then one firing per candidate
/// with the operator's own goal. Firing is the completion of the act — a
/// candidate prepared and never fired is a worktree balls made for nothing.
#[test]
fn a_fan_stages_spreads_and_fires_every_candidate_it_materialized() {
    let mut scripts = focused_scripts();
    scripts.extend([
        vec![prepared()],
        vec![fanned(2)],
        vec![started()],
        vec![started()],
        vec![science()],
    ]);
    scripts.extend(after());
    let (mut model, served) = model_against(scripts);
    focused(&mut model);
    model.fan("p".into(), "bl-1".into(), 2, "do it".into());
    let snap = settle(&mut model, &|s| s.candidates.is_some());
    assert!(snap.error.is_none());
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        ops(&requests)[4..9],
        ["prepare", "fan", "prompt", "prompt", "science"]
    );
    let fan: Value = serde_json::from_slice(&requests[5]).unwrap();
    assert_eq!(fan["op"], "fan");
    assert_eq!(
        (
            fan["project"].clone(),
            fan["ball"].clone(),
            fan["n"].clone()
        ),
        (json!("p"), json!("bl-1"), json!(2))
    );
    let fired: Value = serde_json::from_slice(&requests[6]).unwrap();
    assert_eq!(fired["goal"], "do it");
    assert_eq!(fired["prepared"]["binding"], "/candidate");
}

/// **Every leg of the chain can fail, and the sentence names the fan.** The
/// staging's wrong kind, the spread's, and a firing the engine refused.
#[test]
fn every_leg_of_the_chain_names_the_fan_when_it_stops() {
    let mut scripts = focused_scripts();
    for stopped in [
        vec![vec![tr_reply()]],
        vec![vec![prepared()], vec![nothing_set()]],
        vec![
            vec![prepared()],
            vec![fanned(1)],
            vec![outcome(false, "no")],
        ],
        vec![vec![prepared()], vec![fanned(1)], vec![nothing_set()]],
    ] {
        scripts.extend(stopped);
        scripts.push(vec![science()]);
        scripts.extend(after());
    }
    let (mut model, served) = model_against(scripts);
    focused(&mut model);
    for said in [
        "fan: the engine answered transcript instead",
        "fan: the engine answered roles instead",
        "fan refused: no",
        "fan: the engine answered roles instead",
    ] {
        model.fan("p".into(), "bl-1".into(), 2, "do it".into());
        // The sentence itself is the predicate: three of these are the same
        // gesture failing at three different legs, so waiting for *an* error
        // would read the previous leg's.
        let snap = settle(&mut model, &|s| s.error.as_deref() == Some(said));
        assert!(snap.candidates.is_some(), "the listing was read anyway");
    }
    drop(model);
    served.join().unwrap();
}

/// **A fan with no workspace focused crosses nothing**: the staging it would
/// begin with names one.
#[test]
fn a_fan_with_no_workspace_focused_crosses_nothing() {
    let (mut model, served) = model_against(vec![vec![ws_reply()], vec![ws_reply()]]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.fan("p".into(), "bl-1".into(), 2, "do it".into());
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(snap.error.as_deref(), Some("fan: no workspace is focused"));
    drop(model);
    assert!(
        ops(&served.join().unwrap())
            .iter()
            .all(|op| op == "workspaces")
    );
}

/// **Every leg is an act in doubt when its reply is lost**, and none of them
/// is ever sent again: a repeated fan is n more worktrees and a repeated
/// deliver is a second delivery. The sentence names the listing, which is the
/// read that settles all four.
#[test]
fn a_lost_reply_on_any_leg_is_in_doubt_and_names_the_listing() {
    let focused_turns = || {
        vec![
            Turn::Answer(vec![ws_reply()]),
            Turn::Answer(vec![nothing_set()]),
            Turn::Answer(vec![ws_reply()]),
            Turn::Answer(vec![conv_reply()]),
        ]
    };
    let legs: Vec<(&str, Vec<Vec<u8>>)> = vec![
        ("deliver", vec![]),
        ("fan", vec![]),
        ("fan", vec![prepared()]),
        ("fan", vec![prepared(), fanned(1)]),
    ];
    for (which, answered) in legs {
        let mut turns = focused_turns();
        turns.extend(answered.into_iter().map(|body| Turn::Answer(vec![body])));
        turns.push(Turn::Hangup);
        turns.extend([
            Turn::Answer(vec![science()]),
            Turn::Answer(vec![ws_reply()]),
            Turn::Answer(vec![conv_reply()]),
        ]);
        let (mut model, _served) = model_turns(turns);
        settle(&mut model, &|s| !s.workspaces.is_empty());
        model.focus_workspace(Some("home".into()));
        settle(&mut model, &|s| !s.conversations.is_empty());
        if which == "deliver" {
            model.candidate_act(
                "p".into(),
                "bl-1".into(),
                CandidateAct::Deliver {
                    handle: "at-1".into(),
                    summary: "the winner".into(),
                },
            );
        } else {
            model.fan("p".into(), "bl-1".into(), 2, "do it".into());
        }
        let said = settle(&mut model, &|s| s.error.is_some())
            .error
            .unwrap_or_default();
        assert!(
            said.starts_with(&format!("{which} may have run:")),
            "{said}"
        );
        assert!(said.contains("The candidates are listed again"), "{said}");
    }
}
