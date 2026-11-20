//! **A provider on another machine** (yog REMOTE §5.7; DESIGN §25): a row
//! named `<row>@<client>` is that row on that registered machine, and the
//! name is the whole of how the wire says so. So what this file pins is what
//! is SENT — every op that names a provider carries the listing's own name,
//! qualification included, and what comes back is kept under that same name,
//! apart from the server's row of the same bare name.
//!
//! How the name is shown — under its machine, without the qualification — is
//! `codec::pick::machine`'s and is asserted there.

use super::signin::{focused, frame, tail};
use super::{Turn, conv_reply, nothing_set, ops, settle, ws_reply};
use serde_json::{Value, json};

/// The server's own `chat`, and the same row on the machine `foot`.
fn rows(kind: &str) -> Vec<u8> {
    let row = |name: &str, fact: &str| json!({ "name": name, "fact": fact, "effort": true, "priority": false, "blocked": null });
    json!({ "ok": true, "kind": kind,
            "rows": [row("chat", "not signed in"), row("chat@foot", "signed in")] })
    .to_string()
    .into_bytes()
}

fn said(request: &[u8]) -> Value {
    serde_json::from_slice(request).unwrap_or_default()
}

/// **Listed, asked for its models, and assigned — by its whole name.** The
/// listing keeps the row as the server spelled it, `models` and `model` name
/// it whole, and the models that come back are the qualified row's own:
/// nothing is filed under the server's `chat`.
#[test]
fn a_row_on_another_machine_is_listed_and_its_models_and_its_pick_name_it_whole() {
    let models = json!({ "ok": true, "kind": "models", "rows": ["opus"] })
        .to_string()
        .into_bytes();
    let (mut model, served) = super::model_against(vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
        vec![rows("providers")], // the providers gesture
        vec![ws_reply()],
        vec![conv_reply()],
        vec![models], // the models gesture
        vec![ws_reply()],
        vec![conv_reply()],
        vec![super::pick::applied()], // the pick
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
    ]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(&mut model, &|s| !s.conversations.is_empty());

    model.list_providers();
    let snap = settle(&mut model, &|s| !s.providers.is_empty());
    let listed: Vec<_> = snap.providers.iter().map(|row| row.name.as_str()).collect();
    assert_eq!(listed, ["chat", "chat@foot"]);
    assert_eq!(snap.providers[1].fact, "signed in");

    model.list_models("chat@foot".into());
    let snap = settle(&mut model, &|s| !s.models.is_empty());
    assert_eq!(snap.models["chat@foot"], ["opus"]);
    assert!(
        !snap.models.contains_key("chat"),
        "the server's own row was not asked and holds nothing"
    );

    model.pick_model("chat@foot".into(), "opus".into());
    let snap = settle(&mut model, &|s| s.error.is_none() && !s.models.is_empty());
    assert_eq!(snap.error, None);

    drop(model);
    let requests = served.join().unwrap_or_default();
    assert_eq!(ops(&requests)[7], "models");
    assert_eq!(
        said(&requests[7]),
        json!({ "op": "models", "workspace": "home", "provider": "chat@foot" })
    );
    assert_eq!(
        said(&requests[10]),
        json!({ "op": "model", "workspace": "home", "role": "worker",
                "provider": "chat@foot", "model": "opus" })
    );
}

/// **Signed in by its whole name, and followed by it.** The catalog lists
/// the row, `login` and `login-tail` name it whole — so the run is the far
/// machine's — and the tail that stands is that row's, never painted under
/// the server's row of the same bare name.
#[test]
fn a_row_on_another_machine_is_signed_in_and_followed_by_its_whole_name() {
    let (mut model, served) = focused(vec![
        Turn::Answer(vec![rows("catalog")]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
        Turn::Answer(vec![frame(
            &[(true, "open https://auth.invalid and enter ABCD-1234")],
            None,
        )]),
        Turn::Answer(vec![ws_reply()]),
        Turn::Answer(vec![conv_reply()]),
        Turn::Hold(Vec::new()),
    ]);
    model.list_catalog();
    let snap = settle(&mut model, &|s| !s.catalog.is_empty());
    assert_eq!(snap.catalog[1].name, "chat@foot");

    model.sign_in("chat@foot".into());
    let snap = settle(&mut model, &|s| tail(s, "chat@foot").is_some());
    assert_eq!(
        tail(&snap, "chat@foot").unwrap_or_default(),
        ["open https://auth.invalid and enter ABCD-1234"],
        "the device flow's address and code reach this seat"
    );
    assert_eq!(tail(&snap, "chat"), None);

    drop(model);
    let requests = served.join().unwrap_or_default();
    assert_eq!(
        ops(&requests)[4..],
        [
            "catalog",
            "workspaces",
            "conversations",
            "login",
            "workspaces",
            "conversations",
            "login-tail"
        ]
    );
    assert_eq!(
        said(&requests[7]),
        json!({ "op": "login", "workspace": "home", "provider": "chat@foot" })
    );
    assert_eq!(
        said(&requests[10]),
        json!({ "op": "login-tail", "workspace": "home", "provider": "chat@foot" })
    );
}
