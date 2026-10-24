//! The rule's facts, its drop order at three phone widths, its banner line
//! and its reload mark.

use super::*;
use crate::codec::{AgentState, Tone};
use serde_json::json;

const ALL: [Slot; 8] = [
    Slot::Workspace,
    Slot::Provider,
    Slot::Model,
    Slot::Effort,
    Slot::Priority,
    Slot::Tokens,
    Slot::State,
    Slot::Name,
];

/// What each segment would like at the rule's type size, roughly: the
/// selectors' are ignored (they share), the rest are their words.
const WANTS: [f32; 8] = [0.0, 0.0, 0.0, 80.0, 70.0, 100.0, 110.0, 80.0];

/// The room the segments have on a phone `wide` points across: less the
/// pass's 10-point gutters and the mark's touch square and its gap.
fn room(wide: f32) -> f32 {
    wide - 20.0 - TOUCH - 4.0
}

fn standing(wide: f32) -> Vec<Slot> {
    fit(&ALL, &WANTS, room(wide), 4.0, None)
        .iter()
        .zip(ALL)
        .filter_map(|(w, slot)| w.map(|_| slot))
        .collect()
}

#[test]
fn the_rule_drops_the_diagnostics_then_priority_then_effort_and_never_the_name() {
    use Slot::*;
    assert_eq!(standing(320.0), [Workspace, Provider, Model, Name]);
    assert_eq!(standing(360.0), [Workspace, Provider, Model, Name]);
    assert_eq!(standing(420.0), [Workspace, Provider, Model, Effort, Name]);
    assert_eq!(standing(480.0), [Workspace, Provider, Model, Effort, Name]);
    assert_eq!(
        standing(560.0),
        [Workspace, Provider, Model, Effort, Priority, Name]
    );
    assert_eq!(standing(800.0), ALL);
    // Narrower than the three selectors' floor, nothing else stands and the
    // three and the name still do — they elide; they never drop.
    assert_eq!(standing(100.0), [Workspace, Provider, Model, Name]);
}

#[test]
fn a_knob_whose_list_is_open_does_not_drop() {
    let shown = fit(&ALL, &WANTS, room(320.0), 4.0, Some(Slot::Priority));
    let standing: Vec<Slot> = shown
        .iter()
        .zip(ALL)
        .filter_map(|(w, slot)| w.map(|_| slot))
        .collect();
    assert_eq!(
        standing,
        [
            Slot::Workspace,
            Slot::Provider,
            Slot::Model,
            Slot::Priority,
            Slot::Name
        ]
    );
}

/// **The selectors are water-filled** (§23 item 14, the desktop's rule): a
/// short one keeps its words, the long ones share one cut of what is left,
/// and the others stand at their words.
#[test]
fn a_short_selector_keeps_its_words_and_the_long_ones_share_the_rest() {
    use Slot::*;
    let slots = [Workspace, Provider, Model, Name];
    let widths = fit(&slots, &[50.0, 200.0, 200.0, 60.0], 300.0, 4.0, None);
    assert_eq!(widths, [Some(50.0), Some(89.0), Some(89.0), Some(60.0)]);
    // Where they all fit, each is its words and the rest of the line is air.
    let widths = fit(&slots, &[50.0, 70.0, 90.0, 60.0], 600.0, 4.0, None);
    assert_eq!(widths, [Some(50.0), Some(70.0), Some(90.0), Some(60.0)]);
}

#[test]
fn what_stands_fits_and_every_segment_is_a_touch_target() {
    for wide in [320.0, 360.0, 420.0, 480.0, 800.0] {
        let widths: Vec<f32> = fit(&ALL, &WANTS, room(wide), 4.0, None)
            .into_iter()
            .flatten()
            .collect();
        let gaps = 4.0 * (widths.len() - 1) as f32;
        assert!(
            widths.iter().sum::<f32>() + gaps <= room(wide) + 0.01,
            "{wide}"
        );
        assert!(widths.iter().all(|w| *w >= TOUCH), "{wide}: {widths:?}");
    }
}

fn row() -> ConvRow {
    ConvRow {
        root_id: "c1".into(),
        display: "fixer".into(),
        name: None,
        display_only: false,
        state: AgentState::InFlight,
        uncertain: false,
        preview: String::new(),
        age_secs: 0,
        last_active_unix: 100,
        flight: None,
        attention: 0,
        members: 1,
        direct: 0,
        stoppable: false,
        stop_children: false,
        depth: 0,
        tone: Tone::InFlight,
        failure: None,
        alignment: None,
        ball: None,
    }
}

fn turn(usage: serde_json::Value) -> Entry {
    Entry {
        name: "001.json".into(),
        kind: EntryKind::Model {
            model_id: "opus".into(),
            blocks: Vec::new(),
            usage,
        },
        raw: String::new(),
    }
}

/// The conversation's own `agent` row, saying what it spent and how full
/// its context is — or, with neither, a workspace with no price table and a
/// context nothing measured.
fn head(usd: &str, percent: Option<i64>) -> Agent {
    Agent {
        display: "fixer".into(),
        root: "c1".into(),
        state: "in-flight".into(),
        flight: String::new(),
        present: true,
        refused: false,
        failure: None,
        marks: Vec::new(),
        tip: String::new(),
        strip: None,
        seats: Vec::new(),
        usd: usd.into(),
        context: percent.map(|percent| crate::codec::Context {
            model: "opus".into(),
            percent,
        }),
    }
}

#[test]
fn the_diagnostics_are_this_conversations_alone() {
    assert!(diagnostics(None, &[], Some(&head("$4.00", Some(40))), 0).is_empty());
    let said = diagnostics(Some(&row()), &[], None, 160);
    let texts: Vec<(Slot, &str)> = said.iter().map(|s| (s.slot, s.text.as_str())).collect();
    assert_eq!(
        texts,
        [(Slot::State, "in-flight · 1m"), (Slot::Name, "fixer")]
    );
    assert_eq!(said[0].ink, theme::tone(&Tone::InFlight));
    let turns = [
        turn(json!({ "input_tokens": 1200, "output_tokens": 30 })),
        turn(json!({ "input_tokens": 5 })),
    ];
    let held = head("$4.00", Some(140));
    let said = diagnostics(Some(&row()), &turns, Some(&held), 160);
    let texts: Vec<(Slot, &str)> = said.iter().map(|s| (s.slot, s.text.as_str())).collect();
    // The state first, then the tokens, then the spend in the ledger's own
    // words, then the context — the server's own percent, unclamped — then
    // the name.
    assert_eq!(
        texts,
        [
            (Slot::State, "in-flight · 1m"),
            (Slot::Tokens, "1.2k in · 30 out"),
            (Slot::Spend, "$4.00"),
            (Slot::Context, "context 140%"),
            (Slot::Name, "fixer")
        ]
    );
    // A row that states no price and no fullness paints neither: never `$0`,
    // never `0%`.
    let said = diagnostics(Some(&row()), &[], Some(&head("", None)), 160);
    let slots: Vec<Slot> = said.iter().map(|s| s.slot).collect();
    assert_eq!(slots, [Slot::State, Slot::Name]);
}

#[test]
fn tokens_are_summed_where_spelled_and_absent_where_never() {
    assert_eq!(tokens(&[turn(json!({}))]), None);
    assert_eq!(tokens(&[]), None);
    assert_eq!(tokens(&[turn(json!({ "output_tokens": 7 }))]), Some((0, 7)));
    assert_eq!(count(999), "999");
    assert_eq!(count(4_100_000), "4.1M");
}

#[test]
fn the_banner_says_the_operators_own_act_first_and_stays_down_once_put_down() {
    assert_eq!(banner(None, None, false, None), None);
    assert_eq!(
        banner(Some("/flag needs a reason"), Some("refused"), true, None),
        Some(("/flag needs a reason".into(), State::Annotation))
    );
    assert_eq!(
        banner(None, Some("refused"), true, None),
        Some(("refused".into(), State::Error))
    );
    assert_eq!(
        banner(None, Some("refused"), true, Some("refused")),
        Some((RECONNECTING.into(), State::Working))
    );
    assert_eq!(banner(None, None, true, Some(RECONNECTING)), None);
    // A new failure is a new sentence, and stands.
    assert!(banner(None, Some("other"), false, Some("refused")).is_some());
}

#[test]
fn a_list_is_fresh_once_the_watermark_moves() {
    assert_eq!(reload(3, 3), Reload::Reloading);
    assert_eq!(reload(3, 4), Reload::Fresh);
}
