//! The ball pane's three shapes, read strictly — and the two absences that are
//! facts rather than zeros: a ball nobody holds, and a workspace whose spend
//! the engine rendered no dollars for.

use super::{Pane, View};
use serde_json::{Value, json};

fn object(v: &Value) -> serde_json::Map<String, Value> {
    v.as_object().cloned().unwrap_or_default()
}

#[test]
fn a_ball_row_carries_its_own_words_and_absence_paints_as_nothing() {
    let held = super::row(
        &json!({ "ball_id": "bl-1", "project": "p", "state": "delivered",
                                   "title": "t", "claimant": "alba", "workspace": "ws" }),
    )
    .unwrap();
    assert_eq!((held.id.as_str(), held.project.as_str()), ("bl-1", "p"));
    assert_eq!(held.claimant, "alba");
    assert_eq!(held.workspace, "ws");
    let free = super::row(&json!({ "ball_id": "bl-2", "project": "p", "state": "ready" })).unwrap();
    assert_eq!(free.claimant, "");
    assert_eq!(free.title, "");
}

#[test]
fn a_row_that_is_not_an_object_and_a_row_missing_its_id_both_refuse_by_name() {
    assert_eq!(
        super::row(&json!("bl-1")).unwrap_err(),
        "balls: row is not an object"
    );
    assert_eq!(
        super::row(&json!({ "project": "p", "state": "ready" })).unwrap_err(),
        "missing or non-string field \"ball_id\""
    );
}

/// **The money is the engine's rendering and its absence is a fact**: a spend
/// with no `usd` paints nothing, and nothing here multiplies the counters
/// beside it by a rate of its own.
#[test]
fn a_held_ball_carries_the_spend_the_engine_rendered_and_no_second_opinion() {
    let priced = super::bound(&json!({ "id": "bl-1", "project": "p", "state": "delivered",
                                       "owner": "alba", "badge": "delivered",
                                       "spend": { "usd": "$2.50", "micro_usd": 2_500_000 } }))
    .unwrap();
    assert_eq!(priced.usd, "$2.50");
    assert_eq!(priced.badge, "delivered");
    let unpriced = super::bound(&json!({ "id": "bl-2", "project": "p", "state": "bound",
                                         "owner": "alba", "spend": { "tokens": {} } }))
    .unwrap();
    assert_eq!(unpriced.usd, "");
    let none = super::bound(&json!({ "id": "bl-3", "project": "p", "state": "ready" })).unwrap();
    assert_eq!((none.usd.as_str(), none.owner.as_str()), ("", ""));
}

#[test]
fn a_board_carries_its_columns_its_drones_and_the_fleets_own_sentence() {
    let board = super::board(&object(&json!({
        "rows": [{ "id": "bl-1", "project": "p", "column": "gated", "priority": 2,
                   "claimant": "alba", "parent": "bl-epic",
                   "drones": [{ "name": "Cobalt", "root_id": "c-1" }],
                   "gates": [{ "id": "bl-gate", "mints": "close", "title": "g" }] },
                 { "id": "bl-2", "project": "p", "column": "ready", "priority": 0,
                   "title": "u", "state": "ready", "drones": [], "gates": [] }],
        "fleet": [{ "label": "1/4 drones · tick 1m", "workspace": "ws", "project": "p" }]
    })))
    .unwrap();
    assert_eq!(board.fleet, ["1/4 drones · tick 1m"]);
    let first = board.rows.first().unwrap();
    assert_eq!(first.column, "gated");
    assert_eq!(first.drones, ["Cobalt"]);
    assert_eq!(first.gates, ["bl-gate"]);
    assert_eq!(first.priority, 2);
    let second = board.rows.get(1).unwrap();
    assert_eq!(second.title, "u");
    assert!(second.drones.is_empty());
}

/// An absent `fleet` is *nothing is armed* rather than a refusal — the one
/// case where a `Vec` and an `Option<Vec>` are not two claims.
#[test]
fn a_board_with_nothing_armed_carries_no_fleet_and_is_not_a_refusal() {
    let board = super::board(&object(&json!({ "rows": [] }))).unwrap();
    assert!(board.fleet.is_empty() && board.rows.is_empty());
}

#[test]
fn a_malformed_board_refuses_by_name() {
    assert_eq!(
        super::board(&object(
            &json!({ "rows": [{ "id": "bl-1", "project": "p" }] })
        ))
        .unwrap_err(),
        "missing or non-string field \"column\""
    );
    assert_eq!(
        super::board(&object(&json!({ "rows": [], "fleet": ["a line"] }))).unwrap_err(),
        "board: row is not an object"
    );
    assert_eq!(
        super::board(&object(&json!({
            "rows": [{ "id": "bl-1", "project": "p", "column": "ready", "priority": 0,
                       "drones": [{ "root_id": "c-1" }] }]
        })))
        .unwrap_err(),
        "missing or non-string field \"name\""
    );
    assert_eq!(
        super::board(&object(&json!({ "rows": {} }))).unwrap_err(),
        "missing or non-array field \"rows\""
    );
}

/// **A pane says which read answered it**, which is what makes a view's
/// screen unpaintable under another's answer.
#[test]
fn a_pane_carries_the_view_that_answered_it_and_each_view_names_its_screen() {
    assert_eq!(Pane::Everywhere(Vec::new()).view(), View::Everywhere);
    assert_eq!(Pane::Here(Vec::new()).view(), View::Here);
    assert_eq!(
        Pane::Board(super::Board {
            rows: Vec::new(),
            fleet: Vec::new()
        })
        .view(),
        View::Board
    );
    assert_eq!(View::Everywhere.screen(), "balls");
    assert_eq!(View::Here.screen(), "workspace-balls");
    assert_eq!(View::Board.screen(), "board");
}
