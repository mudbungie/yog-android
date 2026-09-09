//! **The vendored corpus's stamps, read as data** (yog REMOTE §3.2, bl-e598).
//!
//! `corpus/shapes.json` stamps every field PATH with the edition it appeared
//! at, `floor` is the edition the current major was cut at, and the corpus's
//! own edition is the newest stamp — *computed and never stored*, so this
//! reader computes it too rather than trusting a field.
//!
//! A path is stamped once per JSON type it takes (`…:null` beside
//! `…:string`). The path EXISTS from the earliest of those, so the minimum is
//! what is kept — the same reduction `build.rs` makes for the compiled-in
//! table, said here in test code because a test that shared the build script's
//! answer would prove only that one function agrees with itself.

use serde_json::Value;
use std::collections::BTreeMap;

/// One shape's stamps: field path (no type suffix) to the edition it appeared
/// at, and the paths whose value is a string in at least one of its types.
#[derive(Clone)]
pub struct Shape {
    pub stamps: BTreeMap<String, u64>,
    pub strings: Vec<String>,
}

/// The whole ledger.
pub struct Ledger {
    pub floor: u64,
    pub edition: u64,
    pub shapes: BTreeMap<String, Shape>,
}

/// Read it, or say why it will not read. Every failure comes back as an `Err`
/// so the helpers stay total functions — the panic vocabulary belongs to the
/// `#[test]` items.
pub fn read(text: &str) -> Result<Ledger, String> {
    let record: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let floor = record
        .get("floor")
        .and_then(Value::as_u64)
        .ok_or("corpus/shapes.json states no \"floor\"")?;
    let said = record
        .get("shapes")
        .and_then(Value::as_object)
        .ok_or("corpus/shapes.json states no \"shapes\"")?;
    let mut shapes = BTreeMap::new();
    let mut edition = floor;
    for (shape, held) in said {
        let signature = held
            .get("signature")
            .and_then(Value::as_object)
            .ok_or_else(|| format!("{shape}: no \"signature\" object"))?;
        let mut stamps: BTreeMap<String, u64> = BTreeMap::new();
        let mut strings = Vec::new();
        for (typed, stamp) in signature {
            let stamp = stamp
                .as_u64()
                .ok_or_else(|| format!("{shape}/{typed}: the stamp is not an edition"))?;
            let (path, kind) = typed
                .rsplit_once(':')
                .ok_or_else(|| format!("{shape}/{typed}: no type on the path"))?;
            edition = edition.max(stamp);
            stamps
                .entry(path.to_owned())
                .and_modify(|held| *held = (*held).min(stamp))
                .or_insert(stamp);
            if kind == "string" {
                strings.push(path.to_owned());
            }
        }
        strings.sort();
        strings.dedup();
        shapes.insert(shape.clone(), Shape { stamps, strings });
    }
    Ok(Ledger {
        floor,
        edition,
        shapes,
    })
}

/// **Project one frame back to edition `e`**: delete every key whose path is
/// stamped above it. A path the ledger does not name is left alone — it is a
/// key no signature records, and inventing a stamp for it would be this
/// reader deciding what the corpus says.
pub fn project(here: &str, v: &mut Value, stamps: &BTreeMap<String, u64>, e: u64) {
    match v {
        Value::Object(map) => {
            map.retain(|key, _| {
                stamps
                    .get(&format!("{here}/{key}"))
                    .is_none_or(|stamp| *stamp <= e)
            });
            for (key, child) in map.iter_mut() {
                project(&format!("{here}/{key}"), child, stamps, e);
            }
        }
        Value::Array(items) => {
            let under = format!("{here}/[]");
            for child in items.iter_mut() {
                project(&under, child, stamps, e);
            }
        }
        _ => {}
    }
}

/// **Replace every string at one path with a token no build has heard of.**
/// The answer is whether anything was replaced, so a path that reached nothing
/// is a mutation that proved nothing and the caller can say so.
pub fn mutate(path: &str, here: &str, v: &mut Value, token: &str) -> bool {
    if here == path && v.is_string() {
        *v = Value::String(token.to_owned());
        return true;
    }
    // A loop and not `any`, which short-circuits: every string at the path is
    // replaced, because a frame with one occurrence left unmutated would be a
    // frame this replay never actually stressed.
    let mut reached = false;
    match v {
        Value::Object(map) => {
            for (key, child) in map.iter_mut() {
                reached |= mutate(path, &format!("{here}/{key}"), child, token);
            }
        }
        Value::Array(items) => {
            let under = format!("{here}/[]");
            for child in items.iter_mut() {
                reached |= mutate(path, &under, child, token);
            }
        }
        _ => {}
    }
    reached
}
