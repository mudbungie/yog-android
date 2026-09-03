//! The shade tool's pure half: the advertised element, what it promises the
//! model about its own price and its own forgetfulness, the cap reading, and
//! the sentence a build with no device under it gives. The platform call is
//! the device's to answer for — everything up to it is asserted here.

use super::{SHOWN, absent, run, shown, tools};
use crate::tools::bridged::REFUSED;
use serde_json::json;

fn args(v: serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
    v.as_object().cloned().unwrap_or_default()
}

#[test]
fn the_table_names_one_read_only_shade_tool() {
    let set = tools();
    let names: Vec<&str> = set.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names, ["notifications"]);
    assert_eq!(set[0].input_schema["type"], "object");
    // The cap is advertised, because a caller that was capped must know how
    // to ask for the rest.
    assert_eq!(
        set[0].input_schema["properties"]["limit"]["type"],
        "integer"
    );
    // Nothing is required: the shade read a caller meant is the whole shade.
    assert_eq!(set[0].input_schema["required"], json!([]));
}

/// **The containment rule, asserted** (DESIGN §6, §16.1): the description is
/// the one text a model reads before it spends a call, so this tool's price —
/// the enable, the retention ruling, the read-only scope, and why there is no
/// SMS tool to look for — is written there and not only in the refusal that
/// comes back.
#[test]
fn the_tool_states_its_price_and_its_forgetfulness_where_the_model_will_read_it() {
    let said = tools().swap_remove(0).description;
    // The one operator act, named in the description and not only in the
    // refusal.
    assert!(said.contains("notification access"));
    assert!(said.contains("system settings"));
    // The retention ruling, in the words a caller can act on: this cannot
    // answer for a moment nobody asked about.
    assert!(said.contains("nothing is recorded between calls"));
    assert!(said.contains("already dismissed is gone"));
    // The refused shape, stated rather than left for a model to hunt for.
    assert!(said.contains("no SMS tools"));
    // The rung's scope, so a model does not try to dismiss or reply.
    assert!(said.contains("Read-only"));
    assert!(said.contains("ongoing"));
}

/// The cap is a design decision, so it is an assertion: stated is honoured,
/// unstated and unreadable both fall back to the one default, and zero is not
/// a cap — `cap`'s own reading everywhere else in the table.
#[test]
fn the_cap_defaults_and_a_stated_one_is_honoured() {
    assert_eq!(shown(&args(json!({}))), SHOWN);
    assert_eq!(shown(&args(json!({ "limit": 3 }))), 3);
    assert_eq!(shown(&args(json!({ "limit": 0 }))), SHOWN);
    assert_eq!(shown(&args(json!({ "limit": -2 }))), SHOWN);
    assert_eq!(shown(&args(json!({ "limit": "lots" }))), SHOWN);
}

#[test]
fn a_build_with_no_android_says_so() {
    for input in [json!({}), json!({ "limit": 5 })] {
        let capture = run(&args(input));
        assert_eq!(capture.exit_code, REFUSED);
        assert_eq!(
            capture.stderr,
            "this build has no Android whose shade to read: the tool exists only on the \
             device\n"
        );
    }
    assert!(absent().starts_with("err\n"));
}
