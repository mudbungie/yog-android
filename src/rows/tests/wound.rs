//! The settled-failure notice (REMOTE §9.16): the class always, the remedy
//! on the refusal arm, the adapter's words where it left any — and no row at
//! all for the engine's word for *nothing is wounded*.

use super::{go, wounded};
use crate::rows::{RowClass, Tone};

#[test]
fn a_refusal_names_the_provider_row_a_sign_in_is_wanted_on() {
    let rows = go(&[wounded("«wound»", "refused", None, Some("anthropic"))]);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].prefix, "wound: refused");
    assert_eq!(rows[0].preview, "a sign-in is wanted on anthropic");
    assert_eq!(rows[0].class, RowClass::Other);
    assert_eq!(rows[0].tone, Tone::Bad);
    assert_eq!(rows[0].role, None);
}

#[test]
fn a_refusal_with_no_row_derivable_still_offers_the_sign_in() {
    let rows = go(&[wounded("«wound»", "refused", None, None)]);
    assert_eq!(rows[0].preview, "a sign-in is wanted");
}

#[test]
fn a_silent_adapter_says_its_last_words_or_that_the_conversation_is_over() {
    let rows = go(&[wounded(
        "«wound»",
        "no_response",
        Some("the adapter's last words"),
        None,
    )]);
    assert_eq!(rows[0].prefix, "wound: no_response");
    assert_eq!(rows[0].preview, "the adapter's last words");
    let rows = go(&[wounded("«wound»", "output_limit", None, None)]);
    assert_eq!(rows[0].preview, "this conversation is not coming back");
}

#[test]
fn a_wound_of_class_none_is_no_row() {
    assert!(go(&[wounded("«wound»", "none", None, None)]).is_empty());
}
