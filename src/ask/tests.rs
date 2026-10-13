//! Which cells the ask offers, their words and order, what each fires, and
//! the words above them — off the engine's own queue rows.

use serde_json::Value;

use super::{Choice, asking, cells, choices, parked, words};
use crate::codec::{QueueRow, Scope, Verdict};
use crate::theme::{self, State};

/// The corpus's two rows: the first holds a call and says everything, the
/// second says nothing.
fn corpus() -> Vec<QueueRow> {
    let text = std::fs::read_to_string("corpus/reply/attention.json").unwrap();
    let doc: Value = serde_json::from_str(&text).unwrap();
    doc["frames"][0]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| crate::codec::queue::row(v).unwrap())
        .collect()
}

#[test]
fn a_parked_call_offers_six_cells_two_to_a_row_in_the_desktops_words() {
    let rows = corpus();
    let said: Vec<Vec<String>> = cells(&rows[0])
        .into_iter()
        .map(|row| row.into_iter().map(|(word, _)| word).collect())
        .collect();
    assert_eq!(
        said,
        [
            ["1. seen", "2. pass"],
            [
                "3. pass for this conversation",
                "4. pass for this workspace"
            ],
            ["5. refuse", "6. hold"],
        ]
    );
}

#[test]
fn nothing_parked_is_seen_alone() {
    let rows = corpus();
    assert_eq!(choices(&rows[1]), [Choice::Seen]);
    assert_eq!(cells(&rows[1]).len(), 1);
    assert!(parked(&rows[1]).is_none());
}

#[test]
fn a_cell_is_the_verdict_and_its_reach_in_one_tap() {
    let offered = choices(&corpus()[0]);
    assert_eq!(
        offered,
        [
            Choice::Seen,
            Choice::Answer(Verdict::Pass, Scope::Call),
            Choice::Answer(Verdict::Pass, Scope::Conversation),
            Choice::Answer(Verdict::Pass, Scope::Workspace),
            Choice::Answer(Verdict::Refuse, Scope::Call),
            Choice::Answer(Verdict::Hold, Scope::Call),
        ]
    );
    let ops: Vec<String> = offered.iter().map(Choice::op).collect();
    assert_eq!(
        ops,
        ["seen", "answer", "answer", "answer", "answer", "answer"]
    );
}

#[test]
fn the_box_is_the_focused_conversations_own_row_by_both_keys() {
    let rows = corpus();
    assert_eq!(asking(&rows, "ws", "c-1").unwrap().agent, "c-1");
    assert!(asking(&rows, "other", "c-1").is_none());
    assert!(asking(&rows, "ws", "c-9").is_none());
}

#[test]
fn the_parked_call_is_folded_at_the_engines_clause() {
    let folded = parked(&corpus()[0]).unwrap();
    assert_eq!(folded.line, "held: Bash");
    assert_eq!(folded.why, "writes");
}

#[test]
fn the_words_are_the_flag_the_reason_the_failure_and_the_last_words() {
    let mut rows = corpus();
    let said = words(&rows[0]);
    assert_eq!(said.len(), 4);
    assert_eq!(said[0].0, "flagged · please look at this one");
    assert_eq!(said[0].1, theme::accent(State::Annotation));
    assert!(said[1].0.starts_with("parked a tool invocation"));
    assert_eq!(said[1].1, theme::INK_WEAK);
    assert_eq!(
        said[2],
        (
            "failed · Unauthorized".to_owned(),
            theme::accent(State::Error)
        )
    );
    assert_eq!(said[3], ("p".to_owned(), theme::INK));
    // A row whose engine wrote no sentence says its tokens.
    rows[0].says.clear();
    assert_eq!(words(&rows[0])[1].0, "held · mail · flagged");
    assert!(words(&rows[1]).is_empty());
}
