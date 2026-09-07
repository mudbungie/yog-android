//! **What the target does with the device** — and with a build that never
//! produced an APK to give it. Every arm here is one way a deploy can print
//! something reassuring and have installed nothing, which is why the exit
//! code and not a message is what this target answers with.

use crate::fakes::{ADDR, Fixture, refused, spent};

#[test]
fn a_failed_build_installs_nothing() -> Result<(), String> {
    let run = Fixture::new("build-fails")?
        .with_adb()?
        .with_path_gradle()?
        .set("MAKE_CODE", "1")
        .run(&[ADDR])?;
    refused(&run, 1, "the build failed — nothing was installed");
    assert!(!run.log.contains("adb"), "it reached adb: {}", run.log);
    Ok(())
}

#[test]
fn a_build_that_produced_no_apk_is_a_failure() -> Result<(), String> {
    let run = Fixture::new("no-apk")?
        .with_adb()?
        .with_path_gradle()?
        .set("MAKE_APK_TOUCH", "")
        .run(&[ADDR])?;
    refused(&run, 1, "there is no APK at ");
    assert!(!run.log.contains("adb"), "it reached adb: {}", run.log);
    Ok(())
}

#[test]
fn a_refused_connect_stops_before_the_install() -> Result<(), String> {
    let run = Fixture::new("connect-refused")?
        .with_adb()?
        .with_path_gradle()?
        .set("ADB_CONNECT_SAY", "failed to connect to the device")
        .set("ADB_CONNECT_CODE", "1")
        .run(&[ADDR])?;
    refused(
        &run,
        1,
        "adb connect failed: failed to connect to the device",
    );
    assert!(!run.log.contains("install"), "it installed: {}", run.log);
    Ok(())
}

/// The same refusal from the OTHER adb: older builds answer a failed connect
/// with exit 0 and a sentence. The message is read either way, so the exit
/// code is not the only thing standing between a dead phone and a "deployed".
#[test]
fn a_connect_that_exited_zero_without_connecting_is_still_fatal() -> Result<(), String> {
    let run = Fixture::new("connect-quiet")?
        .with_adb()?
        .with_path_gradle()?
        .set("ADB_CONNECT_SAY", "failed to connect to the device")
        .run(&[ADDR])?;
    refused(&run, 1, "adb connect did not connect");
    assert!(!run.log.contains("install"), "it installed: {}", run.log);
    Ok(())
}

/// The one every second run hits: a device already on the list. `adb connect`
/// says so and exits 0, and that is a success.
#[test]
fn already_connected_is_not_an_error() -> Result<(), String> {
    let run = Fixture::new("already")?
        .with_adb()?
        .with_path_gradle()?
        .set("ADB_CONNECT_SAY", "already connected to the device")
        .run(&[ADDR])?;
    assert_eq!(run.code, 0, "it said: {}", run.err);
    assert!(
        run.log.contains("install -r"),
        "it did not install: {}",
        run.log
    );
    Ok(())
}

#[test]
fn a_failed_install_is_a_failure() -> Result<(), String> {
    let run = Fixture::new("install-fails")?
        .with_adb()?
        .with_path_gradle()?
        .set(
            "ADB_INSTALL_SAY",
            "adb: failed to install [INSTALL_FAILED_INSUFFICIENT_STORAGE]",
        )
        .set("ADB_INSTALL_CODE", "1")
        .run(&[ADDR])?;
    refused(&run, 1, "install failed: adb: failed to install");
    Ok(())
}

/// **The signer clash earns its own sentence** (bl-7a68). Android identifies
/// an app by its signer as well as its package, so a release-signed APK over
/// a debug-signed one is refused rather than installed — and the remedy is an
/// uninstall that takes the device's enrolment with it, which is an act only
/// an operator may perform. It is judged BEFORE the exit code because adb
/// reports it both ways, and a run that read the exit code first would answer
/// the generic sentence for the one failure with a specific act behind it.
#[test]
fn a_signer_clash_names_the_one_time_uninstall() -> Result<(), String> {
    for said in [
        "adb: failed to install [INSTALL_FAILED_UPDATE_INCOMPATIBLE]",
        "Failure [INSTALL_FAILED_UPDATE_INCOMPATIBLE: signatures do not match]",
    ] {
        let run = Fixture::new("signer-clash")?
            .with_adb()?
            .with_path_gradle()?
            .set("ADB_INSTALL_SAY", said)
            .set("ADB_INSTALL_CODE", "1")
            .run(&[ADDR])?;
        refused(&run, 1, "signed by another key");
        refused(&run, 1, "uninstall dev.yog on the phone");
    }
    Ok(())
}

/// A signer clash that adb reported with a ZERO exit — the same shape the
/// connect arms above cover, and the reason the message is read either way.
#[test]
fn a_quiet_signer_clash_is_still_the_uninstall_sentence() -> Result<(), String> {
    let run = Fixture::new("signer-quiet")?
        .with_adb()?
        .with_path_gradle()?
        .set(
            "ADB_INSTALL_SAY",
            "signatures do not match previously installed",
        )
        .run(&[ADDR])?;
    refused(&run, 1, "signed by another key");
    Ok(())
}

/// **The key decides which build is deployed** (bl-7a68, DESIGN §20). A box
/// holding this app's permanent signing key builds and installs the
/// release-signed APK, so a phone deployed by hand is on the same signature as
/// one that took an update off the release channel — a phone left on the debug
/// key can never take one. A box without it builds the debug APK exactly as
/// before: removing the key deletes a default and edits no script.
///
/// **Two assertions and deliberately not a third.** What the run BUILT is read
/// off the tool log, and which artifact it then looked for is read out of the
/// sentence naming it — but never the exit code, because whether that path
/// holds an APK is a fact about the box: a checkout that has run `make apk`
/// installs and exits 0, and a clean one refuses. The name is in the run's own
/// words either way, which is the half this pair is about.
fn which_build(name: &str, key: bool) -> Result<(), String> {
    let fixture = Fixture::new(name)?.with_adb()?.with_path_gradle()?;
    let fixture = if key {
        let at = fixture.root.join("release.keystore");
        std::fs::write(&at, "the probe looks for a file, and this is one")
            .map_err(|why| format!("{}: {why}", at.display()))?;
        fixture.set("YOG_KEYSTORE", &at.display().to_string())
    } else {
        fixture
    };
    // Both emptied so the script's own defaults show: what it builds, and
    // where it then looks for what it built.
    let run = fixture
        .set("APK", "")
        .set("MAKE_APK_TOUCH", "")
        .run(&[ADDR])?;
    let (target, artifact) = if key {
        (
            "make apk-release ABIS=arm64-v8a",
            "apk/release/app-release-signed.apk",
        )
    } else {
        ("make apk ABIS=arm64-v8a", "apk/debug/app-debug.apk")
    };
    // `first()` and not `[0]`: this helper is not itself a `#[test]`, so
    // clippy's in-test carve-out for indexing does not reach it, and the
    // manifest denies unchecked indexing everywhere else.
    let built = spent(&run).first().copied().unwrap_or_default();
    assert!(
        built.starts_with(target),
        "expected {target:?}, got {built:?}"
    );
    assert!(
        run.err.contains(artifact),
        "the run never named {artifact}: {}",
        run.err
    );
    Ok(())
}

#[test]
fn the_signing_key_decides_which_build_is_deployed() -> Result<(), String> {
    which_build("release-key", true)
}

#[test]
fn a_box_without_the_key_builds_the_debug_apk() -> Result<(), String> {
    which_build("no-key", false)
}
