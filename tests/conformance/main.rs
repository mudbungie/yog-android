//! **The wire conformance corpus, replayed** (yog REMOTE §3, upstream
//! bl-32cb): one canonical fixture set, generated from the server's own codec,
//! that every client of the wire proves itself against.
//!
//! Why a corpus and not a shared types crate, in REMOTE §3's own words: *"Four
//! components implement one vocabulary, in more than one language, so the
//! failure mode here is not a refusal — it is a quiet miss: a field one end
//! drops and the other never notices, on a wire whose strict decode only ever
//! sees what was actually written. A shared types crate was weighed and
//! declined: it protects only same-language consumers (the android client
//! cannot link it) and couples four release cadences for the one consumer it
//! does protect. A corpus protects every consumer, because a fixture is
//! data."*
//!
//! And what this client owes it, also verbatim: *"decode every frame in both
//! directories into its own types, and round-trip what it emits — decode then
//! re-encode must return the frame exactly. A client that only sends requests
//! still decodes the request fixtures; that is what catches a field it drops
//! on the way out. A shape a client does not implement is still one it must
//! not misread, so skipping a fixture is a decision recorded in the client,
//! never a silent pass."*
//!
//! **The fixtures are vendored**, because REMOTE §3 is explicit that *"there
//! is no published artifact and no endpoint that serves it — the corpus
//! travels with the component that generates it."* A vendored copy can go
//! stale, so the first test below pins the version it was cut at against this
//! build's own `PROTOCOL`: the day the wire's meaning moves, this suite says
//! so rather than passing over a corpus from the era before.
//!
//! **Three failures this replay can produce, and each names its own remedy.**
//! A frame that does not decode is a codec miss. A frame that decodes but does
//! not re-encode to itself is a field dropped on the way out — the quiet miss.
//! A shape with no recorded decision is a vocabulary that grew upstream, and
//! the remedy is a row in `requests.rs`/`replies.rs`, decided rather than
//! defaulted.

mod expect;
mod replies;
mod requests;

use expect::Expect;
use serde_json::Value;
use std::path::PathBuf;
use yog_android::codec::{self, reply};
use yog_android::hello::PROTOCOL;

/// The vendored corpus, beside the manifest that vendored it.
fn corpus() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus")
}

/// One fixture file, read whole. The `direction` and `shape` it states are
/// checked against where it was found: a fixture edited by hand or filed in
/// the wrong directory is not a fixture, and the whole value of vendoring
/// generated data is that nobody here authored it.
///
/// Every failure comes back as an `Err` rather than a panic so the helpers are
/// ordinary total functions — the panic vocabulary belongs to the `#[test]`
/// items below, which is also the only place this crate's lints allow it.
fn frames(direction: &str, shape: &str) -> Result<Vec<Value>, String> {
    let path = corpus().join(direction).join(format!("{shape}.json"));
    let at = |e: &dyn std::fmt::Display| format!("{}: {e}", path.display());
    let text = std::fs::read_to_string(&path).map_err(|e| at(&e))?;
    let fixture: Value = serde_json::from_str(&text).map_err(|e| at(&e))?;
    let stated = |key: &str| fixture.get(key).cloned().unwrap_or(Value::Null);
    let says = |key: &str, word: &str| stated(key).as_str() == Some(word);
    if !says("direction", direction) || !says("shape", shape) {
        return Err(at(&"a fixture filed under a shape it does not claim"));
    }
    if stated("protocol").as_u64() > Some(u64::from(PROTOCOL)) {
        return Err(at(&"stamped past the protocol this build speaks"));
    }
    stated("frames")
        .as_array()
        .cloned()
        .ok_or_else(|| at(&"no frames"))
}

/// The shapes a directory actually carries, off the filenames.
fn shapes(direction: &str) -> Result<Vec<String>, String> {
    let dir = corpus().join(direction);
    let read = std::fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut found: Vec<String> = read
        .filter_map(Result::ok)
        .map(|e| {
            e.file_name()
                .to_string_lossy()
                .trim_end_matches(".json")
                .to_owned()
        })
        .collect();
    found.sort();
    Ok(found)
}

/// The table's shapes, sorted, so a row's position in the file is never part
/// of the contract.
fn recorded(table: &[(&str, Expect)]) -> Vec<String> {
    let mut listed: Vec<String> = table.iter().map(|(shape, _)| (*shape).to_owned()).collect();
    listed.sort();
    listed
}

/// The vendored copy is for the protocol this build speaks. REMOTE §3:
/// *"`corpus/shapes.json` is the standing record: per shape, its field
/// signature and that version, plus the version the corpus as a whole is
/// for."* A newer corpus vendored in, or a `PROTOCOL` bumped here without
/// re-vendoring, fails here rather than one frame at a time downstream.
#[test]
fn the_vendored_corpus_is_the_protocol_this_build_speaks() {
    let text = std::fs::read_to_string(corpus().join("shapes.json")).unwrap();
    let record: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(record["protocol"].as_u64(), Some(u64::from(PROTOCOL)));
    // Every fixture is stamped too, and `frames` refuses one from a later era
    // — so a corpus re-vendored from a yog that has moved on fails here.
    for (direction, table) in [
        ("request", recorded(requests::REQUESTS)),
        ("reply", recorded(replies::REPLIES)),
    ] {
        for shape in table {
            frames(direction, &shape).unwrap();
        }
    }
}

/// Both directions over the request vocabulary: no shape without a decision,
/// no decision without a shape. This is what makes a vocabulary that grows
/// upstream arrive as a question rather than as silence.
#[test]
fn every_request_shape_is_a_recorded_decision() {
    assert_eq!(recorded(requests::REQUESTS), shapes("request").unwrap());
}

#[test]
fn every_reply_shape_is_a_recorded_decision() {
    assert_eq!(recorded(replies::REPLIES), shapes("reply").unwrap());
}

/// Rules 1 and 2 over `corpus/request/`: every frame this codec spells decodes
/// **and re-encodes to itself**, and every frame it does not spell is refused
/// naming the op.
#[test]
fn every_request_frame_reads_or_is_refused_by_name() {
    for (shape, expect) in requests::REQUESTS {
        let frames = frames("request", shape).unwrap();
        let mut read = 0;
        for frame in &frames {
            match codec::decode(frame) {
                Ok(gesture) => {
                    assert_eq!(&codec::encode(&gesture), frame, "{shape}: round trip");
                    read += 1;
                }
                Err(e) => refused_by_name(shape, &e),
            }
        }
        held(shape, *expect, read, frames.len());
    }
}

/// Rule 1 over `corpus/reply/`, and rule 3. There is no rule 2 here: this
/// client emits no reply (see `replies.rs`).
#[test]
fn every_reply_frame_reads_or_is_refused_by_name() {
    for (shape, expect) in replies::REPLIES {
        let frames = frames("reply", shape).unwrap();
        let mut read = 0;
        for frame in &frames {
            match reply::decode(frame) {
                // The inner `Err` is a refusal the envelope faithfully
                // carried — the `refusal` shape's whole content, and a read.
                Ok(_) => read += 1,
                Err(e) => refused_by_name(shape, &e),
            }
        }
        held(shape, *expect, read, frames.len());
    }
}

/// A refusal must **locate** the shape it refused. A sentence that does not
/// name it is indistinguishable from a decoder that fell over somewhere else,
/// and "skipping a fixture is a decision recorded in the client" is only true
/// if the client can say which fixture.
fn refused_by_name(shape: &str, sentence: &str) {
    assert!(
        sentence.contains(shape),
        "{shape}: refused without naming itself: {sentence}"
    );
}

/// The decision, checked against what happened.
fn held(shape: &str, expect: Expect, read: usize, total: usize) {
    let wanted = match expect {
        Expect::Reads => total,
        Expect::Refuses(_) => 0,
        Expect::Partial { reads, .. } => reads,
    };
    assert_eq!(
        read,
        wanted,
        "{shape}: {read} of {total} frames read, the recorded decision says {wanted} ({}) \
         — decide it again in requests.rs/replies.rs rather than moving the count",
        expect.reason()
    );
}
