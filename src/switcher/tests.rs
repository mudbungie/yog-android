//! Which rows, in which order, and what a tap on each yields.

use super::{Entry, Taken, entries};
use crate::codec::{AgentState, ConvRow, Tone, WsKind, WsRow};

fn ws(name: &str, attention: usize) -> WsRow {
    WsRow {
        workspace: name.to_owned(),
        kind: WsKind::Named,
        attention,
        agents: 2,
        running: false,
        pinned: None,
        config_tip: None,
    }
}

fn conv(root_id: &str, last_active_unix: i64, depth: usize) -> ConvRow {
    ConvRow {
        root_id: root_id.to_owned(),
        display: root_id.to_owned(),
        name: None,
        display_only: false,
        state: AgentState::Quiescent,
        uncertain: false,
        preview: String::new(),
        age_secs: 0,
        last_active_unix,
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

/// A row's short spelling, so an order reads as one line.
fn said(entry: &Entry) -> String {
    match entry {
        Entry::Heading {
            workspace, open, ..
        } => format!("{}{workspace}", if *open { "v" } else { ">" }),
        Entry::Conversation { row, rails, .. } => format!("{}:{}", row.root_id, rails.len()),
        Entry::New { workspace } => format!("+{workspace}"),
    }
}

#[test]
fn only_the_focused_workspace_opens_and_its_tree_keeps_the_roster_order() {
    let rows = [conv("old", 1, 0), conv("kid", 1, 1), conv("new", 9, 0)];
    let all = entries(
        &[ws("a", 0), ws("main", 1), ws("z", 0)],
        Some("main"),
        &rows,
    );
    let order: Vec<String> = all.iter().map(said).collect();
    assert_eq!(
        order,
        [">a", "vmain", "+main", "new:0", "old:0", "kid:1", ">z"]
    );
    assert!(matches!(
        &all[1],
        Entry::Heading {
            waiting: true,
            agents: 2,
            ..
        }
    ));
    assert!(matches!(&all[0], Entry::Heading { waiting: false, .. }));
}

#[test]
fn with_nothing_focused_every_workspace_is_folded() {
    let all = entries(&[ws("a", 0), ws("b", 0)], None, &[conv("x", 1, 0)]);
    let order: Vec<String> = all.iter().map(said).collect();
    assert_eq!(order, [">a", ">b"]);
    assert!(entries(&[], None, &[]).is_empty());
}

#[test]
fn an_empty_workspace_still_offers_a_new_conversation() {
    let all = entries(&[ws("main", 0)], Some("main"), &[]);
    let order: Vec<String> = all.iter().map(said).collect();
    assert_eq!(order, ["vmain", "+main"]);
}

#[test]
fn a_tap_focuses_a_folded_heading_opens_a_conversation_and_new_starts() {
    let all = entries(
        &[ws("a", 0), ws("main", 0)],
        Some("main"),
        &[conv("c1", 1, 0)],
    );
    let taken: Vec<Option<Taken>> = all.iter().map(Entry::taken).collect();
    assert_eq!(
        taken,
        [
            Some(Taken::Focus("a".into())),
            None,
            Some(Taken::Start("main".into())),
            Some(Taken::Open {
                workspace: "main".into(),
                agent: "c1".into()
            }),
        ]
    );
}
