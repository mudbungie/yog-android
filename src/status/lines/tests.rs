//! One line where both halves fit, two where they do not, and what the second
//! line keeps.

use super::*;
use crate::status::SHARE;
use crate::theme::TOUCH;

const ALL: [Slot; 10] = [
    Slot::Workspace,
    Slot::Provider,
    Slot::Model,
    Slot::Effort,
    Slot::Priority,
    Slot::State,
    Slot::Tokens,
    Slot::Spend,
    Slot::Context,
    Slot::Name,
];

/// Roughly what each segment wants at the rule's small type.
const WANTS: [f32; 10] = [0.0, 0.0, 0.0, 80.0, 70.0, 90.0, 100.0, 50.0, 80.0, 80.0];

/// The gap a separator stands in: the item spacing either side of `|`.
const GAP: f32 = 11.0;

fn placed(wide: f32) -> Vec<Place> {
    // Less the pass's gutters; the rule also less the mark, the lead `—`
    // and the spacing either side of it.
    let under = wide - 20.0;
    layout(&ALL, &WANTS, under - TOUCH - 20.0, under, GAP, None)
}

fn on(places: &[Place], under: bool) -> Vec<Slot> {
    places
        .iter()
        .zip(ALL)
        .filter(|(p, _)| match p {
            Place::Rule(_) => !under,
            Place::Under(_) => under,
            Place::Off => false,
        })
        .map(|(_, slot)| slot)
        .collect()
}

/// **The walk's own figures** (bl-9817, re-read for the doors of §23 item
/// 14): a 411-point glass, the rule's spacing either side of a `|`, and the
/// doors' wants read off the bundled faces at `SMALL` — the air inside the
/// door, the words and ` ⏶`: a short workspace, `anthropic`, a long model
/// id, `effort high`, and `priority on` with no mark (a flip opens no list).
#[test]
fn at_411_points_effort_stands_and_a_short_selector_keeps_its_words() {
    use Slot::*;
    let wants = [47.0, 77.0, 150.0, 82.3, 58.0, 90.0, 100.0, 50.0, 80.0, 80.0];
    let under = 411.0 - 20.0;
    let gap = 4.0 * 2.0 + 3.2;
    let room = under - TOUCH - 4.0 - 11.95 - 4.0;
    let places = layout(&ALL, &wants, room, under, gap, None);
    assert_eq!(on(&places, false), [Workspace, Provider, Model, Effort]);
    // The workspace is a touch target and its words; the provider keeps its
    // words; the model takes the rest, and that is more than the share.
    assert_eq!(places[0], Place::Rule(TOUCH));
    assert_eq!(places[1], Place::Rule(77.0));
    assert!(
        matches!(places[2], Place::Rule(w) if w >= SHARE),
        "{:?}",
        places[2]
    );
}

#[test]
fn a_wide_rule_holds_both_halves_on_one_line() {
    let places = placed(1200.0);
    assert_eq!(on(&places, false), ALL);
    assert!(on(&places, true).is_empty());
}

#[test]
fn at_the_walks_width_the_diagnostics_stand_under_the_selectors() {
    use Slot::*;
    let places = placed(411.0);
    assert_eq!(on(&places, false), [Workspace, Provider, Model, Effort]);
    // The spend is the first to leave the second line (bl-a7ec)…
    assert_eq!(on(&places, true), [State, Tokens, Context, Name]);
    // …and stands where the line holds all five.
    assert_eq!(
        on(&placed(470.0), true),
        [State, Tokens, Spend, Context, Name]
    );
    // The second line fits what it was given.
    let gaps = GAP * 3.0;
    let sum: f32 = places
        .iter()
        .map(|p| if let Place::Under(w) = p { *w } else { 0.0 })
        .sum();
    assert!(sum + gaps <= 411.0 - 20.0 + 0.01);
}

#[test]
fn a_narrow_second_line_lets_the_spend_the_context_then_the_tokens_go_and_keeps_state_and_name() {
    use Slot::*;
    // Less the spend, 90 + 100 + 80 + 80 + gaps is 362: at 320 the context goes.
    assert_eq!(on(&placed(340.0), true), [State, Tokens, Name]);
    // At 240 the tokens go too; the state and the name elide and stay.
    assert_eq!(on(&placed(240.0), true), [State, Name]);
    assert_eq!(on(&placed(120.0), true), [State, Name]);
}

#[test]
fn nothing_focused_is_the_selectors_alone() {
    let slots = [Slot::Workspace, Slot::Provider, Slot::Model, Slot::Effort];
    let places = layout(&slots, &[0.0, 0.0, 0.0, 80.0], 400.0, 300.0, 4.0, None);
    assert!(places.iter().all(|p| matches!(p, Place::Rule(_))));
    // Too narrow even for effort: it leaves, and no second line stands.
    let places = layout(&slots, &[0.0, 0.0, 0.0, 80.0], 160.0, 300.0, 4.0, None);
    assert_eq!(places[3], Place::Off);
    assert!(!places.iter().any(|p| matches!(p, Place::Under(_))));
}
