//! **The sighted pair** (DESIGN §16.1, rung 1b of the teleoperation corpus):
//! what this device can see, and where it is. Two tools, each behind a runtime
//! permission the operator grants and only the operator can grant.
//!
//! **A still answers a PATH, never the image** — the screenshot precedent
//! (DESIGN §6), and REMOTE §5.3's rule that a capture is text. A megabyte of
//! base64 is not something a model can read, and encoding one would be this
//! client adding a shape to the boundary. So the JPEG lands in the app's own
//! storage — the one directory this uid can always write — and the capture
//! names it, its size and its dimensions.
//!
//! **The default name is fixed and the file is overwritten**
//! (`camera.jpg`, `screenshot.png`'s own choice). A timestamped name per shot
//! would make an agent taking stills all day fill private storage with files
//! nobody deletes, on a device whose storage no one is watching; a caller that
//! wants to keep two names the second one itself.
//!
//! **A fix states how old and how rough it is, always.** The failure mode
//! here is not a refusal, it is a model acting on a stale fix — a phone that
//! has been indoors for an hour still has a last-known location, and it is
//! somewhere else. So the age is a line of every answer, said in the units a
//! reader acts on, and an answer that is a last-known fix rather than a new
//! one says which it is; the accuracy rides beside it because a fix good to a
//! kilometre and a fix good to five metres are different facts wearing the
//! same two numbers.
//!
//! **Both are foreground-bound at this rung, and both say so where a model
//! reads it.** Android refuses the camera outright to an app that is not on
//! screen, and delivers no fresh location to one without the separate
//! background-location grant this rung does not ask for (§16.1). A tool that
//! reported success for a capture the platform silently dropped would be the
//! decoy shape bl-5710 named, so each asks before it acts and refuses in band
//! naming the one operator act that fixes it.
//!
//! **What is pure lives here and is tested**: the advertised elements, the
//! price each states, and the argument reading. The answers come back in
//! [`super::bridged`]'s two-line protocol, and the JNI that fetches them is
//! [`bridge`] — android-only and excluded from coverage, the same seam every
//! other bridged tool draws.

use serde_json::{Map, Value, json};

use super::bridged::answer;
use super::{BAD_INPUT, arg, object_schema, refused};
use crate::codec::{Capture, Tool};

#[cfg(target_os = "android")]
mod bridge;

#[cfg(test)]
mod tests;

pub(crate) const CAMERA: &str = "camera";
pub(crate) const LOCATION: &str = "location";

/// The name a still lands under when the caller states no path. It sits in the
/// app's own storage beside the screenshot's, for the same reason.
const STILL_NAME: &str = "camera.jpg";

/// The two lenses a phone has. Read here rather than passed through, because
/// the platform's own answer to a third word would be a camera id it could not
/// find — a mis-call the model should be told about in its own terms.
const LENSES: [&str; 2] = ["back", "front"];

/// Both advertised elements. Advertised whether or not their grants are held:
/// an advertisement is a fact about what this machine offers, and whether it
/// can act right now is a refusal in band (REMOTE §5's staleness correction).
pub(crate) fn tools() -> Vec<Tool> {
    vec![
        super::tool(
            CAMERA,
            "Take one still photograph with this Android device's camera and write it to a \
             JPEG in the app's own storage. Answers the path, the size in bytes and the \
             pixel dimensions — not the image, which no tool result can carry; whoever \
             wants the picture fetches the file off the device. Requires Android's camera \
             permission, and requires yog to be the app on screen: Android refuses the \
             camera to an app that is not in front, so this refuses in band saying so \
             rather than answering for a photograph nobody took. The default path is \
             overwritten by the next call — name your own to keep one.",
            object_schema(
                json!({ "lens": { "type": "string", "enum": ["back", "front"],
                                  "description": "back (the default, pointing away from \
                                                  whoever holds the phone) or front" },
                        "path": { "type": "string",
                                  "description": "where to write the JPEG" } }),
                &[],
            ),
        ),
        super::tool(
            LOCATION,
            "Ask this Android device where it is: one fix, as latitude and longitude, with \
             how rough it is in metres and HOW OLD IT IS. Read the age before you act on \
             the position — if no new fix arrives while this waits, the answer is the last \
             one this device recorded, which may be hours old and somewhere else entirely, \
             and it says which it is. Requires Android's location permission and the \
             device's own location switch to be on; each refuses in band naming the act \
             that fixes it. Only while yog is on screen: this app does not hold the \
             background-location grant, which is a separate settings trip, so a phone in a \
             pocket has no new fix to give.",
            object_schema(json!({}), &[]),
        ),
    ]
}

/// Dispatch one sighted tool. `data_dir` is this app's own storage, which is
/// where a still goes when the caller names no path.
pub(crate) fn run(tool: &str, o: &Map<String, Value>, data_dir: &str) -> Capture {
    if tool == LOCATION {
        return answer(&bridge_location());
    }
    still(o, data_dir)
}

/// A still names its lens and its file, and may name neither: pointing away
/// from the holder and into the app's own storage is the shot a caller that
/// stated nothing meant.
fn still(o: &Map<String, Value>, data_dir: &str) -> Capture {
    let lens = match o.get("lens") {
        None => LENSES[0].to_owned(),
        Some(named) => match named.as_str().filter(|word| LENSES.contains(word)) {
            Some(word) => word.to_owned(),
            None => {
                return refused(BAD_INPUT, "\"lens\" is either \"back\" or \"front\"");
            }
        },
    };
    answer(&bridge_still(&lens, &destination(o, data_dir)))
}

/// Where the JPEG goes: the caller's path, or this app's own storage under the
/// one name. A function rather than a line inside the dispatch so the default
/// is assertable without a device — the file's location is a design decision,
/// and a decision nothing checks is a comment.
fn destination(o: &Map<String, Value>, data_dir: &str) -> String {
    arg(o, "path").unwrap_or_else(|_| format!("{data_dir}/{STILL_NAME}"))
}

/// The bridge, or the sentence a build without one gives (see
/// [`super::bridged::absent`]).
#[cfg(not(target_os = "android"))]
pub(crate) fn absent() -> String {
    super::bridged::absent("Android to look with")
}

#[cfg(not(target_os = "android"))]
fn bridge_still(_lens: &str, _path: &str) -> String {
    absent()
}
#[cfg(not(target_os = "android"))]
fn bridge_location() -> String {
    absent()
}

#[cfg(target_os = "android")]
use bridge::{bridge_location, bridge_still};
