//! **The offers surface** (REMOTE §15.1, DESIGN §13.18): what the mint
//! sends and holds, the read behind every act, and the closing that crosses
//! no wire.

use serde_json::{Value, json};

use super::{conv_reply, nothing_set, ops, settle, tr_reply, ws_reply};
use crate::codec::Settle;
use crate::leaf::Grade;

const URL: &str = "https://engine.invalid/yog/register/ABCD2345";

fn offered() -> Vec<u8> {
    json!({ "ok": true, "kind": "offered", "url": URL, "expires": 1_700_000_600 })
        .to_string()
        .into_bytes()
}

fn offers(state: &str) -> Vec<u8> {
    json!({ "ok": true, "kind": "offers", "rows": [
        { "name": "phone-2", "grade": "foot", "state": state, "expires": 1_700_000_600,
          "word": "2LFTUR" } ] })
    .to_string()
    .into_bytes()
}

fn receipt(kind: &str) -> Vec<u8> {
    json!({ "ok": true, "kind": kind }).to_string().into_bytes()
}

fn focused_scripts() -> Vec<Vec<Vec<u8>>> {
    vec![
        vec![ws_reply()],
        vec![nothing_set()],
        vec![ws_reply()],
        vec![conv_reply()],
    ]
}

fn after() -> Vec<Vec<Vec<u8>>> {
    vec![vec![ws_reply()], vec![conv_reply()]]
}

fn focused(model: &mut super::Model) {
    settle(model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(model, &|s| !s.conversations.is_empty());
}

fn frame(requests: &[Vec<u8>], at: usize) -> Value {
    serde_json::from_slice(requests.get(at).unwrap_or_else(|| unreachable!())).unwrap()
}

/// **The mint names the workspace, the device and the grade**; the URL rides
/// back beside the pending rows it now appears among, and a refresh keeps it.
#[test]
fn a_mint_names_the_three_and_the_url_stands_beside_the_pending_rows() {
    let mut scripts = focused_scripts();
    scripts.extend([vec![offered()], vec![offers("open")]]);
    scripts.extend(after());
    scripts.push(vec![offers("proved")]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.offer("phone-2".into(), Grade::Foot);
    let snap = settle(&mut model, &|s| s.offers.is_some());
    let held = snap.offers.unwrap_or_else(|| unreachable!());
    assert!(held.about("home"));
    let (name, minted) = held.minted.unwrap_or_else(|| unreachable!());
    assert_eq!((name.as_str(), minted.url.as_str()), ("phone-2", URL));
    assert!(!held.rows.iter().any(crate::codec::OfferRow::approvable));
    model.offers();
    let snap = settle(&mut model, &|s| {
        s.offers
            .as_ref()
            .is_some_and(|held| held.rows.iter().any(crate::codec::OfferRow::approvable))
    });
    assert!(
        snap.offers.is_some_and(|held| held.minted.is_some()),
        "a refresh keeps the URL the surface shows"
    );
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        frame(&requests, 4),
        json!({ "op": "offer", "workspace": "home", "name": "phone-2", "grade": "foot" })
    );
    assert_eq!(
        frame(&requests, 5),
        json!({ "op": "offers", "workspace": "home" })
    );
}

/// **Approve and decline each send their op and re-read the rows**, and
/// closing the surface drops it and sends nothing.
#[test]
fn settling_sends_the_op_and_rereads_and_closing_crosses_no_wire() {
    let mut scripts = focused_scripts();
    scripts.extend([vec![receipt("approved")], vec![offers("open")]]);
    scripts.extend(after());
    scripts.extend([vec![receipt("declined")], vec![offers("claimed")]]);
    scripts.extend(after());
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.settle(Settle::Approve, "phone-2".into());
    settle(&mut model, &|s| s.offers.is_some());
    model.settle(Settle::Decline, "phone-2".into());
    let snap = settle(&mut model, &|s| {
        s.offers.as_ref().is_some_and(|held| {
            held.rows
                .iter()
                .any(|row| row.state == crate::codec::offer::State::Claimed)
        })
    });
    assert_eq!(snap.error, None);
    model.forget();
    settle(&mut model, &|s| s.offers.is_none());
    drop(model);
    let requests = served.join().unwrap();
    assert_eq!(
        frame(&requests, 4),
        json!({ "op": "approve", "workspace": "home", "name": "phone-2" })
    );
    let wire = ops(&requests);
    let count = |op: &str| wire.iter().filter(|said| *said == op).count();
    assert_eq!(
        (count("approve"), count("decline"), count("offers")),
        (1, 1, 2)
    );
}

/// **With no workspace focused nothing crosses**, and an answer of another
/// kind names the act — for the mint, for a settle and for the read.
#[test]
fn no_workspace_crosses_nothing_and_a_wrong_kind_names_the_act() {
    let (mut model, served) = super::model_against(vec![vec![ws_reply()]; 3]);
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.offer("phone-2".into(), Grade::Foot);
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("offer: no workspace is focused")
    );
    model.settle(Settle::Approve, "phone-2".into());
    let snap = settle(&mut model, &|s| {
        s.error.as_deref() == Some("approve: no workspace is focused")
    });
    assert!(snap.offers.is_none());
    drop(model);
    assert!(
        ops(&served.join().unwrap())
            .iter()
            .all(|op| op == "workspaces")
    );

    for (fire, said) in [
        (0, "offer: the server answered transcript instead"),
        (1, "decline: the server answered transcript instead"),
    ] {
        let mut scripts = focused_scripts();
        scripts.extend([vec![tr_reply()], vec![tr_reply()]]);
        scripts.extend(after());
        let (mut model, served) = super::model_against(scripts);
        focused(&mut model);
        if fire == 0 {
            model.offer("phone-2".into(), Grade::Foot);
        } else {
            model.settle(Settle::Decline, "phone-2".into());
        }
        let snap = settle(&mut model, &|s| s.error.is_some());
        assert_eq!(snap.error.as_deref(), Some(said));
        assert!(snap.offers.is_none());
        drop(model);
        served.join().unwrap();
    }

    let mut scripts = focused_scripts();
    scripts.push(vec![tr_reply()]);
    scripts.extend(after());
    let (mut model, served) = super::model_against(scripts);
    focused(&mut model);
    model.offers();
    let snap = settle(&mut model, &|s| s.error.is_some());
    assert_eq!(
        snap.error.as_deref(),
        Some("offers: the server answered transcript instead")
    );
    drop(model);
    served.join().unwrap();
}
