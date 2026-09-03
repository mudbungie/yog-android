//! **The options half of the file** (bl-0267, bl-e9f9): the selectors'
//! offerings and the assignments beside them, the workspace that owns both,
//! and the version stamp that discards what an older build wrote.

use super::super::{Envelopes, TAG, VERSION, read, write};
use super::{all, deep, path, ws};
use serde_json::json;

/// The same, with the selectors' offerings for the focused workspace
/// (bl-0267).
fn with_options() -> Envelopes {
    Envelopes {
        options_workspace: Some("home".to_owned()),
        providers: Some(json!({ "kind": "providers", "ok": true,
                                "rows": [{ "name": "acme", "fact": "credential present", "effort": true, "priority": true,
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
        &format!("\"{TAG}\":{VERSION}"),
        &format!("\"{TAG}\":{}", VERSION - 1),
    );
    assert_ne!(older, body, "the stamp must be in the file to be lowered");
    std::fs::write(&at, older).unwrap();
    assert!(read(&at).is_none());
}

/// **A cache written by the last build discards cleanly** (bl-e837). The
/// installed app stored its envelopes at PROTOCOL 2; this one speaks 4, and
/// the file names the version it was written at, so it is refused whole
/// rather than half-read. Two independent things would each refuse it — the
/// stamp, and the rows themselves, which were written before
/// `last_active_unix` existed and no longer decode — and the test pins both,
/// because the second is what would catch a future author who decided the
/// stamp was redundant.
#[test]
fn a_cache_from_the_previous_protocol_discards() {
    let at = path();
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    let old_row = json!({ "root_id": "a1", "display": "d", "display_only": false,
                          "state": "quiescent", "uncertain": false, "preview": "",
                          "age_secs": 0, "attention": 0, "members": 1, "direct": 0,
                          "stoppable": false, "stop_children": false, "depth": 0,
                          "tone": "plain" });
    let stale = json!({
        TAG: VERSION, "protocol": 2,
        "focus": { "workspace": "home", "agent": null },
        "workspaces": ws(),
        "conversations": { "ok": true, "kind": "conversations", "rows": [old_row] },
        "transcript": null });
    std::fs::write(&at, stale.to_string()).unwrap();
    assert!(read(&at).is_none(), "the version stamp refuses it");

    // …and with the stamp forged to this build's, the rows still refuse: a
    // conversation row without `last_active_unix` is not one this build can
    // read (REMOTE §9.9).
    let forged = stale.to_string().replace(
        "\"protocol\":2",
        &format!("\"protocol\":{}", crate::hello::PROTOCOL),
    );
    std::fs::write(&at, forged).unwrap();
    assert!(read(&at).is_none(), "the rows refuse it too");
}
