//! **The receipt's reading, and the one event it exists to make audible**
//! (REMOTE §5.1, PROTOCOL 8, yog bl-66d4 / bl-cc54). Its own file for
//! `consent.rs`'s reason: the parent is the loop's own story, and this is the
//! one fact the loop learns about a world it cannot see.
//!
//! The two tests are the same script with the `wrote` moved from one
//! presentation to the other, which is the whole distinction: identical bytes
//! on the wire, opposite meanings, and only the host knows which presentation
//! earned them.

use super::super::RESTORED;
use super::{advertised, host_against, restored, routed, settle, work};
use serde_json::json;

/// **A re-assertion that WROTE is a disarming.** The engine writes only a set
/// that differs, so this device presenting what it has always presented and
/// being told the document changed means the set in force was somebody else's
/// — put there while this box was running a tool and therefore holding no
/// parked read for the engine's two guards to work with.
///
/// It is counted rather than stopped on: the set has already been restored by
/// the presentation that discovered it, and there is nothing left to fix from
/// here. Being told is the remedy.
#[test]
fn a_reassertion_that_wrote_is_counted_as_a_disarming() {
    let (mut host, _served) = host_against(vec![
        vec![advertised()],
        vec![work(
            json!([{ "invocation": "i1", "tool": "echo", "input": {} }]),
        )],
        vec![routed("i1")],
        // The hand-off's re-assertion, and the engine says it changed the
        // document: something had replaced this device's set.
        vec![restored()],
        vec![],
    ]);
    let standing = settle(&mut host, &|s| s.restored == 1);
    assert_eq!(standing.served, 1);
    // The sentence the frame paints beside the count names both readings,
    // because this device cannot tell them apart and guessing is worse than
    // saying so.
    assert!(RESTORED.contains("has been put back"), "{RESTORED}");
    assert!(
        RESTORED.contains("bearing this device's identity"),
        "{RESTORED}"
    );
    assert!(RESTORED.contains("engine lost the set"), "{RESTORED}");
}

/// **A `true` on a channel's FIRST presentation says nothing.** Every fresh
/// channel presents into whatever the engine happens to be holding — nothing,
/// or an older set from a build ago — and the ordinary first presentation
/// writes. Only a presentation made after work this device just did can tell a
/// rival from a beginning, so the first one's reading is discarded and the
/// count stays at zero through a whole hand-off.
#[test]
fn a_wrote_on_a_channels_first_presentation_is_ordinary_and_says_nothing() {
    let (mut host, _served) = host_against(vec![
        vec![restored()],
        vec![work(
            json!([{ "invocation": "i1", "tool": "echo", "input": {} }]),
        )],
        vec![routed("i1")],
        // The re-assertion finds the set identical, which is the ordinary
        // answer and the one this loop expects on every later presentation.
        vec![advertised()],
        vec![],
    ]);
    let standing = settle(&mut host, &|s| s.served == 1);
    assert_eq!(standing.restored, 0);
}
