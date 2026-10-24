//! Which rows the dock holds, in which order, and what it says of them.

use super::*;
use crate::codec::Tone;

fn row(id: &str, state: AgentState, depth: usize, last: i64) -> ConvRow {
    ConvRow {
        root_id: id.into(),
        display: id.into(),
        name: None,
        display_only: false,
        state,
        uncertain: false,
        preview: String::new(),
        age_secs: 0,
        last_active_unix: last,
        flight: None,
        attention: 0,
        members: 1,
        direct: 0,
        stoppable: false,
        stop_children: false,
        depth,
        tone: Tone::Live,
        failure: None,
        alignment: None,
        ball: None,
    }
}

fn ids(rows: &[ConvRow]) -> Vec<&str> {
    rows.iter().map(|r| r.root_id.as_str()).collect()
}

#[test]
fn only_live_and_in_flight_rows_run_in_the_switchers_order() {
    let rows = [
        row("old", AgentState::Live, 0, 10),
        row("old-kid", AgentState::InFlight, 1, 10),
        row("quiet", AgentState::Quiescent, 0, 50),
        row("stopped", AgentState::Stopped, 0, 60),
        row("odd", AgentState::Unknown("parked".into()), 0, 70),
        row("new", AgentState::InFlight, 0, 40),
    ];
    // Newest subtree first, each in the server's descent.
    assert_eq!(ids(&running(&rows)), ["new", "old", "old-kid"]);
    assert!(running(&[row("q", AgentState::Quiescent, 0, 1)]).is_empty());
}

#[test]
fn the_line_counts_and_names_the_first_and_says_its_fold() {
    let rows = [
        row("brave-fox", AgentState::InFlight, 0, 1),
        row("moss", AgentState::Live, 0, 1),
    ];
    assert_eq!(
        line(&rows, false).as_deref(),
        Some("+ 2 running · brave-fox")
    );
    assert_eq!(
        line(&rows, true).as_deref(),
        Some("− 2 running · brave-fox")
    );
    // Nothing running is no dock at all.
    assert_eq!(line(&[], false), None);
}

#[test]
fn a_row_says_its_name_its_state_in_its_ink_and_its_age() {
    let said = row_words(&row("fox", AgentState::InFlight, 0, 100), 100 + 3 * 60);
    let text: Vec<&str> = said.iter().map(|(t, _)| t.as_str()).collect();
    assert_eq!(
        text,
        [
            "fox",
            "  in-flight",
            &format!("  {}", crate::roster::stamp(100, 280))
        ]
    );
    assert_eq!(said[0].1, theme::INK);
    assert_eq!(said[1].1, theme::tone(&Tone::Live));
    assert_eq!(said[2].1, theme::INK_WEAK);
}
