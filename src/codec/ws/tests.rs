//! The workspace row, both directions of the strictness contract: the
//! server's full spelling reads back, absence of the optionals is a fact, and
//! every malformed shape refuses by name.

use super::{ConfigTip, WsKind, row};
use serde_json::json;

#[test]
fn full_row_reads_back() {
    let v = json!({
        "workspace": "home", "kind": "named", "attention": 2, "agents": 5,
        "running": true, "pinned": 0,
        "config_tip": { "oid": "notreal-full", "short_oid": "notreal" },
    });
    let r = row(&v).unwrap();
    assert_eq!(r.workspace, "home");
    assert_eq!(r.kind, WsKind::Named);
    assert_eq!((r.attention, r.agents, r.running), (2, 5, true));
    assert_eq!(r.pinned, Some(0));
    assert_eq!(
        r.config_tip,
        Some(ConfigTip {
            oid: "notreal-full".into(),
            short_oid: "notreal".into()
        })
    );
}

#[test]
fn absent_optionals_are_facts() {
    let v = json!({
        "workspace": "w", "kind": "foreign", "attention": 0, "agents": 0,
        "running": false,
    });
    let r = row(&v).unwrap();
    assert_eq!(r.kind, WsKind::Foreign);
    assert_eq!(r.pinned, None);
    assert_eq!(r.config_tip, None);
}

#[test]
fn replay_kind_reads() {
    let v = json!({
        "workspace": "w", "kind": "replay", "attention": 0, "agents": 1,
        "running": false,
    });
    assert_eq!(row(&v).unwrap().kind, WsKind::Replay);
}

#[test]
fn refusals_name_the_offender() {
    assert_eq!(row(&json!(3)).unwrap_err(), "workspace row: not an object");
    let bad_kind = json!({
        "workspace": "w", "kind": "weird", "attention": 0, "agents": 0,
        "running": false,
    });
    assert_eq!(
        row(&bad_kind).unwrap_err(),
        "workspace row: unknown kind \"weird\""
    );
    let bad_tip = json!({
        "workspace": "w", "kind": "named", "attention": 0, "agents": 0,
        "running": false, "config_tip": 7,
    });
    assert_eq!(row(&bad_tip).unwrap_err(), "config_tip: not an object");
}
