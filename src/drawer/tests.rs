//! How wide, when it stands, which gestures move it, what a row says.

use super::*;
use crate::codec::{Tone, WsKind};

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

fn conv(state: AgentState, tone: Tone) -> ConvRow {
    ConvRow {
        root_id: "c1".into(),
        display: "fixer".into(),
        name: None,
        display_only: false,
        state,
        uncertain: false,
        preview: "first line\nsecond".into(),
        age_secs: 0,
        last_active_unix: 1_700_000_000,
        flight: None,
        attention: 0,
        members: 1,
        direct: 0,
        stoppable: false,
        stop_children: false,
        depth: 0,
        tone,
        failure: None,
        alignment: None,
        ball: None,
    }
}

#[test]
fn a_drag_from_the_edge_that_reaches_opens_and_nothing_else_does() {
    assert!(opens(Some(10.0), Some(80.0)));
    assert!(opens(Some(EDGE), Some(EDGE + REACH)));
    assert!(!opens(Some(30.0), Some(200.0)), "not from the edge");
    assert!(!opens(Some(10.0), Some(60.0)), "not far enough");
    assert!(!opens(Some(10.0), None));
    assert!(!opens(None, Some(80.0)));
}

#[test]
fn a_drag_from_inside_the_drawer_that_travels_left_past_reach_closes() {
    let wide = width(411.0);
    assert!(closes(Some(300.0), Some(300.0 - REACH), wide));
    assert!(closes(Some(wide), Some(0.0), wide));
    assert!(!closes(Some(wide + 1.0), Some(0.0), wide), "from the scrim");
    assert!(!closes(Some(300.0), Some(250.0), wide), "not far enough");
    assert!(!closes(Some(100.0), Some(300.0), wide), "the wrong way");
    assert!(!closes(None, Some(0.0), wide) && !closes(Some(0.0), None, wide));
}

/// Five sixths of the glass, capped at 360: a phone's whole width less a
/// strip of the chat, a tablet's list rather than its half.
#[test]
fn the_width_is_five_sixths_of_the_glass_and_never_more_than_the_cap() {
    assert!((width(360.0) - 300.0).abs() < 0.01);
    assert!((width(411.0) - 342.5).abs() < 0.01);
    assert!((width(1200.0) - CAP).abs() < f32::EPSILON);
    assert!(width(-5.0).abs() < f32::EPSILON);
    assert!((EASE - 0.12).abs() < f32::EPSILON);
}

#[test]
fn it_stands_when_opened_and_always_while_nothing_is_focused() {
    assert!(stands(true, true));
    assert!(stands(false, false));
    assert!(stands(true, false));
    assert!(!stands(false, true));
}

#[test]
fn the_head_says_the_workspace_or_the_word() {
    assert_eq!(head(Some("main")), "main");
    assert_eq!(head(None), WORKSPACE);
}

#[test]
fn the_waiting_count_is_summed_over_every_workspace() {
    assert_eq!(waiting(&[ws("a", 2), ws("b", 0), ws("c", 3)]), 5);
    assert_eq!(waiting(&[]), 0);
}

#[test]
fn the_state_glyph_is_the_desktops() {
    assert_eq!(mark(&AgentState::Live), glyph::WORKING);
    assert_eq!(mark(&AgentState::InFlight), glyph::WORKING);
    assert_eq!(mark(&AgentState::Quiescent), glyph::AT_REST);
    assert_eq!(mark(&AgentState::Stopped), glyph::STOPPED);
    assert_eq!(mark(&AgentState::Unknown("odd".into())), glyph::NOTE);
}

#[test]
fn a_row_reads_as_the_desktop_bars_open_row() {
    let row = conv(AgentState::InFlight, Tone::InFlight);
    let said = words(&row, false, 1_700_003_600);
    assert_eq!(
        said,
        Words {
            mark: (glyph::WORKING.to_owned(), theme::tone(&Tone::InFlight)),
            asking: false,
            name: ("fixer".into(), theme::INK),
            beneath: Some("first line second".into()),
            age: "1h".into(),
        }
    );
    // The focused one is in the brand, and an asking row says so.
    let mut row = conv(AgentState::Quiescent, Tone::Plain);
    row.attention = 1;
    row.preview = String::new();
    let said = words(&row, true, 1_700_000_000);
    assert_eq!(said.name, ("fixer".into(), theme::BRAND));
    assert!(said.asking);
    assert_eq!(said.beneath, None);
    assert_eq!(said.age, "now");
    assert_eq!(asking_ink(), theme::accent(State::Attention));
    // The failure outranks the preview beneath, folded to one line.
    row.failure = Some("the provider\nrefused".into());
    assert_eq!(
        words(&row, false, 0).beneath.as_deref(),
        Some("failed · the provider refused")
    );
}
