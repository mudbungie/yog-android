//! **The two-line answer protocol** every Java bridge in this crate speaks,
//! and the one parser that reads it.
//!
//! Each platform entry point answers a single string: `ok\n<payload>` or
//! `err\n<sentence>`. Two lines rather than a thrown exception because an
//! exception across JNI must be checked for and cleared at every call site,
//! and one that is missed aborts the process under `CheckJNI` — a prefix
//! cannot be forgotten.
//!
//! It lives here rather than beside its first caller because there is more
//! than one bridge now — the interface tools' service (`tools::ui`), the paper
//! tools' platform half (`tools::paper`) and the sighted pair
//! (`tools::sighted`) — and a protocol with two definitions is a protocol that
//! drifts. The parser is pure and is tested below; the JNI on the far side of
//! it is the device's to answer for.
//!
//! [`Door`] is the call that speaks it, and it is here for the same reason:
//! two doors resolve a class of this app's and marshal N strings into a
//! descriptor built from their count, and a second copy of that would drift
//! from the first inside a week (bl-b0a9).

use crate::codec::Capture;

#[cfg(target_os = "android")]
mod door;

#[cfg(target_os = "android")]
pub(crate) use door::Door;

/// The verdict a bridged tool that could not act earns — a service that is
/// not enabled, a permission that is not held, an app that is not in front,
/// a platform that refused. One code, because the sentence is what a caller
/// acts on and a second number would be a second thing to keep in step.
pub(crate) const REFUSED: i32 = 1;

/// A bridge's answer as a capture. `ok\n…` is the payload and `err\n…` is the
/// sentence; anything else is a bridge this crate and that class no longer
/// agree on, which is worth saying plainly rather than guessing at.
pub(crate) fn answer(reply: &str) -> Capture {
    match reply.split_once('\n') {
        Some(("ok", payload)) => super::answered(payload.to_owned()),
        Some(("err", why)) => super::refused(REFUSED, why),
        _ => super::refused(
            REFUSED,
            &format!("the platform answered something unreadable: {reply:?}"),
        ),
    }
}

/// The refusal a build with no Android under it gives, in the same protocol,
/// naming what it has no platform to ask. A host build has nothing to call,
/// and saying so beats a tool that silently does nothing — the suite
/// exercises this arm, which is why the seam is a function rather than a
/// `cfg` inside every caller.
#[cfg(not(target_os = "android"))]
pub(crate) fn absent(what: &str) -> String {
    format!("err\nthis build has no {what}: the tool exists only on the device")
}

#[cfg(test)]
mod tests {
    use super::{REFUSED, answer};

    #[test]
    fn the_two_line_protocol_splits_into_the_captures_three_facts() {
        let ok = answer("ok\nthe payload\nsecond line");
        assert_eq!(ok.stdout, "the payload\nsecond line");
        assert_eq!(ok.stderr, "");
        assert_eq!(ok.exit_code, 0);
        let err = answer("err\nthe service is not enabled");
        assert_eq!(err.stdout, "");
        assert_eq!(err.stderr, "the service is not enabled\n");
        assert_eq!(err.exit_code, REFUSED);
        // An empty payload is an ordinary answer, not a malformed one.
        assert_eq!(answer("ok\n").stdout, "");
    }

    #[test]
    fn an_answer_in_no_protocol_at_all_says_so_rather_than_guessing() {
        for reply in ["", "ok", "what", "\n"] {
            let capture = answer(reply);
            assert_eq!(capture.exit_code, REFUSED, "for {reply:?}");
            assert!(
                capture.stderr.contains("unreadable"),
                "for {reply:?}: {}",
                capture.stderr
            );
        }
    }
}
