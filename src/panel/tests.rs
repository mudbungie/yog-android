//! The card's facts: which, in what order, and which fold behind `+`.

use super::{LONG, conversation};
use crate::codec::{AgentState, ConvRow, Entry, EntryKind, Tone};
use crate::theme;

fn row(id: &str, depth: usize) -> ConvRow {
    ConvRow {
        root_id: id.into(),
        display: format!("{id}-name"),
        name: None,
        display_only: false,
        state: AgentState::Quiescent,
        uncertain: false,
        preview: String::new(),
        age_secs: 0,
        last_active_unix: 0,
        flight: None,
        attention: 0,
        members: 1,
        direct: 0,
        stoppable: false,
        stop_children: false,
        depth,
        tone: Tone::Plain,
        failure: None,
        alignment: None,
        ball: None,
    }
}

fn turn(model: &str, usage: serde_json::Value) -> Entry {
    Entry {
        name: "001.json".into(),
        kind: EntryKind::Model {
            model_id: model.into(),
            blocks: Vec::new(),
            usage,
        },
        raw: String::new(),
    }
}

fn texts(rows: &[ConvRow], agent: &str, transcript: &[Entry]) -> Vec<String> {
    conversation(rows, agent, "infra", transcript)
        .into_iter()
        .map(|fact| fact.text)
        .collect()
}

#[test]
fn a_root_says_its_name_its_workspace_and_what_answered() {
    let rows = [row("a", 0)];
    let said = [
        turn("old", serde_json::json!({})),
        turn(
            "opus",
            serde_json::json!({"input_tokens": 1200, "output_tokens": 30}),
        ),
        Entry {
            name: "002.json".into(),
            kind: EntryKind::Raw,
            raw: String::new(),
        },
    ];
    assert_eq!(
        texts(&rows, "a", &said),
        ["a-name", "on infra", "opus · 1.2k in · 30 out"]
    );
    let facts = conversation(&rows, "a", "infra", &said);
    assert_eq!(facts[0].ink, theme::INK);
    assert!(facts[1..].iter().all(|fact| fact.ink == theme::INK_WEAK));
}

#[test]
fn a_conversation_nothing_has_answered_says_only_where_it_is() {
    assert_eq!(texts(&[], "zz", &[]), ["zz", "on infra"]);
    assert_eq!(
        texts(&[], "zz", &[turn("opus", serde_json::json!({}))]),
        ["zz", "on infra", "opus"]
    );
}

#[test]
fn a_member_says_where_it_hangs_root_first() {
    let rows = [
        row("r", 0),
        row("x", 1),
        row("x2", 2),
        row("p", 1),
        row("q", 2),
        row("k", 3),
    ];
    let said = texts(&rows, "k", &[]);
    assert_eq!(
        said.last().map(String::as_str),
        Some("under r-name / p-name / q-name")
    );
    assert_eq!(texts(&rows, "r", &[]).len(), 2, "a root hangs nowhere");
}

#[test]
fn a_long_fact_folds_behind_the_plus() {
    let long = "m".repeat(LONG + 1);
    let facts = conversation(&[], "a", "w", &[turn(&long, serde_json::json!({}))]);
    assert!(!facts[0].long);
    assert!(facts[2].long);
}
