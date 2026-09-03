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
            "adb: failed to install [INSTALL_FAILED_UPDATE_INCOMPATIBLE]",
        )
        .set("ADB_INSTALL_CODE", "1")
        .run(&[ADDR])?;
    refused(&run, 1, "install failed: adb: failed to install");
    Ok(())
}

/// The arm the ball asked for by name: `Success` is what proves an install,
/// not a zero exit. An adb that streamed the APK and said nothing else has not
/// installed anything.
#[test]
fn an_install_that_never_said_success_is_a_failure() -> Result<(), String> {
    let run = Fixture::new("no-success")?
        .with_adb()?
        .with_path_gradle()?
        .set("ADB_INSTALL_SAY", "Performing Streamed Install")
        .run(&[ADDR])?;
    refused(&run, 1, "the install did not answer Success");
    Ok(())
}

#[test]
fn the_whole_act_is_build_then_connect_then_install() -> Result<(), String> {
    let fixture = Fixture::new("happy")?.with_adb()?.with_path_gradle()?;
    let run = fixture.run(&[ADDR])?;
    assert_eq!(run.code, 0, "it said: {}", run.err);
    let apk = fixture.root.join("app-debug.apk");
    let expected = [
        "make apk ABIS=arm64-v8a GRADLE=".to_owned(),
        format!("adb connect {ADDR}"),
        format!("adb -s {ADDR} install -r {}", apk.display()),
    ];
    let done = spent(&run);
    assert_eq!(done.len(), expected.len(), "spent: {}", run.log);
    for (did, want) in done.iter().zip(expected.iter()) {
        assert!(did.starts_with(want), "expected {want:?}, got {did:?}");
    }
    Ok(())
}
