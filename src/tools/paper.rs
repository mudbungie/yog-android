//! **The paper tools** (DESIGN §16.1, rung 1 of the teleoperation corpus):
//! what this device is doing, what it holds, what it can put in front of the
//! operator, and what it can open. Four direct verbs for things the interface
//! tools could only reach by puppeting the glass.
//!
//! **Paper because they cost no platform service at all** — no
//! `AccessibilityService` to enable, no foreground service to run. Their
//! whole price is what the OS grants an app uid, and each one states its own
//! in the description a model reads before it spends a call:
//!
//! * `device` — plain reads, no runtime permission, works with nothing on
//!   screen.
//! * `clipboard_set` — a write, which the platform allows where it blocks a
//!   read. There is deliberately no clipboard *read* tool (§16.1's refused
//!   shapes): Android blocks it outside the focused app, and `ui_read` is the
//!   honest alternative because it reads what is actually on the glass.
//! * `notify` — `POST_NOTIFICATIONS` is a runtime grant on API 33+. Not held
//!   is a refusal in band naming the act that grants it, and the system's own
//!   dialog goes up when this app is in front to raise it (the bl-d815
//!   permission-result hook, which `dev.yog.Notify` rides on its own request
//!   code).
//! * `open` — no permission, but the platform has refused a background
//!   activity launch since API 29, so it refuses in band when this app is not
//!   in front. It is **typed** — a URL or shared text, never a run-any-intent
//!   payload: REMOTE §5.2 refused that wrapper meta-tool twice and the
//!   reasoning binds here.
//!
//! **Every refusal names the one operator act that fixes it**, which is
//! bl-5710's editorial lesson kept as this corpus's rule (§16.1): a refusal
//! that teaches is the difference between a priced capability and a decoy.
//!
//! **What is pure lives here and is tested**: the advertised elements and the
//! argument reading. The answers come back in [`super::bridged`]'s two-line
//! protocol, and the JNI that fetches them is [`bridge`] — android-only and
//! excluded from coverage, the same seam the interface tools draw.

use serde_json::{Map, Value, json};

use super::bridged::answer;
use super::{BAD_INPUT, arg, object_schema, refused};
use crate::codec::{Capture, Tool};

#[cfg(target_os = "android")]
mod bridge;

#[cfg(test)]
mod tests;

pub(crate) const DEVICE: &str = "device";
pub(crate) const CLIPBOARD: &str = "clipboard_set";
pub(crate) const NOTIFY: &str = "notify";
pub(crate) const OPEN: &str = "open";

/// Every paper tool's advertised element. Like the interface tools they are
/// advertised whether or not their grant is held: an advertisement is a fact
/// about what this machine offers, and whether it can act right now is a
/// refusal in band (REMOTE §5's staleness correction). A set that tracked
/// permission state would rewrite the engine-side document on every grant
/// flip, which is what §16.1 means by *the advertisement is static and whole*.
pub(crate) fn tools() -> Vec<Tool> {
    vec![
        super::tool(
            DEVICE,
            "Read what this Android device is doing right now: battery level and whether it \
             is charging, which kind of network it is on, and how much of its storage is \
             free. Plain reads that need no runtime permission, ask the operator for \
             nothing, and answer whether or not the app is on screen.",
            object_schema(json!({}), &[]),
        ),
        super::tool(
            CLIPBOARD,
            "Put text on this Android device's clipboard, for the operator to paste into any \
             app. Works with the app in the background — Android's clipboard restriction is \
             on the read, not the write — but Android 13 and later clears the clipboard \
             about an hour after it is set. There is no tool that READS the clipboard: that \
             half really is blocked outside the focused app, so ui_read — which reads what \
             is actually on the glass — is the honest way to see what is there.",
            object_schema(
                json!({ "text": { "type": "string",
                                  "description": "the text to put on the clipboard" } }),
                &["text"],
            ),
        ),
        super::tool(
            NOTIFY,
            "Post a notification on this Android device: the way to reach the operator when \
             nothing of yours is on their screen. Requires Android's notification \
             permission — without it this refuses in band naming the settings act that \
             grants it, and raises the system's own dialog when the app is in front. This is \
             a tool you invoke; it is not the seat's own attention machinery, which is the \
             app's and not yours to fire.",
            object_schema(
                json!({ "title": { "type": "string", "description": "the notification's title" },
                        "text": { "type": "string",
                                  "description": "the line under it; optional" } }),
                &["title"],
            ),
        ),
        super::tool(
            OPEN,
            "Open something on this Android device's screen: `url` opens a web page, a map, \
             a dialable number or any other addressable thing, and `text` hands text to the \
             device's share sheet. State one; if both are given the url is what opens. \
             Android has refused an activity launch from an app that is not in front since \
             Android 10, so this refuses in band saying so unless yog is the app on screen. \
             There is no run-any-intent tool: this typed pair is the whole of it.",
            object_schema(
                json!({ "url": { "type": "string",
                                 "description": "what to open, as a URI" },
                        "text": { "type": "string",
                                  "description": "text to hand to the share sheet" } }),
                &[],
            ),
        ),
    ]
}

/// Dispatch one paper tool. Nothing here takes the app's storage path: none
/// of the four writes a file, which is why `device` answers its facts as text
/// rather than a path the way a screenshot must.
pub(crate) fn run(tool: &str, o: &Map<String, Value>) -> Capture {
    match tool {
        DEVICE => answer(&bridge_device()),
        CLIPBOARD => match arg(o, "text") {
            Ok(text) => answer(&bridge_clipboard(&text)),
            Err(why) => refused(BAD_INPUT, &why),
        },
        NOTIFY => notify(o),
        _ => open(o),
    }
}

/// A notification is a title and, optionally, the line under it. The title is
/// required because a notification with nothing on its first line is a row
/// the operator cannot read; the body is not, because *"the build is green"*
/// is a whole message.
fn notify(o: &Map<String, Value>) -> Capture {
    match arg(o, "title") {
        Ok(title) => {
            let text = o.get("text").and_then(Value::as_str).unwrap_or_default();
            answer(&bridge_notify(&title, text))
        }
        Err(why) => refused(BAD_INPUT, &why),
    }
}

/// An open names a URL or text to share, and naming neither is the one thing
/// it cannot do — the same shape `ui_tap` takes, for the same reason: an act
/// with no subject is a mis-call, not an empty action.
fn open(o: &Map<String, Value>) -> Capture {
    if let Some(url) = o.get("url").and_then(Value::as_str) {
        return answer(&bridge_open("url", url));
    }
    match o.get("text").and_then(Value::as_str) {
        Some(text) => answer(&bridge_open("text", text)),
        None => refused(
            BAD_INPUT,
            "state either \"url\", the thing to open, or \"text\", the text to share",
        ),
    }
}

/// The bridge, or the sentence a build without one gives (see
/// [`super::bridged::absent`]).
#[cfg(not(target_os = "android"))]
pub(crate) fn absent() -> String {
    super::bridged::absent("Android to ask")
}

#[cfg(not(target_os = "android"))]
fn bridge_device() -> String {
    absent()
}
#[cfg(not(target_os = "android"))]
fn bridge_clipboard(_text: &str) -> String {
    absent()
}
#[cfg(not(target_os = "android"))]
fn bridge_notify(_title: &str, _text: &str) -> String {
    absent()
}
#[cfg(not(target_os = "android"))]
fn bridge_open(_kind: &str, _value: &str) -> String {
    absent()
}

#[cfg(target_os = "android")]
use bridge::{bridge_clipboard, bridge_device, bridge_notify, bridge_open};
