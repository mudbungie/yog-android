//! The four ops on the wire, as yog's corpus spells them, and what a row
//! says and enables.

use super::{OfferAct, OfferRow, Offered, Pending, Settle, State, offered, rows};
use crate::codec::{Act, Ask, Gesture, decode, encode};
use crate::leaf::Grade;
use serde_json::{Value, json};

fn object(v: &Value) -> serde_json::Map<String, Value> {
    v.as_object().cloned().unwrap()
}

/// Each request encodes to the server's frame and reads back as itself — a
/// grade word this build cannot spell included, carried rather than flattened
/// (REMOTE §3.2).
#[test]
fn the_four_requests_round_trip_as_the_servers_frames() {
    let cases = [
        (
            Gesture::Act(Act::Offer(OfferAct::Mint {
                workspace: "ws".into(),
                name: "phone-1".into(),
                grade: Grade::Operator,
            })),
            json!({ "op": "offer", "workspace": "ws", "name": "phone-1", "grade": "operator" }),
        ),
        (
            Gesture::Act(Act::Offer(OfferAct::Mint {
                workspace: "ws".into(),
                name: "builder".into(),
                grade: Grade::Foot,
            })),
            json!({ "op": "offer", "workspace": "ws", "name": "builder", "grade": "foot" }),
        ),
        (
            Gesture::Act(Act::Offer(OfferAct::Mint {
                workspace: "ws".into(),
                name: "x".into(),
                grade: Grade::Unknown("admin".into()),
            })),
            json!({ "op": "offer", "workspace": "ws", "name": "x", "grade": "admin" }),
        ),
        (
            Gesture::Ask(Ask::Offers {
                workspace: "ws".into(),
            }),
            json!({ "op": "offers", "workspace": "ws" }),
        ),
        (
            Gesture::Act(Act::Offer(OfferAct::Settle {
                settle: Settle::Approve,
                workspace: "ws".into(),
                name: "phone-1".into(),
            })),
            json!({ "op": "approve", "workspace": "ws", "name": "phone-1" }),
        ),
        (
            Gesture::Act(Act::Offer(OfferAct::Settle {
                settle: Settle::Decline,
                workspace: "ws".into(),
                name: "phone-1".into(),
            })),
            json!({ "op": "decline", "workspace": "ws", "name": "phone-1" }),
        ),
    ];
    for (gesture, frame) in cases {
        assert_eq!(encode(&gesture), frame);
        assert_eq!(decode(&frame), Ok(gesture));
    }
}

/// `offered` is the URL and its expiry; either missing is named.
#[test]
fn the_offered_reply_is_a_url_and_an_expiry() {
    let frame = json!({ "ok": true, "kind": "offered",
        "url": "https://engine.invalid/yog/register/ABCD2345", "expires": 1_700_000_600 });
    assert_eq!(
        offered(&object(&frame)),
        Ok(Offered {
            url: "https://engine.invalid/yog/register/ABCD2345".into(),
            expires: 1_700_000_600,
        })
    );
    assert!(offered(&object(&json!({ "url": "u" }))).is_err());
    assert!(offered(&object(&json!({ "expires": 1 }))).is_err());
}

fn listed() -> Vec<OfferRow> {
    rows(&object(&json!({ "rows": [
        { "name": "phone-1", "grade": "operator", "state": "open", "expires": 1_700_000_600 },
        { "name": "tablet", "grade": "operator", "state": "claimed", "expires": 1_700_000_600,
          "word": "K7M2QX" },
        { "name": "builder", "grade": "foot", "state": "proved", "expires": 1_700_000_600,
          "word": "2LFTUR" },
        { "name": "later", "grade": "operator", "state": "parked", "expires": 0 },
    ] })))
    .unwrap()
}

/// **Only a proved row may be approved**, a word is absent until a device has
/// claimed, and a state this build has not heard of is carried as its word
/// and enables nothing.
#[test]
fn a_row_says_its_state_and_word_and_only_proved_is_approvable() {
    let listed = listed();
    let said: Vec<String> = listed.iter().map(OfferRow::said).collect();
    assert_eq!(
        said,
        [
            "phone-1 · operator · open · no word yet",
            "tablet · operator · claimed · word K7M2QX",
            "builder · foot · proved · word 2LFTUR",
            "later · operator · parked · no word yet",
        ]
    );
    let approvable: Vec<bool> = listed.iter().map(OfferRow::approvable).collect();
    assert_eq!(approvable, [false, false, true, false]);
    assert_eq!(
        listed.last().unwrap().state,
        State::Unknown("parked".into())
    );
}

/// A row that is not an object, and a listing with no rows at all, are named.
#[test]
fn a_malformed_listing_is_refused() {
    assert_eq!(
        rows(&object(&json!({ "rows": [7] }))).unwrap_err(),
        "offers: row is not an object"
    );
    assert!(rows(&object(&json!({}))).is_err());
    assert!(rows(&object(&json!({ "rows": [{ "name": "x" }] }))).is_err());
}

/// The surface's value answers which workspace it is about, and the two
/// settling acts wear their ops.
#[test]
fn a_pending_listing_is_about_its_workspace() {
    let pending = Pending {
        workspace: "home".into(),
        minted: None,
        rows: Vec::new(),
    };
    assert!(pending.about("home") && !pending.about("away"));
    assert_eq!(Settle::Approve.op(), "approve");
    assert_eq!(Settle::Decline.op(), "decline");
}
