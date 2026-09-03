//! **The standing parity assertion** (yog `docs/PARITY.md` §5): the ops the
//! engine says every seat owes a control, against the `act:` tags a walk of
//! this app actually observed.
//!
//! **It is `#[ignore]`d, and that is the honest spelling.** The inventory does
//! not exist until a device has been driven: `make screens` boots a headless
//! emulator, walks the named screens and captures a `uiautomator` dump beside
//! each screenshot (DESIGN §15). A host `cargo test` has no dumps, and a test
//! that judged an empty inventory would answer a question nobody asked — every
//! control absent, on a tree where they are all present. So the walk runs it,
//! at the end of the walk, where the bytes are; `make screens` fails if it
//! fails, and `make parity` is the same assertion by hand over a run's output.
//!
//! What DOES gate on every `make check` is the half that needs no device:
//! `src/parity/tests.rs` reads this tree's own `parity.toml` against the
//! vendored roster, so an exemption that stops citing, stops parsing, or names
//! an op the engine no longer classes `control` reddens immediately.
//!
//! Two files reach it as text and one as a directory:
//!
//! - the roster and the exemptions are compiled in, because both are committed
//!   in this tree and reading them at runtime would only add a way to fail;
//! - the dumps come from `$PARITY_DUMPS` (default `target/screens`, where the
//!   walk writes). Every `*.ui.xml` in it is one screen's accessibility tree.

use std::path::{Path, PathBuf};

/// Where the walk left its dumps.
fn dumps() -> PathBuf {
    std::env::var("PARITY_DUMPS").map_or_else(|_| PathBuf::from("target/screens"), PathBuf::from)
}

/// Every inventory the walk captured, concatenated. Presence is the claim
/// (§5), so which screen a tag was found on does not enter the judgement —
/// that a walked screen carried it does.
///
/// **Two extensions, one scanner.** `.tags` is what the app wrote about what
/// it painted (PARITY §6's fallback, DESIGN §15.1) and `.ui.xml` is the
/// platform's accessibility dump, still captured because it is the instrument
/// the day upstream's android adapter stops aborting this app (bl-a6f3). The
/// scanner looks for the `act:` token and not for a format, so the same
/// judgement reads either, and the changeover is a deletion here rather than a
/// second gate.
///
/// Total, and answering in `Result`, because a free helper in a `tests/`
/// binary is production code to clippy (clippy.toml, bl-93e3): the panic
/// vocabulary belongs in the `#[test]` item below, where it reads as an
/// assertion.
fn inventory(from: &Path) -> Result<String, String> {
    let listing = std::fs::read_dir(from).map_err(|why| {
        format!(
            "no walk output at {}: {why} — run `make screens`",
            from.display()
        )
    })?;
    let mut names: Vec<PathBuf> = listing
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|ext| ext == "tags" || ext == "xml")
        })
        .collect();
    if names.is_empty() {
        return Err(format!(
            "{} holds no inventory — the walk captured neither a *.tags file nor a dump",
            from.display()
        ));
    }
    names.sort();
    Ok(names
        .iter()
        .filter_map(|path| std::fs::read_to_string(path).ok())
        .collect::<Vec<_>>()
        .join("\n"))
}

#[test]
#[ignore = "needs the accessibility dumps a `make screens` walk captures"]
fn every_control_op_is_reachable_or_cited() {
    let from = dumps();
    let inventory = inventory(&from).expect("the walk's accessibility dumps");
    let judged = yog_android::parity::judge(
        include_str!("../corpus/reply/help.json"),
        include_str!("../parity.toml"),
        &inventory,
    );
    // The report prints on every run, passing or failing: an absence is never
    // silent (PARITY §7), and the roster is what a reader came for.
    println!("{}", judged.report);
    assert!(
        judged.failures.is_empty(),
        "interface parity, judged over {}:\n{}",
        from.display(),
        judged.failures.join("\n")
    );
}
