//! **The stale-APK guard, both directions** (bl-c3fc).
//!
//! `scripts/screens-freshness.sh` is the one preflight beat of the render-and-
//! see loop that needs no emulator: a git tree, two mtimes and a sentence. So
//! it is the one this suite can drive, and it drives every arm — the warning
//! fires when the artifact is older than the tracked source, it stays quiet
//! when it is not, and neither an untracked file nor a docs edit can make it
//! speak. A guard that fires on everything is ignored, and a guard that fires
//! on nothing passes green forever; both are the defect this ball is about.
//!
//! Every case builds its own throwaway repository, because the question is
//! about a tree's index and this suite's own checkout is not a fixture.

use std::path::{Path, PathBuf};
use std::process::Command;

/// A run of the guard: its exit code and everything it said.
struct Said {
    code: i32,
    err: String,
}

impl Said {
    fn warns(&self) -> bool {
        self.err.contains("OLDER than the source")
    }
}

/// A repository with a tracked file under each scanned root, an APK, and
/// mtimes this fixture sets rather than inherits.
struct Tree {
    root: PathBuf,
}

impl Tree {
    fn new(name: &str) -> Result<Self, String> {
        let root = std::env::temp_dir().join(format!("yog-freshness-{name}"));
        drop(std::fs::remove_dir_all(&root));
        let tree = Self { root };
        tree.git(&["init", "-q"])?;
        tree.write("src/lib.rs", "// source\n")?;
        tree.write("android/app/build.gradle", "// gradle\n")?;
        tree.write("docs/DESIGN.md", "prose\n")?;
        tree.write("app-debug.apk", "not really an apk\n")?;
        tree.git(&["add", "src", "android", "docs"])?;
        Ok(tree)
    }

    fn write(&self, at: &str, body: &str) -> Result<(), String> {
        let path = self.root.join(at);
        let parent = path.parent().ok_or_else(|| format!("{at}: no parent"))?;
        std::fs::create_dir_all(parent).map_err(|why| format!("{at}: {why}"))?;
        std::fs::write(&path, body).map_err(|why| format!("{at}: {why}"))
    }

    /// The one knob every arm turns: an mtime, stated rather than raced for.
    fn dated(&self, at: &str, stamp: &str) -> Result<&Self, String> {
        run(
            "touch",
            &["-d", stamp, &self.root.join(at).display().to_string()],
            &self.root,
        )
        .map(|_| self)
    }

    fn git(&self, args: &[&str]) -> Result<String, String> {
        std::fs::create_dir_all(&self.root).map_err(|why| format!("mkdir: {why}"))?;
        run("git", args, &self.root)
    }

    fn judge(&self, apk: &str) -> Result<Said, String> {
        let guard = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/screens-freshness.sh");
        let out = Command::new(&guard)
            .current_dir(&self.root)
            .arg(apk)
            .output()
            .map_err(|why| format!("spawning the guard: {why}"))?;
        Ok(Said {
            code: out.status.code().unwrap_or(-1),
            err: String::from_utf8_lossy(&out.stderr).into_owned(),
        })
    }
}

fn run(tool: &str, args: &[&str], at: &Path) -> Result<String, String> {
    let out = Command::new(tool)
        .args(args)
        .current_dir(at)
        .output()
        .map_err(|why| format!("{tool}: {why}"))?;
    if !out.status.success() {
        return Err(format!(
            "{tool} {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// The direction the ball is named for: source edited after the build.
#[test]
fn an_apk_older_than_the_source_is_announced() -> Result<(), String> {
    let tree = Tree::new("stale")?;
    tree.dated("android/app/build.gradle", "2026-09-01 10:00:00 UTC")?
        .dated("app-debug.apk", "2026-09-01 11:00:00 UTC")?
        .dated("src/lib.rs", "2026-09-02 10:00:00 UTC")?;
    let said = tree.judge("app-debug.apk")?;
    assert_eq!(said.code, 0, "it refused: {}", said.err);
    assert!(said.warns(), "it said nothing: {}", said.err);
    assert!(said.err.contains("src/lib.rs"), "no culprit: {}", said.err);
    assert!(
        said.err.contains("make apk ABIS=x86_64"),
        "no fix named: {}",
        said.err
    );
    Ok(())
}

/// The other root, so the scan is not one directory wearing two names.
#[test]
fn the_gradle_half_of_the_tree_counts_too() -> Result<(), String> {
    let tree = Tree::new("stale-android")?;
    tree.dated("src/lib.rs", "2026-09-01 10:00:00 UTC")?
        .dated("app-debug.apk", "2026-09-01 11:00:00 UTC")?
        .dated("android/app/build.gradle", "2026-09-02 10:00:00 UTC")?;
    let said = tree.judge("app-debug.apk")?;
    assert!(said.warns(), "it said nothing: {}", said.err);
    assert!(
        said.err.contains("android/app/build.gradle"),
        "no culprit: {}",
        said.err
    );
    Ok(())
}

/// The quiet direction, and it must be genuinely quiet: a guard that warns
/// about every build is one nobody reads.
#[test]
fn an_apk_newer_than_the_source_passes_quietly() -> Result<(), String> {
    let tree = Tree::new("fresh")?;
    tree.dated("src/lib.rs", "2026-09-01 10:00:00 UTC")?
        .dated("android/app/build.gradle", "2026-09-01 10:00:00 UTC")?
        .dated("app-debug.apk", "2026-09-02 10:00:00 UTC")?;
    let said = tree.judge("app-debug.apk")?;
    assert_eq!(said.code, 0, "it refused: {}", said.err);
    assert!(!said.warns(), "it warned anyway: {}", said.err);
    Ok(())
}

/// Older is the test, not different: a build and the edit that went into it
/// can share a second, and the walk is fine.
#[test]
fn an_apk_the_same_age_as_the_source_is_not_stale() -> Result<(), String> {
    let tree = Tree::new("same")?;
    tree.dated("src/lib.rs", "2026-09-02 10:00:00 UTC")?
        .dated("android/app/build.gradle", "2026-09-02 10:00:00 UTC")?
        .dated("app-debug.apk", "2026-09-02 10:00:00 UTC")?;
    let said = tree.judge("app-debug.apk")?;
    assert!(!said.warns(), "it warned anyway: {}", said.err);
    Ok(())
}

/// Scoped on purpose: prose does not change what the APK paints.
#[test]
fn a_docs_edit_does_not_make_an_apk_stale() -> Result<(), String> {
    let tree = Tree::new("docs")?;
    tree.dated("src/lib.rs", "2026-09-01 10:00:00 UTC")?
        .dated("android/app/build.gradle", "2026-09-01 10:00:00 UTC")?
        .dated("app-debug.apk", "2026-09-02 10:00:00 UTC")?
        .dated("docs/DESIGN.md", "2026-09-03 10:00:00 UTC")?;
    let said = tree.judge("app-debug.apk")?;
    assert!(!said.warns(), "it warned anyway: {}", said.err);
    Ok(())
}

/// Tracked-only is load-bearing rather than tidy: the APK is BUILT under
/// `android/`, so a worktree walk would compare every fresh build against its
/// own build tree and call it stale.
#[test]
fn the_build_tree_beside_the_apk_is_not_source() -> Result<(), String> {
    let tree = Tree::new("untracked")?;
    tree.dated("src/lib.rs", "2026-09-01 10:00:00 UTC")?
        .dated("android/app/build.gradle", "2026-09-01 10:00:00 UTC")?
        .dated("app-debug.apk", "2026-09-02 10:00:00 UTC")?;
    tree.write("android/app/build/intermediates/whatever", "byproduct\n")?;
    tree.dated(
        "android/app/build/intermediates/whatever",
        "2026-09-03 10:00:00 UTC",
    )?;
    let said = tree.judge("app-debug.apk")?;
    assert!(!said.warns(), "it warned anyway: {}", said.err);
    Ok(())
}

/// The empty-set guard, `make line-cap`'s own: a scan that enumerates nothing
/// must fail loudly, never pass as "not stale" forever.
#[test]
fn a_scan_that_enumerates_nothing_refuses() -> Result<(), String> {
    let tree = Tree::new("empty")?;
    tree.git(&["rm", "-q", "-r", "--cached", "src", "android"])?;
    let said = tree.judge("app-debug.apk")?;
    assert_eq!(said.code, 2, "it passed: {}", said.err);
    assert!(
        said.err.contains("the scan is broken, not the tree"),
        "wrong refusal: {}",
        said.err
    );
    Ok(())
}

/// Called with no artifact, or one that is not there: a harness bug, and the
/// APK's own existence is `screens.sh`'s check rather than this one's.
#[test]
fn an_apk_that_is_not_there_is_a_harness_bug() -> Result<(), String> {
    let tree = Tree::new("missing")?;
    let said = tree.judge("nowhere.apk")?;
    assert_eq!(said.code, 2, "it passed: {}", said.err);
    assert!(
        said.err.contains("no APK at"),
        "wrong refusal: {}",
        said.err
    );

    let bare = Command::new(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/screens-freshness.sh"),
    )
    .current_dir(&tree.root)
    .output()
    .map_err(|why| format!("spawning the guard: {why}"))?;
    assert_eq!(bare.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&bare.stderr).contains("usage:"));
    Ok(())
}
