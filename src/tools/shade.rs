//! **The notification shade, as text** (DESIGN §16.1, rung 2 of the
//! teleoperation corpus): one tool, `notifications`, and the platform service
//! the operator enables once for it to have anything to read.
//!
//! **This is the SMS-adjacent surface, and the SMS permissions are refused.**
//! The teleoperation want behind *"read my texts"* is reading what the phone
//! was told — a two-factor code, a message — and the shade already carries it
//! as the messaging app's own notification text. `READ_SMS`/`SEND_SMS` are
//! hard-restricted permissions, and sending one is the operator's voice on a
//! channel with no undo. One settings enable answers the read want whole; the
//! description says so, because a model that does not know why there is no
//! `read_sms` will look for one.
//!
//! **What a caller may read is the whole shade, and there is deliberately no
//! per-app filter.** A `NotificationListenerService` sees every notification
//! on the device or none — that is the shape of the platform's own grant — so
//! an allowlist inside this app would advertise a narrowing the OS does not
//! enforce, and it is §16.1's refused per-tool toggle screen wearing a
//! different hat: a second authority beside the OS grant, drifting the first
//! time one of them is changed. The operator's severability is the enable
//! itself, which is where the capability is.
//!
//! **Nothing is kept.** The service holds no history, writes no file and logs
//! nothing — it does not override the posted/removed callbacks at all, so a
//! notification that arrives while nobody is asking is not recorded anywhere.
//! Every answer is `getActiveNotifications` read at the moment of the call:
//! the platform already holds the shade, and a copy would be a second store of
//! one fact — a durable one, on a device where nothing sweeps it, carrying
//! exactly the material this rung exists to read. The honest cost of that
//! ruling rides in the description: what was dismissed is gone, and this tool
//! cannot answer what arrived while it was not asked.
//!
//! **Read-only at this rung**: no dismiss, no reply, no action fired. The
//! service could do all three; none is built, and the ball that would build
//! one is the place to argue for it.
//!
//! **What is pure lives here and is tested**: the advertised element, the
//! price it states, and the cap reading. The answer comes back in
//! [`super::bridged`]'s two-line protocol, and the JNI that fetches it is
//! [`bridge`] — android-only and excluded from coverage, the same seam every
//! other bridged tool draws.

use serde_json::{Map, Value, json};

use super::bridged::answer;
use super::{cap, object_schema};
use crate::codec::{Capture, Tool};

#[cfg(target_os = "android")]
mod bridge;

#[cfg(test)]
mod tests;

pub(crate) const NOTIFICATIONS: &str = "notifications";

/// How many rows an unqualified call answers with. A shade is usually shorter
/// than this, so the cap is not felt in the ordinary case; what it excludes is
/// the phone that has sat unlooked-at for a week answering a hundred rows into
/// a model's context. The answer says how many there were either way, so a
/// caller that was capped knows it and can ask for more.
const SHOWN: usize = 20;

/// The advertised element. Advertised whether or not the listener is enabled:
/// an advertisement is a fact about what this machine offers, and whether it
/// can act right now is a refusal in band (REMOTE §5's staleness correction).
pub(crate) fn tools() -> Vec<Tool> {
    vec![super::tool(
        NOTIFICATIONS,
        "Read this Android device's notification shade as text — for each notification the \
         app that posted it, how long ago it arrived, its title and its text, newest first. \
         This is how to read what the phone was TOLD: a two-factor code, a message, a \
         delivery alert. There are no SMS tools and there will not be: Android's SMS \
         permissions are hard-restricted, and a message's text is in the shade anyway. \
         Requires notification access, which the operator enables once in system settings \
         and nobody else can; without it this refuses in band naming that act. It reads what \
         is in the shade AT THE MOMENT YOU CALL — nothing is recorded between calls, so a \
         notification already dismissed is gone and this cannot tell you what arrived while \
         you were not asking. Rows marked ongoing are the standing ones (a charging phone, a \
         running download), not events. Read-only: nothing here dismisses a notification, \
         replies to one, or presses its buttons.",
        object_schema(
            json!({ "limit": { "type": "integer",
                               "description": "how many to answer with, newest first; \
                                               20 by default" } }),
            &[],
        ),
    )]
}

/// Dispatch. One tool, one argument, and the shade itself is the platform's.
pub(crate) fn run(o: &Map<String, Value>) -> Capture {
    answer(&bridge_notifications(&shown(o).to_string()))
}

/// How many rows the caller asked for. A function rather than a line inside
/// the dispatch so the default is assertable without a device — the cap is a
/// design decision, and a decision nothing checks is a comment.
fn shown(o: &Map<String, Value>) -> usize {
    cap(o, "limit", SHOWN)
}

/// The bridge, or the sentence a build without one gives (see
/// [`super::bridged::absent`]).
#[cfg(not(target_os = "android"))]
pub(crate) fn absent() -> String {
    super::bridged::absent("Android whose shade to read")
}

#[cfg(not(target_os = "android"))]
fn bridge_notifications(_limit: &str) -> String {
    absent()
}

#[cfg(target_os = "android")]
use bridge::bridge_notifications;
