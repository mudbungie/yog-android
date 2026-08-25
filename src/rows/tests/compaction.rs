//! The marker that stands for what is not in the record: the counter it
//! proves, and the summary it may or may not have.

use super::{compacted, go};
use crate::rows::{RowClass, Tone};

#[test]
fn a_span_names_both_ends_and_pluralizes_its_count() {
    let rows = go(&[compacted("020", 3, 17, "they argued about paths")]);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].prefix, "✂ 15 entries compacted away — 003–017");
    assert_eq!(rows[0].preview, "they argued about paths");
    assert_eq!(rows[0].class, RowClass::Other);
    assert_eq!(rows[0].tone, Tone::Weak);
    assert_eq!(rows[0].role, None);
    assert!(rows[0].hover.starts_with("These entries were removed:"));
}

#[test]
fn one_entry_reads_as_one_number_in_the_singular() {
    let rows = go(&[compacted("020", 7, 7, "one turn went")]);
    assert_eq!(rows[0].prefix, "✂ 1 entry compacted away — 007");
}

#[test]
fn a_mark_with_no_summary_still_says_the_entries_are_gone() {
    let rows = go(&[compacted("020", 1, 2, "")]);
    assert_eq!(rows[0].prefix, "✂ 2 entries compacted away — 001–002");
    assert_eq!(rows[0].preview, "(no summary on this mark)");
}

#[test]
fn a_reversed_span_saturates_rather_than_underflowing() {
    let rows = go(&[compacted("020", 9, 4, "")]);
    assert_eq!(rows[0].prefix, "✂ 1 entry compacted away — 009–004");
}
