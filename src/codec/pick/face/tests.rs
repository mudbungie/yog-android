//! The three faces, and the one arm each of them has for *nothing is known*.

use super::{effort, model, priority};
use crate::codec::RoleRow;

fn set(provider: &str, model: &str, level: Option<&str>, priority: bool) -> RoleRow {
    RoleRow {
        role: "worker".to_owned(),
        provider: provider.to_owned(),
        model: model.to_owned(),
        effort: level.map(str::to_owned),
        priority,
    }
}

#[test]
fn the_model_face_is_the_workspaces_own_when_nobody_has_picked() {
    let row = set("anthropic", "opus", None, false);
    assert_eq!(model(None, Some(&row), Some("anthropic")), "opus");
}

/// The optimistic pick wins, because the operator just made it.
#[test]
fn a_pick_overtakes_the_assignment() {
    let row = set("anthropic", "opus", None, false);
    assert_eq!(
        model(Some("sonnet".to_owned()), Some(&row), Some("anthropic")),
        "sonnet"
    );
}

/// A model belongs to its provider: under another one the pair would not
/// exist, so the control wears its own name until a model is picked.
#[test]
fn the_assignment_is_not_shown_under_another_provider() {
    let row = set("anthropic", "opus", None, false);
    assert_eq!(model(None, Some(&row), Some("openai")), "model");
    assert_eq!(model(None, Some(&row), None), "model");
    assert_eq!(model(None, None, Some("anthropic")), "model");
}

#[test]
fn the_effort_face_carries_its_name_and_its_level() {
    let row = set("anthropic", "opus", Some("high"), false);
    assert_eq!(effort(None, Some(&row)), "effort: high");
    assert_eq!(effort(Some("low".to_owned()), Some(&row)), "effort: low");
}

/// An assignment with no level IS a value — `off`, the absence carried as a
/// real null — while no assignment at all is not, and the two must not read
/// alike.
#[test]
fn no_level_reads_off_and_no_assignment_reads_nothing() {
    let row = set("anthropic", "opus", None, false);
    assert_eq!(effort(None, Some(&row)), "effort: off");
    assert_eq!(effort(None, None), "effort");
}

#[test]
fn the_priority_face_says_which_way_it_is_set() {
    let on = set("anthropic", "opus", None, true);
    let off = set("anthropic", "opus", None, false);
    assert_eq!(priority(None, Some(&on)), (true, "priority: on".to_owned()));
    assert_eq!(
        priority(None, Some(&off)),
        (false, "priority: off".to_owned())
    );
    assert_eq!(
        priority(Some(true), Some(&off)),
        (true, "priority: on".to_owned())
    );
    assert_eq!(priority(None, None), (false, "priority".to_owned()));
}
