//! **The vendored wire version's one file-shaped home, compiled in**
//! (bl-6fec).
//!
//! The repo-root `PROTOCOL` file states the integer this build speaks — one
//! line, nothing else — and this script turns it into the constant
//! `src/hello.rs` includes. The file IS the source: there is no second
//! copy of the number to hold equal to it.
//!
//! **Why a file and not a Rust declaration.** This number is fetched by gates
//! that do not build this crate — yog's release gate reads it off this
//! repository's `main` before it will publish a bump (yog `docs/REMOTE.md`
//! §3), and this repository's own gate reads the engine's the same way — and a
//! Rust path is not a stable address for a fetch. yog split
//! `src/wire/hello.rs` into `src/wire/hello/version.rs` and left the old path
//! re-exporting: invisible to a build, unreadable to a regex, so this
//! repository's gate silently stopped being able to read the engine at all and
//! would have held its own release forever on the bump it was waiting for. A
//! top-level file with no extension is the one path a module split cannot
//! move, and every gate in all four repositories now reads it there.
//!
//! Errors here fail the build, which is the correct loudness: a `PROTOCOL`
//! file that is missing or is not one integer means nothing downstream can be
//! trusted to have read it either.

use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

/// Read the root `PROTOCOL` file and the vendored corpus, and write the two
/// generated files into `OUT_DIR`.
fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed=PROTOCOL");
    println!("cargo::rerun-if-changed=corpus/shapes.json");
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let out = PathBuf::from(env::var("OUT_DIR")?);
    let stated = fs::read_to_string(root.join("PROTOCOL"))?;
    let protocol: u32 = stated.trim().parse().map_err(|_| {
        format!("the repo-root PROTOCOL file must state one integer; it states {stated:?}")
    })?;
    fs::write(
        out.join("protocol.rs"),
        format!(
            "/// The integer the repo-root `PROTOCOL` file states, compiled in by\n\
             /// `build.rs`. Re-exported, with the reasoning, as the module's\n\
             /// `PROTOCOL`.\n\
             pub const PROTOCOL: u32 = {protocol};\n"
        ),
    )?;
    fs::write(out.join("ledger.rs"), ledger(&root)?)?;
    Ok(())
}

/// **The vendored corpus's editions, compiled in** (yog REMOTE §3.2, bl-e598).
///
/// `PROTOCOL` is a major and every ADDITION inside one is stamped with an
/// edition per field path in `corpus/shapes.json`. Three facts come out of that
/// file and none of them is written a second time anywhere:
///
/// * `FLOOR` — the edition the current major was cut at. Every path stamped at
///   or below it is on every engine of this major, so nothing about it is worth
///   asking.
/// * `EDITION` — the newest stamp in the vendored corpus, which is what this
///   build states in its own §3 preface. Computed, never stored: a corpus that
///   grows a field says so by carrying its stamp.
/// * `STAMPS` — the post-floor paths only, `(shape, path, edition)`. Everything
///   at or below the floor is left out because it can never answer anything
///   but *yes*, which is also why this table is EMPTY at the major's own cut.
///
/// A path may be stamped once per JSON type it takes (`…:null` beside
/// `…:string`). The path EXISTS from the earliest of those, so the minimum is
/// what is kept.
fn ledger(root: &Path) -> Result<String, Box<dyn Error>> {
    let text = fs::read_to_string(root.join("corpus/shapes.json"))?;
    let record: serde_json::Value = serde_json::from_str(&text)?;
    let floor = record
        .get("floor")
        .and_then(serde_json::Value::as_u64)
        .ok_or("corpus/shapes.json states no \"floor\"")?;
    let mut stamps: BTreeMap<(String, String), u64> = BTreeMap::new();
    let shapes = record
        .get("shapes")
        .and_then(serde_json::Value::as_object)
        .ok_or("corpus/shapes.json states no \"shapes\"")?;
    for (shape, said) in shapes {
        let signature = said
            .get("signature")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| format!("{shape}: no \"signature\" object"))?;
        for (typed, stamp) in signature {
            let stamp = stamp
                .as_u64()
                .ok_or_else(|| format!("{shape}/{typed}: the stamp is not an edition"))?;
            let path = typed.rsplit_once(':').map_or(typed.as_str(), |(p, _)| p);
            let seat = stamps.entry((shape.clone(), path.to_owned()));
            seat.and_modify(|held| *held = (*held).min(stamp))
                .or_insert(stamp);
        }
    }
    let edition = stamps.values().copied().max().unwrap_or(floor);
    let mut rows = String::new();
    for ((shape, path), stamp) in stamps.iter().filter(|(_, stamp)| **stamp > floor) {
        writeln!(rows, "    ({shape:?}, {path:?}, {stamp}),")?;
    }
    Ok(format!(
        "/// The edition this major was cut at (`corpus/shapes.json`).\n\
         pub const FLOOR: u32 = {floor};\n\
         /// The newest stamp in the vendored corpus — this build's edition.\n\
         pub const EDITION: u32 = {edition};\n\
         /// Every POST-floor field path and the edition it appeared at.\n\
         pub(crate) const STAMPS: &[(&str, &str, u32)] = &[\n{rows}];\n"
    ))
}
