//! **The interface tools**: read the screen, drive the screen (bl-1511).
//!
//! They are the half of tool hosting that needs a platform service, because
//! an app uid can do neither on its own — established by probe: `screencap`
//! wants a signature-level permission, and one app cannot see another's views
//! at all. An `AccessibilityService` carries reading, gesture dispatch and
//! screenshots in one place, and **enabling it is the operator's act**
//! (DESIGN §5): the app never grants itself anything, so until it is on,
//! every tool here refuses in band with a sentence naming the fix — which is
//! REMOTE §5's own staleness correction, a client refusing a tool it cannot
//! presently carry.
//!
//! **The table is advertised whether or not the service is on**, because the
//! advertisement is a fact about what this machine offers and the refusal is
//! a fact about right now. Two tables — one for enabled and one for not —
//! would put a connectivity-rate fact into a durable document, which is the
//! defect REMOTE §5 was amended to remove.
//!
//! **What is pure lives here and is tested**: the advertised elements and the
//! argument reading. The two-line answer protocol the Java side speaks is
//! [`super::bridged`]'s, shared with the paper tools since bl-f34f — one
//! protocol, one parser. The JNI itself is [`bridge`], android-only and
//! excluded from coverage — the seam is drawn so that everything except the
//! platform call is verified without a device.

use serde_json::{Map, Value, json};

use super::bridged::answer;
use super::{BAD_INPUT, arg, object_schema, refused};
use crate::codec::{Capture, Tool};

#[cfg(target_os = "android")]
mod bridge;

pub(crate) const READ: &str = "ui_read";
pub(crate) const TAP: &str = "ui_tap";
pub(crate) const TYPE: &str = "ui_type";
pub(crate) const KEY: &str = "ui_key";
pub(crate) const SHOT: &str = "screenshot";

/// The name a screenshot lands under when the caller states no path. It sits
/// in the app's own storage, which is the one directory this uid can always
/// write.
const SHOT_NAME: &str = "screenshot.png";

/// Every interface tool's advertised element.
pub(crate) fn tools() -> Vec<Tool> {
    vec![
        super::tool(
            READ,
            "Read whatever is on this Android device's screen right now, as text: one line \
             per node with its class, its words, its position and size, and whether it is \
             clickable or editable. This is the tool to read the screen with — a screenshot \
             is for a person to look at. Requires the yog accessibility service to be \
             enabled on the device; it refuses saying so if it is not, and cannot see \
             windows the system marks secure.",
            object_schema(json!({}), &[]),
        ),
        super::tool(
            TAP,
            "Tap this Android device's screen, either at a pixel coordinate or on the first \
             clickable node whose text or description contains `text`. Coordinates come from \
             ui_read. Requires the yog accessibility service.",
            object_schema(
                json!({ "x": { "type": "integer", "description": "screen x, with y" },
                        "y": { "type": "integer", "description": "screen y, with x" },
                        "text": { "type": "string",
                                  "description": "tap the node matching this instead" } }),
                &[],
            ),
        ),
        super::tool(
            TYPE,
            "Type text into whatever field currently holds input focus on this Android \
             device, replacing what is there. Tap the field first. Requires the yog \
             accessibility service.",
            object_schema(
                json!({ "text": { "type": "string", "description": "the text to enter" } }),
                &["text"],
            ),
        ),
        super::tool(
            KEY,
            "Press one of this Android device's system controls: back, home, recents, \
             notifications or quick-settings. Requires the yog accessibility service.",
            object_schema(
                json!({ "key": { "type": "string",
                                 "enum": ["back", "home", "recents",
                                          "notifications", "quick-settings"] } }),
                &["key"],
            ),
        ),
        super::tool(
            SHOT,
            "Take a screenshot of this Android device and write it to a PNG in the app's own \
             storage. Answers with the path, the size in bytes and the dimensions — not the \
             image, which no tool result can carry. Use ui_read to find out what is on the \
             screen. Requires the yog accessibility service.",
            object_schema(
                json!({ "path": { "type": "string",
                                  "description": "where to write the PNG" } }),
                &[],
            ),
        ),
    ]
}

/// Dispatch one interface tool. `data_dir` is the app's own storage, which is
/// where a screenshot goes when the caller names no path.
pub(crate) fn run(tool: &str, o: &Map<String, Value>, data_dir: &str) -> Capture {
    match tool {
        READ => answer(&bridge_read()),
        TAP => tap(o),
        TYPE => match arg(o, "text") {
            Ok(text) => answer(&bridge_type(&text)),
            Err(why) => refused(BAD_INPUT, &why),
        },
        KEY => match arg(o, "key") {
            Ok(key) => answer(&bridge_key(&key)),
            Err(why) => refused(BAD_INPUT, &why),
        },
        _ => {
            let path = arg(o, "path").unwrap_or_else(|_| format!("{data_dir}/{SHOT_NAME}"));
            answer(&bridge_shot(&path))
        }
    }
}

/// A tap names a point or a node, and naming neither is the one thing it
/// cannot do — a tap with nowhere to land is a mis-call, not an empty action.
fn tap(o: &Map<String, Value>) -> Capture {
    if let Some(text) = o.get("text").and_then(Value::as_str) {
        return answer(&bridge_tap_text(text));
    }
    match (coord(o, "x"), coord(o, "y")) {
        (Some(x), Some(y)) => answer(&bridge_tap(x, y)),
        _ => refused(
            BAD_INPUT,
            "state either \"text\", or both \"x\" and \"y\" in screen pixels",
        ),
    }
}

/// One screen coordinate, narrowed to what a display can hold.
fn coord(o: &Map<String, Value>, key: &str) -> Option<i32> {
    o.get(key)
        .and_then(Value::as_i64)
        .and_then(|n| i32::try_from(n).ok())
}

/// The bridge, or the sentence a build without one gives — [`super::bridged`]
/// holds both the protocol and this refusal's shape, because the paper tools
/// speak the same one.
#[cfg(not(target_os = "android"))]
pub(crate) fn absent() -> String {
    super::bridged::absent("Android interface to read")
}

#[cfg(not(target_os = "android"))]
fn bridge_read() -> String {
    absent()
}
#[cfg(not(target_os = "android"))]
fn bridge_tap(_x: i32, _y: i32) -> String {
    absent()
}
#[cfg(not(target_os = "android"))]
fn bridge_tap_text(_text: &str) -> String {
    absent()
}
#[cfg(not(target_os = "android"))]
fn bridge_type(_text: &str) -> String {
    absent()
}
#[cfg(not(target_os = "android"))]
fn bridge_key(_key: &str) -> String {
    absent()
}
#[cfg(not(target_os = "android"))]
fn bridge_shot(_path: &str) -> String {
    absent()
}

#[cfg(target_os = "android")]
use bridge::{bridge_key, bridge_read, bridge_shot, bridge_tap, bridge_tap_text, bridge_type};

#[cfg(test)]
mod tests;
