//! **The invocation, end to end, on a device** (bl-05b6): the driver half.
//!
//! Every tool this client advertises is host-tested, and no test in this repo
//! had ever put an invocation through the tool-host channel to a device and
//! read the capture back. The half that gap covers is exactly the half a host
//! test cannot reach — an activity launch the platform refuses for being in
//! the background, a notification that really is in the shade, a battery
//! figure that is this device's.
//!
//! **This file dials; `scripts/invoke.sh` provides the world.** The loop out
//! there boots an emulator and a bounded engine, mints a foot leaf, seeds the
//! device with it and judges the platform afterwards; in here is the one thing
//! a shell cannot do — speak the wire. It is `#[ignore]`d for `tests/parity.rs`'s
//! reason exactly: there is nothing to dial until a device has been driven,
//! and a host `cargo test` that answered this question would be answering
//! about a world that does not exist.
//!
//! **`yog gesture` will not do, and that is the ball's finding.** The
//! invocation mailbox is per-process in-memory state, so an `/invoke` run in a
//! second process addresses a different mailbox from the one the phone's
//! parked `invocations` read is waiting on. The gesture has to cross the
//! listener, which means a seat.
//!
//! **The envelopes are written by hand, and that is not laziness.** This
//! crate's codec refuses `invoke` and `capture` by name (REMOTE §4.2: *"a foot
//! is invoked; it never invokes"*), so the asking side has no spelling here to
//! borrow — `tests/conformance/requests.rs` records exactly that decision.
//! `Seat::ask` hands back the reply frames as JSON, which is the whole of what
//! a driver needs and adds nothing to the product's vocabulary.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use yog_android::material::Material;
use yog_android::transport::Seat;

/// How long the device gets to connect, advertise and answer. Generous
/// because the far end is an emulator booting an app; bounded because a
/// harness that waits forever is a harness nobody can put in a loop.
const PATIENCE: Duration = Duration::from_mins(2);

/// **Every helper here is total, and the panics live in the `#[test]` items.**
/// clippy judges a free function in an integration crate as production code
/// (`clippy.toml` records that surprise once); a helper the whole file leans
/// on is worth writing total anyway.
fn env(name: &str) -> Result<String, String> {
    std::env::var(name).map_err(|_| format!("{name} is not set — run this from make invoke"))
}

/// The seat this driver is: the fixture's own operator-grade client leaf.
fn seat() -> Result<Seat, String> {
    Seat::open(&Material {
        anchors: PathBuf::from(env("WIRE_ANCHORS")?),
        chain: PathBuf::from(env("WIRE_CHAIN")?),
        key: PathBuf::from(env("WIRE_KEY")?),
        address: env("WIRE_ADDRESS")?,
    })
}

/// One array field of a reply row, or nothing. An envelope this reader cannot
/// find its way through is an empty answer here and a timeout above, which is
/// the same sentence either way: the device never advertised.
fn field(of: &Value, name: &str) -> Vec<Value> {
    of.get(name)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

/// One gesture, one reply body. Every read here answers in one frame.
fn asked(seat: &Seat, request: &Value) -> Result<Value, String> {
    let frames = seat
        .ask(request)
        .map_err(|why| format!("{request}: {}", why.sentence()))?;
    frames
        .into_iter()
        .next_back()
        .ok_or_else(|| format!("{request}: the engine answered no frame at all"))
}

/// **Wait for the device to be a tool host**, which is the readiness this
/// harness actually depends on: not that the app launched, but that the
/// engine holds an advertised set under the foot's own name.
fn advertised(seat: &Seat, client: &str, workspace: &str) -> Result<Vec<String>, String> {
    let deadline = Instant::now() + PATIENCE;
    let mut last = Value::Null;
    while Instant::now() < deadline {
        last = asked(seat, &json!({ "op": "clients", "workspace": workspace }))?;
        let rows = field(&last, "rows");
        for row in rows {
            if row.get("client") != Some(&json!(client)) {
                continue;
            }
            let tools: Vec<String> = field(&row, "tools")
                .iter()
                .filter_map(|tool| tool.get("name").and_then(Value::as_str).map(str::to_owned))
                .collect();
            if !tools.is_empty() {
                return Ok(tools);
            }
        }
        std::thread::sleep(Duration::from_secs(2));
    }
    Err(format!(
        "{client} never advertised a tool; the roster said: {last}"
    ))
}

/// One invocation, waited out. `invoke` answers a handle at once — the
/// engine's intake is one thread for the whole world — so the wait is the
/// caller's, which is what `capture` is for.
fn invoked(seat: &Seat, client: &str, tool: &str, input: &Value) -> Result<Value, String> {
    let queued = asked(
        seat,
        &json!({ "op": "invoke", "client": client, "tool": tool, "input": input }),
    )?;
    let handle = queued
        .get("invocation")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{tool}: the engine answered no handle: {queued}"))?
        .to_owned();
    let deadline = Instant::now() + PATIENCE;
    while Instant::now() < deadline {
        let answer = asked(seat, &json!({ "op": "capture", "invocation": handle }))?;
        if let Some(capture) = answer.get("capture")
            && !capture.is_null()
        {
            return Ok(capture.clone());
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    Err(format!(
        "{tool}: the device never answered invocation {handle}"
    ))
}

/// The four tools this loop spends, and the input each is given.
fn calls(nonce: &str) -> Vec<(&'static str, Value)> {
    vec![
        ("shell", json!({ "command": format!("echo {nonce}") })),
        ("device", json!({})),
        (
            "notify",
            json!({ "title": nonce, "text": "from the invoke beat" }),
        ),
        ("open", json!({ "url": "https://example.invalid/" })),
    ]
}

/// **The device is a tool host**, which is the first thing this loop proves
/// and the thing everything below stands on: an advertisement crossed the
/// wire from a device, and the engine holds it under the foot's own name.
///
/// It is a beat of its own because of WHEN it has to happen. The foreground
/// service that keeps a pocketed foot's read open is armed by the activity's
/// own resume (DESIGN §18.1), so the app must be in front once before the
/// harness sends it to the background — and the honest signal that the app got
/// that far is the set, not a sleep.
#[test]
#[ignore = "needs the emulator, the engine and the foot leaf make invoke provides"]
fn the_device_advertises_the_set_this_build_offers() -> Result<(), String> {
    let offered = advertised(&seat()?, &env("FOOT_CLIENT")?, &env("FOOT_WORKSPACE")?)?;
    for (wanted, _) in calls("") {
        assert!(
            offered.iter().any(|name| name == wanted),
            "{wanted} is not in the advertised set: {offered:?}"
        );
    }
    println!("invoke: the device advertises {} tools", offered.len());
    Ok(())
}

/// **The four**, chosen so that each answers with something no host test could
/// have produced: a value this harness minted, a figure that is this device's,
/// a row in the platform's own shade, and a refusal the platform itself makes.
/// Each capture is written out whole for `scripts/invoke-judge.sh` to judge
/// against the device it came from.
#[test]
#[ignore = "needs the emulator, the engine and the foot leaf make invoke provides"]
fn four_invocations_reach_the_platform_and_their_captures_come_back() -> Result<(), String> {
    let seat = seat()?;
    let client = env("FOOT_CLIENT")?;
    let nonce = env("INVOKE_NONCE")?;
    let out = PathBuf::from(env("INVOKE_OUT")?);
    // The set again, and not for tidiness: this runs with the app in the
    // BACKGROUND, and a foot whose host did not survive being pocketed would
    // otherwise fail as a timed-out invocation rather than as what it is.
    advertised(&seat, &client, &env("FOOT_WORKSPACE")?)?;
    for (tool, input) in calls(&nonce) {
        let capture = invoked(&seat, &client, tool, &input)?;
        let body = serde_json::to_string_pretty(&capture).map_err(|why| why.to_string())?;
        std::fs::write(out.join(format!("{tool}.json")), &body)
            .map_err(|why| format!("{tool}: writing the capture: {why}"))?;
        let exit = capture.get("exit_code").cloned().unwrap_or(Value::Null);
        println!("invoke: {tool} answered exit {exit}");
        assert!(
            exit.is_number(),
            "{tool}: a capture states its own exit: {body}"
        );
    }
    Ok(())
}
