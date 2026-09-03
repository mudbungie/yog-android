//! The sighted pair's pure half: the advertised elements, what each one
//! promises the model about its own price, the lens and path reading, and the
//! sentence a build with no device under it gives. The platform calls are the
//! device's to answer for — everything up to them is asserted here.

use super::{absent, destination, run, tools};
use crate::tools::BAD_INPUT;
use crate::tools::bridged::REFUSED;
use serde_json::json;

fn args(v: serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
    v.as_object().cloned().unwrap_or_default()
}

#[test]
fn the_table_names_both_sighted_tools() {
    let set = tools();
    let names: Vec<&str> = set.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["camera", "location"]);
    for tool in &set {
        assert_eq!(tool.input_schema["type"], "object");
        assert!(!tool.description.is_empty(), "{} says nothing", tool.name);
    }
    // The lens is advertised as the two words the argument reading accepts:
    // one fact, and the schema is where a model meets it.
    assert_eq!(
        tools()[0].input_schema["properties"]["lens"]["enum"],
        json!(["back", "front"])
    );
}

/// **The containment rule, asserted** (DESIGN §6, §16.1), the shape the paper
/// tools made mechanical: the description is the one text a model reads before
/// it spends a call, so each tool's own price — the permission, the foreground
/// fact, the staleness a fix can carry — is written there and not only in the
/// refusal that comes back.
#[test]
fn every_tool_states_its_own_price_where_the_model_will_read_it() {
    let said = |name: &str| {
        tools()
            .into_iter()
            .find(|t| t.name == name)
            .map(|t| t.description)
            .unwrap_or_default()
    };
    // The still: the grant, the foreground refusal, and that a path is not an
    // image — the one thing a model would otherwise assume it could read.
    assert!(said("camera").contains("camera permission"));
    assert!(said("camera").contains("not in front"));
    assert!(said("camera").contains("not the image"));
    assert!(said("camera").contains("overwritten"));
    // The fix: the grant, the device switch, the age, and the background
    // grant this rung does not ask for.
    assert!(said("location").contains("location permission"));
    assert!(said("location").contains("location switch"));
    assert!(said("location").contains("HOW OLD IT IS"));
    assert!(said("location").contains("background-location grant"));
}

#[test]
fn a_lens_this_device_does_not_have_is_a_mis_call_named_in_the_models_terms() {
    for named in [json!("sideways"), json!(""), json!(3)] {
        let capture = run("camera", &args(json!({ "lens": named })), "/nonexistent");
        assert_eq!(capture.exit_code, BAD_INPUT, "for {named}");
        assert_eq!(capture.stderr, "\"lens\" is either \"back\" or \"front\"\n");
    }
}

/// Where a still lands, which is a design decision and therefore an
/// assertion: the app's own storage under one name, overwritten by the next
/// call, unless the caller said otherwise.
#[test]
fn a_still_with_no_path_lands_in_the_apps_own_storage_under_one_name() {
    let storage = "/data/user/0/dev.yog/files";
    assert_eq!(
        destination(&args(json!({})), storage),
        "/data/user/0/dev.yog/files/camera.jpg"
    );
    // Twice, because "the default is overwritten" is the description's own
    // promise and a name that moved between calls would break it.
    assert_eq!(
        destination(&args(json!({ "lens": "front" })), storage),
        destination(&args(json!({})), storage)
    );
    assert_eq!(
        destination(&args(json!({ "path": "/sdcard/kept.jpg" })), storage),
        "/sdcard/kept.jpg"
    );
    // A path that is not a string is not a path: the default stands rather
    // than the call failing, which is `arg`'s own reading everywhere else.
    assert_eq!(
        destination(&args(json!({ "path": 7 })), storage),
        "/data/user/0/dev.yog/files/camera.jpg"
    );
}

#[test]
fn a_build_with_no_android_says_so_for_both_sighted_tools() {
    // The host suite is such a build, so this is what the pair answers here —
    // and every dispatch arm is reached on the way: the defaulted lens and
    // path, a stated lens, a stated path, and the fix.
    for (tool, input) in [
        ("camera", json!({})),
        ("camera", json!({ "lens": "front" })),
        ("camera", json!({ "lens": "back", "path": "/tmp/shot.jpg" })),
        ("camera", json!({ "path": "/tmp/shot.jpg" })),
        ("location", json!({})),
    ] {
        let capture = run(tool, &args(input), "/nonexistent");
        assert_eq!(capture.exit_code, REFUSED, "for {tool}");
        assert_eq!(
            capture.stderr,
            "this build has no Android to look with: the tool exists only on the device\n",
            "for {tool}"
        );
    }
    assert!(absent().starts_with("err\n"));
}
