//! **What the target resolves before it builds** — the SDK it needs stated
//! for it, and the gradle it is expected to find in one of the two places a
//! working one lives on a real box. Nothing here reaches a device: an arm
//! that got as far as `adb` has already failed the thing it tests.
//!
//! **The gradle arms drive `scripts/gradle.sh` through its first caller**
//! (bl-2a31). That rule has one home now, because it used to have two and
//! they disagreed: `make apk` defaulted `GRADLE ?= gradle` and died with
//! `gradle: not found` on the very box this script built on. The behavioural
//! half of "one resolution" is every arm below, which all run through the
//! shared script; the structural half is the last test, which asserts the
//! other caller reaches it and spells no probe of its own — the two callers
//! cannot share a fixture, since `make apk` runs cargo-ndk and gradle for
//! real.

use crate::fakes::{ADDR, Fixture, refused, spent};

#[test]
fn no_address_is_a_usage_error_naming_the_make_invocation() -> Result<(), String> {
    let run = Fixture::new("no-addr")?.run(&[])?;
    refused(&run, 2, "usage: make deploy-phone ADDR=<ip:port>");
    assert!(spent(&run).is_empty(), "it spent something: {}", run.log);
    Ok(())
}

#[test]
fn a_missing_adb_names_android_home_and_platform_tools() -> Result<(), String> {
    let run = Fixture::new("no-adb")?.run(&[ADDR])?;
    refused(&run, 1, "no adb at ");
    refused(&run, 1, "set ANDROID_HOME, or install platform-tools");
    Ok(())
}

#[test]
fn no_gradle_anywhere_names_both_probes_and_builds_nothing() -> Result<(), String> {
    let run = Fixture::new("no-gradle")?.with_adb()?.run(&[ADDR])?;
    refused(&run, 1, "'gradle' is not on PATH");
    refused(&run, 1, "wrapper/dists");
    refused(&run, 1, "GRADLE=/path/to/gradle");
    assert!(spent(&run).is_empty(), "it built anyway: {}", run.log);
    Ok(())
}

#[test]
fn the_wrapper_cache_is_the_second_probe_and_the_newest_wins() -> Result<(), String> {
    let run = Fixture::new("dists")?
        .with_adb()?
        .with_dist("8.7")?
        .with_dist("8.11.1")?
        .run(&[ADDR])?;
    assert_eq!(run.code, 0, "it said: {}", run.err);
    let built = spent(&run)[0];
    assert!(
        built.contains("gradle-8.11.1/bin/gradle"),
        "not the newest distribution: {built}"
    );
    Ok(())
}

#[test]
fn a_gradle_on_path_outranks_the_wrapper_cache() -> Result<(), String> {
    let fixture = Fixture::new("path-gradle")?
        .with_adb()?
        .with_path_gradle()?
        .with_dist("8.11.1")?;
    let run = fixture.run(&[ADDR])?;
    assert_eq!(run.code, 0, "it said: {}", run.err);
    let built = spent(&run)[0];
    assert!(
        built.contains(&format!("GRADLE={}/bin/gradle", fixture.root.display())),
        "PATH did not win: {built}"
    );
    Ok(())
}

#[test]
fn an_explicit_gradle_outranks_both_probes() -> Result<(), String> {
    let fixture = Fixture::new("named-gradle")?
        .with_adb()?
        .with_named_gradle()?
        .with_path_gradle()?
        .with_dist("8.11.1")?;
    let run = fixture.run(&[ADDR])?;
    assert_eq!(run.code, 0, "it said: {}", run.err);
    let built = spent(&run)[0];
    assert!(
        built.contains(&format!(
            "GRADLE={}/elsewhere/gradle",
            fixture.root.display()
        )),
        "the named gradle did not win: {built}"
    );
    Ok(())
}

/// **The other caller reaches the same script, and states no rule of its own**
/// (bl-2a31). `make apk` is the target a fresh checkout runs first and the
/// one `deploy-phone` builds through, so a second resolution here is a build
/// that fails after the slow half on a box where the deploy would have
/// worked. Read off the Makefile because the recipe itself cannot be driven
/// without an NDK and a gradle; what it proves is that there is nothing to
/// drive — no second probe exists to disagree.
#[test]
fn the_apk_target_resolves_gradle_through_the_one_script() -> Result<(), String> {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assert!(
        root.join("scripts/gradle.sh").is_file(),
        "no scripts/gradle.sh"
    );
    let makefile = std::fs::read_to_string(root.join("Makefile"))
        .map_err(|why| format!("reading the Makefile: {why}"))?;
    // The canned recipe both APK targets spend (`define assemble`, bl-7a68):
    // one Gradle line, parameterized by variant, so the debug build and the
    // release-signed one cannot resolve a different gradle from each other
    // either.
    let assembles: Vec<&str> = makefile
        .lines()
        .filter(|line| line.contains("\"$$gradle\" assemble"))
        .collect();
    assert_eq!(
        assembles.len(),
        1,
        "the Gradle assemble is spelled {} times; it must have one home",
        assembles.len()
    );
    for assemble in &assembles {
        assert!(
            assemble.contains("scripts/gradle.sh"),
            "the apk recipe names a gradle of its own: {assemble}"
        );
    }
    for spelled in ["wrapper/dists", "GRADLE ?=", "GRADLE :=", "GRADLE="] {
        assert!(
            !makefile.contains(&format!("\n{spelled}")),
            "the Makefile carries a second gradle rule: {spelled}"
        );
    }
    Ok(())
}
