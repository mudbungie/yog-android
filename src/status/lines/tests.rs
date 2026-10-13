//! One line where both halves fit, two where they do not, and what the second
//! line keeps.

use super::*;
use crate::theme::TOUCH;

const ALL: [Slot; 9] = [
    Slot::Workspace,
    Slot::Provider,
    Slot::Model,
    Slot::Effort,
    Slot::Priority,
    Slot::State,
    Slot::Tokens,
    Slot::Context,
    Slot::Name,
];

/// Roughly what each segment wants at the rule's small type.
const WANTS: [f32; 9] = [0.0, 0.0, 0.0, 80.0, 70.0, 90.0, 100.0, 80.0, 80.0];

fn placed(wide: f32) -> Vec<Place> {
    // Less the pass's gutters; the rule also less the mark and its gap.
    let under = wide - 20.0;
    layout(&ALL, &WANTS, under - TOUCH - 4.0, under, 4.0, None)
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
    assert_eq!(on(&places, true), [State, Tokens, Context, Name]);
    // The second line fits what it was given.
    let gaps = 4.0 * 3.0;
    let sum: f32 = places
        .iter()
        .map(|p| if let Place::Under(w) = p { *w } else { 0.0 })
        .sum();
    assert!(sum + gaps <= 411.0 - 20.0 + 0.01);
}

#[test]
fn a_narrow_second_line_lets_the_context_then_the_tokens_go_and_keeps_state_and_name() {
    use Slot::*;
    // 90 + 100 + 80 + 80 + gaps is 362: at 320 the context goes.
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
