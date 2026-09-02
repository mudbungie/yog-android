//! The cache both ways, and every way it refuses. The fixtures are the
//! ENGINE's own envelopes (the shapes `seat::tests` scripts its server with),
//! because those are what the file holds.

use super::{Envelopes, VERSION, read, write};
use crate::seat::Focus;
use crate::test_support::scratch;
use serde_json::{Value, json};

fn ws() -> Value {
    json!({ "ok": true, "kind": "workspaces",
            "rows": [{ "workspace": "home", "kind": "named", "attention": 0,
                       "agents": 1, "running": false }] })
}

fn convs() -> Value {
    json!({ "ok": true, "kind": "conversations",
            "rows": [{ "root_id": "a1", "display": "d", "display_only": false,
                       "state": "quiescent", "uncertain": false, "preview": "",
                       "age_secs": 0, "attention": 0, "members": 1, "direct": 0,
                       "stoppable": false, "stop_children": false, "depth": 0,
                       "tone": "plain" }] })
}

fn transcript() -> Value {
    json!({ "ok": true, "kind": "transcript",
            "rows": [{ "name": "001", "raw": "", "kind": "raw" }] })
}

fn deep() -> Focus {
    Focus {
        workspace: Some("home".to_owned()),
        agent: Some("a1".to_owned()),
    }
}

fn all() -> Envelopes {
    Envelopes {
        workspaces: Some(ws()),
        conversations: Some(convs()),
        transcript: Some(transcript()),
        ..Envelopes::default()
    }
}

/// The same, with the selectors' offerings for the focused workspace
/// (bl-0267).
fn with_options() -> Envelopes {
    Envelopes {
        options_workspace: Some("home".to_owned()),
        providers: Some(json!({ "kind": "providers", "ok": true,
                                "rows": [{ "name": "acme", "fact": "credential present",
                                           "blocked": null }] })),
        models: [(
            "acme".to_owned(),
            json!({ "kind": "models", "ok": true, "rows": ["opus"] }),
        )]
        .into_iter()
        .collect(),
        ..all()
    }
}

fn path() -> std::path::PathBuf {
    scratch().join("cache").join("seat.json")
}

/// The round trip, at the deepest focus: what went in comes back decoded, the
/// focus with it, and the live `error` field is not a thing a cache carries.
#[test]
fn a_stored_pass_reads_back_as_the_snapshot_it_was() {
    let at = path();
    write(&at, &deep(), &all()).unwrap();
    let (focus, snap, _) = read(&at).unwrap();
    assert_eq!(focus, deep());
    assert_eq!(snap.focus, deep());
    assert_eq!(snap.workspaces[0].workspace, "home");
    assert_eq!(snap.conversations[0].root_id, "a1");
    assert_eq!(snap.transcript[0].name, "001");
    assert_eq!(snap.error, None);
}

/// A shallow focus stores one envelope and reads back with the two deeper
/// lists empty — the depth is the focus's, not a second field.
#[test]
fn a_shallow_pass_stores_and_reads_only_what_it_asked() {
    let at = path();
    let kept = Envelopes {
        workspaces: Some(ws()),
        ..Envelopes::default()
    };
    write(&at, &Focus::default(), &kept).unwrap();
    let (focus, snap, _) = read(&at).unwrap();
    assert_eq!(focus, Focus::default());
    assert!(snap.conversations.is_empty() && snap.transcript.is_empty());
    assert_eq!(snap.workspaces.len(), 1);
}

/// Every way it refuses, and they all answer the same: paint nothing.
#[test]
fn every_doubt_discards_the_whole_file() {
    let at = path();
    // Absent.
    assert!(read(&at).is_none());
    // Not JSON at all.
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(&at, b"not json").unwrap();
    assert!(read(&at).is_none());
    // A JSON file that is not this file: no tag.
    std::fs::write(&at, json!({ "ok": true }).to_string()).unwrap();
    assert!(read(&at).is_none());
    let with = |body: Value| {
        std::fs::write(&at, body.to_string()).unwrap();
        read(&at)
    };
    let good = json!({
        super::TAG: VERSION, "protocol": crate::hello::PROTOCOL,
        "focus": { "workspace": null, "agent": null },
        "workspaces": ws(), "conversations": null, "transcript": null });
    assert!(with(good.clone()).is_some(), "the control must read");
    // Another layout version, and another protocol.
    let mut other = good.clone();
    other[super::TAG] = json!(VERSION + 1);
    assert!(with(other).is_none());
    let mut skewed = good.clone();
    skewed["protocol"] = json!(u64::from(crate::hello::PROTOCOL) + 1);
    assert!(with(skewed).is_none());
    // An envelope this decoder refuses, and one of the wrong kind.
    let mut broken = good.clone();
    broken["workspaces"] = json!({ "ok": true, "kind": "workspaces" });
    assert!(with(broken).is_none());
    let mut wrong = good.clone();
    wrong["workspaces"] = transcript();
    assert!(with(wrong).is_none());
    // No roster at all: a pass that answered nothing was never stored.
    let mut bare = good.clone();
    bare["workspaces"] = json!(null);
    assert!(with(bare).is_none());
}

/// **The pairing law, checked on the file** (the `Snapshot` invariant): rows
/// deeper than the focus they were asked at are not paintable, in either
/// direction — a depth without its focus, and a focus without its depth.
#[test]
fn a_depth_that_disagrees_with_its_focus_is_refused() {
    let at = path();
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    let with = |body: Value| {
        std::fs::write(&at, body.to_string()).unwrap();
        read(&at)
    };
    let mut orphan = json!({
        super::TAG: VERSION, "protocol": crate::hello::PROTOCOL,
        "focus": { "workspace": null, "agent": null },
        "workspaces": ws(), "conversations": convs(), "transcript": null });
    assert!(with(orphan.clone()).is_none());
    // …and a focused workspace whose conversations are missing.
    orphan["conversations"] = json!(null);
    orphan["focus"]["workspace"] = json!("home");
    assert!(with(orphan.clone()).is_none());
    // The transcript half of the same law.
    orphan["conversations"] = convs();
    orphan["transcript"] = transcript();
    assert!(with(orphan).is_none());
}

/// A wrong kind under a deeper key discards too — the check is per envelope,
/// not only on the roster.
#[test]
fn a_deep_envelope_of_the_wrong_kind_discards() {
    let at = path();
    let mut kept = all();
    kept.conversations = Some(transcript());
    write(&at, &deep(), &kept).unwrap();
    assert!(read(&at).is_none());
    let mut kept = all();
    kept.transcript = Some(convs());
    write(&at, &deep(), &kept).unwrap();
    assert!(read(&at).is_none());
}

/// A write that cannot land says so, and says where. Both halves: a parent
/// that is a FILE (no directory can be made there), and a path that is a
/// directory (nothing can be written over it).
#[test]
fn a_write_that_cannot_land_names_the_path() {
    let dir = scratch();
    let blocked = dir.join("wall");
    std::fs::write(&blocked, b"in the way").unwrap();
    let under = blocked.join("seat.json");
    assert!(write(&under, &Focus::default(), &all()).is_err());
    let occupied = dir.join("occupied");
    std::fs::create_dir_all(&occupied).unwrap();
    assert!(write(&occupied, &Focus::default(), &all()).is_err());
}

/// **The selectors' offerings round-trip with the rows** (bl-0267), and the
/// same pairing law covers them: options naming a workspace the focus does
/// not is the mispairing the whole file is fail-closed about.
#[test]
fn stored_options_read_back_and_a_foreign_workspace_discards() {
    let at = path();
    write(&at, &deep(), &with_options()).unwrap();
    let (_, _, kept) = read(&at).unwrap();
    assert_eq!(kept.options_workspace.as_deref(), Some("home"));
    assert!(kept.providers.is_some());
    assert_eq!(kept.models.len(), 1);

    // A file with no options at all is ordinary: the selectors may simply
    // never have been opened.
    write(&at, &deep(), &all()).unwrap();
    let (_, _, kept) = read(&at).unwrap();
    assert_eq!(kept.options_workspace, None);
    assert!(kept.providers.is_none() && kept.models.is_empty());

    // Options under another workspace than the focus discard the file.
    let mut foreign = with_options();
    foreign.options_workspace = Some("away".to_owned());
    write(&at, &deep(), &foreign).unwrap();
    assert!(read(&at).is_none());
}

/// A layout bump discards, which is what makes the version stamp worth
/// carrying: the options slot arrived in version 2, and a version 1 file has
/// no honest reading of it.
#[test]
fn a_file_of_the_previous_layout_is_discarded() {
    let at = path();
    write(&at, &deep(), &with_options()).unwrap();
    let body = std::fs::read_to_string(&at).unwrap();
    let older = body.replace(
        &format!("\"{}\":{}", super::TAG, super::VERSION),
        &format!("\"{}\":{}", super::TAG, super::VERSION - 1),
    );
    assert_ne!(older, body, "the stamp must be in the file to be lowered");
    std::fs::write(&at, older).unwrap();
    assert!(read(&at).is_none());
}
