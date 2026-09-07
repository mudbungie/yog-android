//! **The tool window's half of the lane** (REMOTE §5.5, PROTOCOL 15): which
//! flight paints which half, what the window adds to the record, and what it
//! must never paint twice.

use super::{
    RUNNING, calling, closed, delivered, kinds, opened, parked, settled, stream, tail, windowing,
};
use crate::codec::EntryKind;

/// **The prose is the streaming call's, not the step's** (REMOTE §5.5,
/// bl-ee21). The lane now stays open through the tool phase with the settled
/// call's words still in the fold, and the engine stops carrying a tail at
/// exactly that moment — so a fold painted on any flight would put the
/// committed answer on the glass a second time for as long as the commands
/// run.
#[test]
fn a_step_running_tools_paints_no_prose_tail() {
    let read = vec![delivered("001"), tail("", "what it said before")];
    let out = settled(read, Some(&stream("", "what it said before")), RUNNING);
    assert_eq!(kinds(&out), ["delivered"]);
}

/// The window is the step's, so it paints under any flight — including the
/// one that says nothing in prose, which is the case it exists for.
#[test]
fn the_window_paints_the_call_the_record_has_not_carried_yet() {
    let held = windowing(vec![opened("toolu_01", "box2_Bash")]);
    let out = settled(vec![delivered("001")], Some(&held), RUNNING);
    assert_eq!(kinds(&out), ["delivered", "window:box2_Bash:None"]);
    let payload = out.last().unwrap();
    assert_eq!(payload.name, "window/toolu_01", "keyed by the call");
    assert!(
        matches!(&payload.kind, EntryKind::Windowed { input, .. } if input.contains("uptime")),
        "the input the engine bounded rides through"
    );
}

/// Both transitions of one call are one row: the closing one restates nothing
/// and folds onto the opening one by id.
#[test]
fn the_closing_transition_lands_on_the_row_the_opening_one_made() {
    let held = windowing(vec![opened("toolu_01", "box2_Bash"), closed("toolu_01", 0)]);
    let out = settled(vec![delivered("001")], Some(&held), RUNNING);
    assert_eq!(kinds(&out), ["delivered", "window:box2_Bash:Some(0)"]);
}

/// **The record is the record.** A call a committed block already names is the
/// transcript's to paint, so the lane adds nothing for it — the same
/// structural dedupe the tail has, keyed on the id both sides share.
#[test]
fn a_call_the_record_carries_is_not_painted_twice() {
    let held = windowing(vec![opened("toolu_01", "box2_Bash")]);
    let read = vec![delivered("001"), calling("002", "toolu_01")];
    assert_eq!(
        kinds(&settled(read, Some(&held), RUNNING)),
        ["delivered", "model"]
    );
}

/// A call whose record carried no readable name is still said — the operator
/// is being told something is running on their machine, and the id is the
/// least that can say it.
#[test]
fn a_nameless_call_is_labelled_by_its_own_id() {
    let held = windowing(vec![closed("toolu_09", 2)]);
    let out = settled(vec![delivered("001")], Some(&held), RUNNING);
    assert_eq!(kinds(&out), ["delivered", "window:toolu_09:Some(2)"]);
}

/// **A held call lands neither of the two files** the window's pair of
/// transitions is made of, so before PROTOCOL 18 the lane carried nothing at
/// all for the one call the operator is the blocker on. It rides as an
/// ordinary windowed entry, and the control's sentence rides with it.
#[test]
fn a_held_call_is_a_window_row_carrying_the_controls_sentence() {
    let window = windowing(vec![parked("toolu_01", "box2_Bash", "classified loss")]);
    let out = settled(vec![delivered("001")], Some(&window), RUNNING);
    assert_eq!(kinds(&out), ["delivered", "window:box2_Bash:None"]);
    assert!(
        matches!(
            &out.last().unwrap().kind,
            EntryKind::Windowed { held: Some(why), .. } if why == "classified loss"
        ),
        "the control's own words cross unrewritten (REMOTE §8.1)"
    );
}

/// The hold is stated once, on the opening transition, and every later
/// transition of the same call folds onto the row it made — so a capture that
/// lands after the operator passed the call does not un-say what was held.
#[test]
fn a_hold_survives_a_later_transition_of_the_same_call() {
    let window = windowing(vec![
        parked("toolu_01", "box2_Bash", "classified loss"),
        closed("toolu_01", 0),
    ]);
    let out = settled(vec![delivered("001")], Some(&window), RUNNING);
    assert!(matches!(
        &out.last().unwrap().kind,
        EntryKind::Windowed {
            held: Some(_),
            exit_code: Some(0),
            ..
        }
    ));
}

/// At rest there is neither half, whatever the fold still holds.
#[test]
fn a_conversation_at_rest_paints_no_window() {
    let held = windowing(vec![opened("toolu_01", "box2_Bash")]);
    assert_eq!(
        kinds(&settled(vec![delivered("001")], Some(&held), None)),
        ["delivered"]
    );
}
