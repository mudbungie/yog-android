//! The claim end to end against a door played by a closure, the word read
//! back off the files, and the watch against a real mTLS server on loopback.

use super::key::tests::VECTOR_CA;
use super::{Answer, Registered, Watch, Watcher, answer, claim, word};
use crate::material::{ADDRESS, ANCHORS, KEY};
use crate::test_support::{mint_ca, mint_leaf, scratch, serve_many};
use rustls::pki_types::SubjectPublicKeyInfoDer;
use rustls::pki_types::pem::PemObject;
use serde_json::{Value, json};

const URL: &str = "https://engine.example/yog/register/ABCD2345";

fn issued() -> Vec<u8> {
    json!({
        "workspace": "home",
        "name": "phone",
        "grade": "operator",
        "address": ["engine.example:7737", "192.0.2.7:7737"],
        "ca": VECTOR_CA,
        "cert": VECTOR_CA,
    })
    .to_string()
    .into_bytes()
}

fn door(status: u16, body: Vec<u8>) -> impl FnMut(&str, &str) -> Result<Answer, String> {
    move |_, _| {
        Ok(Answer {
            status,
            body: body.clone(),
        })
    }
}

/// **The whole claim**: the public half of a key made here goes to the URL as
/// the one JSON field, the answer is filed with the private half, and the
/// word said is the word the files give back.
#[test]
fn a_claim_posts_the_public_key_files_the_answer_and_says_the_word() {
    let dir = scratch().join("wire");
    let mut posted = Vec::new();
    let registered = claim(&dir, &format!("  {URL}\n"), &mut |url, body| {
        posted.push((url.to_owned(), body.to_owned()));
        Ok(Answer {
            status: 200,
            body: issued(),
        })
    })
    .unwrap();
    let [(url, body)] = posted.as_slice() else {
        panic!("one request, not {}", posted.len());
    };
    assert_eq!(url, URL);
    let body: Value = serde_json::from_str(body).unwrap();
    let sent = body["public_key"].as_str().unwrap();
    let spki = SubjectPublicKeyInfoDer::from_pem_slice(sent.as_bytes()).unwrap();
    assert_eq!(spki.as_ref().len(), 91);
    assert_eq!(
        body.as_object().unwrap().len(),
        1,
        "the key and nothing else"
    );
    assert_eq!(
        registered,
        Registered {
            workspace: "home".to_owned(),
            name: "phone".to_owned(),
            grade: "operator".to_owned(),
            word: word(&dir).unwrap(),
            addresses: vec![
                "engine.example:7737".to_owned(),
                "192.0.2.7:7737".to_owned()
            ],
        }
    );
    assert_eq!(registered.word.len(), 6);
    assert_eq!(
        registered.said(),
        "Registered as phone (operator) in workspace home."
    );
    assert!(registered.approve().contains("`lernie approve home phone`"));
    let key = std::fs::read_to_string(dir.join(KEY)).unwrap();
    assert!(
        super::key::spki_of(&key).is_some(),
        "the private half is filed as a key this app reads back"
    );
    assert!(!sent.contains("PRIVATE"), "the private half is never sent");
}

/// **Every status but 200 is one sentence**, and nothing is filed: the door
/// answers a bare 404 for a code that is unknown, expired or used.
#[test]
fn anything_but_200_is_an_offer_that_cannot_be_claimed() {
    for status in [404, 502, 301] {
        let dir = scratch().join("wire");
        let refusal = claim(&dir, URL, &mut door(status, Vec::new())).unwrap_err();
        assert_eq!(
            refusal,
            format!(
                "this offer cannot be claimed: engine.example answered {status}. The code is \
                 unknown, expired or already used — a registration URL works once. Ask for a \
                 new offer."
            )
        );
        assert!(!dir.exists(), "nothing was filed");
    }
}

/// A request that reached nothing spent nothing, and says so; text that is
/// not a registration URL refuses before the door is asked at all.
#[test]
fn a_door_that_was_not_reached_and_a_url_that_is_not_one_claim_nothing() {
    let dir = scratch().join("wire");
    let refusal = claim(&dir, URL, &mut |_, _| Err("no route".to_owned())).unwrap_err();
    assert!(
        refusal.starts_with("this offer could not be reached: no route. Nothing was claimed"),
        "{refusal}"
    );
    let mut asked = false;
    let refusal = claim(&dir, "{\"yog-enroll\":1}", &mut |_, _| {
        asked = true;
        Err(String::new())
    })
    .unwrap_err();
    assert!(refusal.contains("is not a registration URL"), "{refusal}");
    assert!(!asked && !dir.exists());
}

/// A 200 that is not the answer files nothing: a field missing, and a `ca`
/// with no certificate in it to read the word from.
#[test]
fn an_answer_that_is_not_the_answer_files_nothing() {
    let dir = scratch().join("wire");
    let refusal = claim(&dir, URL, &mut door(200, b"<html>".to_vec())).unwrap_err();
    assert!(refusal.contains("nothing was filed"), "{refusal}");
    let mut body: Value = serde_json::from_slice(&issued()).unwrap();
    body["ca"] = json!("not a certificate");
    let refusal = claim(&dir, URL, &mut door(200, body.to_string().into_bytes())).unwrap_err();
    assert!(refusal.contains("no CA certificate"), "{refusal}");
    assert!(!dir.exists());
}

/// The bridge's protocol: `ok`, the status, the body verbatim — or `err` and
/// a sentence. Anything else is not an answer.
#[test]
fn the_bridges_answer_is_read_as_a_status_and_a_body() {
    assert_eq!(
        answer("ok\n200\n{\"a\":\n1}"),
        Ok(Answer {
            status: 200,
            body: b"{\"a\":\n1}".to_vec()
        })
    );
    assert_eq!(
        answer("ok\n404"),
        Ok(Answer {
            status: 404,
            body: Vec::new()
        })
    );
    assert_eq!(
        answer("err\nno route to host\n"),
        Err("no route to host".to_owned())
    );
    for unread in ["", "ok", "ok\nsoon\n{}"] {
        let refusal = answer(unread).unwrap_err();
        assert!(
            refusal.contains("nothing readable"),
            "{unread:?}: {refusal}"
        );
    }
}

/// Material this app did not make has no word: nothing filed, and a leaf
/// carried here by hand under another key.
#[test]
fn material_that_was_never_offered_has_no_word() {
    let dir = scratch();
    assert_eq!(word(&dir), None);
    mint_ca(&dir, "ca");
    mint_leaf(&dir, "ca", "client", false);
    assert_eq!(word(&dir), None, "an RSA key is not one this app made");
    std::fs::remove_file(dir.join(ANCHORS)).unwrap();
    assert_eq!(word(&dir), None);
}

/// A material directory the test PKI fills under the reader's own names,
/// dialling a server that answers each connection with the next of `frames`.
fn watched(frames: Vec<Value>) -> Watcher {
    let dir = scratch();
    mint_ca(&dir, "ca");
    mint_leaf(&dir, "ca", "server", true);
    mint_leaf(&dir, "ca", "client", false);
    let scripts = frames
        .into_iter()
        .map(|frame| vec![frame.to_string().into_bytes()])
        .collect();
    let (address, _served) = serve_many(&dir, "ca", "server", scripts);
    std::fs::write(dir.join(ADDRESS), address).unwrap();
    Watcher::new(&dir)
}

fn refusal(said: &str) -> Value {
    json!({ "ok": false, "error": said })
}

/// **An operator-grade device waits through the hold's refusal and is
/// approved by the first answer**.
#[test]
fn a_held_device_waits_until_a_request_is_answered() {
    let mut watcher = watched(vec![
        refusal("held"),
        refusal("held"),
        json!({ "ok": true, "kind": "workspaces", "rows": [] }),
    ]);
    assert_eq!(watcher.look(), Watch::Waiting);
    assert_eq!(watcher.look(), Watch::Waiting);
    assert_eq!(watcher.look(), Watch::Approved);
}

/// **A foot is approved by its refusal CHANGING**: it may never ask this
/// question, so what ends its wait is being refused for its grade rather
/// than for its hold. An answer this build cannot read is a change too.
#[test]
fn a_refusal_that_changes_ends_the_wait() {
    let mut watcher = watched(vec![refusal("held"), refusal("a foot may not ask that")]);
    assert_eq!(watcher.look(), Watch::Waiting);
    assert_eq!(watcher.look(), Watch::Approved);
    let mut watcher = watched(vec![json!({ "ok": true, "kind": 7 })]);
    assert_eq!(watcher.look(), Watch::Approved);
}

/// A dial that reached nothing is said as that, with the endpoint's own
/// sentence — and so is a directory with nothing filed in it.
#[test]
fn a_dial_that_reaches_nothing_is_not_a_wait() {
    let mut watcher = watched(Vec::new());
    std::fs::write(watcher.dir.join(ADDRESS), "127.0.0.1:1").unwrap();
    let Watch::Unreached(why) = watcher.look() else {
        panic!("reached");
    };
    assert!(why.starts_with("connect 127.0.0.1:1: "), "{why}");
    assert_eq!(
        Watcher::new(&scratch()).look(),
        Watch::Unreached("no material is filed".to_owned())
    );
}

/// The three lines the screen puts under the word.
#[test]
fn each_watch_says_its_line() {
    assert!(Watch::Waiting.said().starts_with("Waiting for approval: "));
    assert_eq!(Watch::Approved.said(), "Approved.");
    let said = Watch::Unreached("connect engine.example:7737: refused".to_owned()).said();
    assert!(
        said.contains("cannot be approved: connect engine.example:7737: refused. The material"),
        "{said}"
    );
}
