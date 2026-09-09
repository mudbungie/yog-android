//! **The two replays REMOTE §3.2 asks every consumer for** (bl-e598), beside
//! the corpus replay in [`super`].
//!
//! `PROTOCOL` is a major now, and every ADDITION inside one ships with no
//! bump. So the question the plain replay cannot answer is: does this reader
//! survive an engine of the same major that is NEWER or OLDER than the corpus
//! it vendors? Two directions, and REMOTE §3.2 names one replay each.
//!
//! **Projection — the older engine.** *"For every reply shape and every
//! edition `e` from `floor` to the corpus's edition, delete from each frame
//! every key whose stamp exceeds `e` and decode: nothing refuses."* That is
//! replay at every recorded edition with no archive of old fixtures kept: the
//! stamps ARE the history, and one record projects to any point of it.
//!
//! **Mutation — the newer engine.** *"For every string-typed path in a reply
//! shape other than `kind`, replace the value in one frame with a token no
//! build has heard of and decode: nothing refuses."* Free text passes
//! trivially; a vocabulary passes only through its catch-all, which is the
//! whole of what this replay is for. `kind` is exempt because §3.2 rule 4
//! keeps the reply envelope's kind strict — a reader asks only what it paints.
//!
//! **What is vacuous today, said out loud.** The corpus's edition IS its floor
//! at a major's own cut, so the projection loop runs once and deletes nothing.
//! That is the honest state of a freshly cut major and not a broken check —
//! but a replay that can only pass is a replay that proves nothing, so
//! [`the_projector_deletes_what_a_stamp_puts_out_of_reach`] holds the
//! projector itself against a ledger that HAS grown. The day the corpus grows
//! a field, the loop below starts doing the work and that test stays its
//! witness.

use serde_json::Value;
use yog_android::codec::reply;

use super::expect::Expect;
use super::{corpus, frames, ledger, replies};

/// The vendored ledger, or the failure that says it will not read.
fn vendored() -> Result<ledger::Ledger, String> {
    let path = corpus().join("shapes.json");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    ledger::read(&text)
}

/// One shape's stamps and string paths, named so a missing row says which.
fn shape_of(vendored: &ledger::Ledger, shape: &str) -> Result<ledger::Shape, String> {
    vendored
        .shapes
        .get(&format!("reply/{shape}"))
        .cloned()
        .ok_or_else(|| format!("reply/{shape}: no row in corpus/shapes.json"))
}

/// A reply frame decodes when the outer answer is `Ok`: the INNER `Err` is a
/// refusal the envelope faithfully carried, which is a read.
fn reads(frame: &Value) -> Result<(), String> {
    reply::decode(frame).map(|_| ())
}

/// The reply shapes this client decodes — the `Reads` rows, because a shape
/// recorded as refused is refused on its `kind`, which stays strict.
fn read_shapes() -> Vec<String> {
    replies::REPLIES
        .iter()
        .filter(|(_, expect)| matches!(expect, Expect::Reads))
        .map(|(shape, _)| (*shape).to_owned())
        .collect()
}

/// **Projection replay**: every reply shape, at every edition from the floor
/// to the corpus's own, with every post-`e` key deleted.
#[test]
fn every_reply_shape_reads_at_every_edition_of_this_major() {
    let vendored = vendored().unwrap();
    let mut editions = 0;
    for shape in read_shapes() {
        let stamps = shape_of(&vendored, &shape).unwrap().stamps;
        for e in vendored.floor..=vendored.edition {
            editions += 1;
            for frame in frames("reply", &shape).unwrap() {
                let mut projected = frame.clone();
                ledger::project("", &mut projected, &stamps, e);
                reads(&projected)
                    .unwrap_or_else(|why| panic!("{shape} at edition {e}: {why}\n{projected}"));
            }
        }
    }
    assert!(editions > 0, "no reply shape was projected at any edition");
}

/// **Word mutation**: every string-typed path but `kind`, replaced with a
/// token no build has heard of. A vocabulary passes only through its
/// catch-all — which is what makes this the assertion that REMOTE §3.2's rule
/// 3 is actually implemented rather than merely written down.
#[test]
fn every_word_a_reply_carries_may_be_one_this_build_has_never_heard_of() {
    const STRANGER: &str = "a-word-no-build-has-heard-of";
    let vendored = vendored().unwrap();
    let mut mutated = 0;
    for shape in read_shapes() {
        let paths = shape_of(&vendored, &shape).unwrap().strings;
        for path in &paths {
            if path == "/kind" {
                continue;
            }
            for frame in frames("reply", &shape).unwrap() {
                let mut said = frame.clone();
                if !ledger::mutate(path, "", &mut said, STRANGER) {
                    continue;
                }
                mutated += 1;
                reads(&said).unwrap_or_else(|why| panic!("{shape} at {path}: {why}\n{said}"));
            }
        }
    }
    assert!(mutated > 0, "no word in any reply frame was mutated");
}

/// The projector itself, against a ledger that has grown — the witness the
/// module doc names, because the vendored one has not.
#[test]
fn the_projector_deletes_what_a_stamp_puts_out_of_reach() {
    let stamps = [("/rows/[]/mood".to_owned(), 20), ("/whole".to_owned(), 25)]
        .into_iter()
        .collect();
    let frame = serde_json::json!({
        "kind": "conversations", "whole": "later",
        "rows": [{ "tone": "plain", "mood": "chipper" }],
    });
    let mut at_19 = frame.clone();
    ledger::project("", &mut at_19, &stamps, 19);
    assert_eq!(
        at_19,
        serde_json::json!({ "kind": "conversations", "rows": [{ "tone": "plain" }] })
    );
    let mut at_25 = frame.clone();
    ledger::project("", &mut at_25, &stamps, 25);
    assert_eq!(at_25, frame);
}

/// The mutator's own two answers: it replaces a string wherever the path
/// reaches one, and it says so when the path reached nothing — which is what
/// stops a mis-spelled path from passing as a mutation nobody made.
#[test]
fn the_mutator_says_whether_it_reached_anything() {
    let mut frame = serde_json::json!({ "rows": [{ "tone": "plain" }, { "tone": "weak" }] });
    assert!(ledger::mutate("/rows/[]/tone", "", &mut frame, "shiny"));
    assert_eq!(
        frame,
        serde_json::json!({ "rows": [{ "tone": "shiny" }, { "tone": "shiny" }] })
    );
    assert!(!ledger::mutate("/rows/[]/nowhere", "", &mut frame, "shiny"));
    // A path that names a value which is not a string is reached and left.
    let mut counted = serde_json::json!({ "rows": 3 });
    assert!(!ledger::mutate("/rows", "", &mut counted, "shiny"));
}
