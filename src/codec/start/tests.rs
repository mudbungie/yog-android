//! The start family's pins: the exact envelopes the server's decoder reads,
//! and the prepared body read back whole. A drifted spelling here is a
//! conversation that cannot be started, so the bytes are asserted rather
//! than the shape.

use super::{Prepared, encode_prepare, encode_prompt, prepared_of, reply_of};
use serde_json::json;

fn staged() -> Prepared {
    Prepared {
        workspace: "home".into(),
        binding: None,
        lineage: None,
        goal: "look into the flake".into(),
        origin: "conversation".into(),
    }
}

#[test]
fn staging_spells_the_bare_rung() {
    assert_eq!(
        encode_prepare("home"),
        json!({ "op": "prepare", "workspace": "home",
                "payload": { "rung": "bare" } })
    );
}

#[test]
fn firing_carries_the_body_whole_with_real_nulls() {
    let v = encode_prompt(&staged(), "look into the flake");
    assert_eq!(
        v,
        json!({ "op": "prompt",
                "prepared": { "workspace": "home", "binding": null,
                              "lineage": null, "goal": "look into the flake",
                              "origin": "conversation" },
                "goal": "look into the flake", "seed": null })
    );
    // The absences are FIELDS whose value is null, not omissions: the server
    // reads the field, and a body missing it is a body it refuses.
    let prepared = &v["prepared"];
    assert!(prepared.get("binding").is_some());
    assert!(prepared.get("lineage").is_some());
    assert!(v.get("seed").is_some());
}

#[test]
fn a_stated_binding_and_lineage_ride_through_untouched() {
    let rich = Prepared {
        binding: Some("/w/x".into()),
        lineage: Some("some-lineage".into()),
        ..staged()
    };
    let v = encode_prompt(&rich, "go");
    assert_eq!(v["prepared"]["binding"], "/w/x");
    assert_eq!(v["prepared"]["lineage"], "some-lineage");
    // Whole means whole: what came off the wire goes back on it unchanged.
    assert_eq!(prepared_of(&v["prepared"]).unwrap(), rich);
}

#[test]
fn a_prepared_reply_reads_back() {
    let envelope = json!({ "ok": true, "kind": "prepared",
                           "prepared": { "workspace": "home", "binding": null,
                                         "lineage": null, "goal": "g",
                                         "origin": "world" } });
    let o = envelope.as_object().unwrap();
    let read = reply_of(o).unwrap();
    assert_eq!(read.workspace, "home");
    assert_eq!(read.origin, "world");
    assert_eq!((read.binding, read.lineage), (None, None));
}

#[test]
fn a_malformed_prepared_body_refuses_by_name() {
    assert_eq!(
        prepared_of(&json!("x")).unwrap_err(),
        "prepared: not an object"
    );
    assert_eq!(
        prepared_of(&json!({ "binding": null, "lineage": null,
                             "goal": "g", "origin": "world" }))
        .unwrap_err(),
        "missing or non-string field \"workspace\""
    );
    let empty = json!({}).as_object().cloned().unwrap_or_default();
    assert_eq!(
        reply_of(&empty).unwrap_err(),
        "prepared: missing field \"prepared\""
    );
}

#[test]
fn an_origin_token_this_client_cannot_spell_still_rides_through() {
    // The token is carried as the word the engine wrote: this client never
    // branches on it, and refusing an unknown one would refuse a staging it
    // has no quarrel with.
    let v = json!({ "workspace": "home", "binding": null, "lineage": null,
                    "goal": "g", "origin": "something-new" });
    assert_eq!(prepared_of(&v).unwrap().origin, "something-new");
}
