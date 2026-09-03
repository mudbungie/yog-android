//! **The inventory**: the `act:<op>` tags read back out of a walk's
//! accessibility dumps.
//!
//! It scans text rather than parsing XML, and that is deliberate. The dump is
//! an `AccessibilityNodeInfo` tree serialized by `uiautomator`, where a tag
//! rides inside a `content-desc` attribute among other prose; an XML parse
//! would be a dependency and a second thing to keep true about a format this
//! repo does not own. What a tag IS — the reserved prefix and an op token —
//! is the whole grammar, and finding it anywhere in the bytes is exactly the
//! claim being made: *this string reached the platform's accessibility layer*.
//!
//! **The token charset is the roster's own.** Ops are lowercase with hyphens
//! (`clear-trail`, `delete-agent`, `work-diff`), so the run stops at the first
//! character that cannot be one — a quote, a space, an XML delimiter. A `act:`
//! with nothing after it yields the empty token, which then fails the
//! "names a corpus op" assertion by name rather than being swallowed.

use std::collections::BTreeSet;

/// Every op tagged anywhere in `text`. Duplicates collapse: the claim is
/// presence, and a control painted on three screens is one control.
pub(super) fn found(text: &str) -> BTreeSet<String> {
    text.split(super::PREFIX)
        .skip(1)
        .map(|tail| tail.chars().take_while(|c| is_op(*c)).collect())
        .collect()
}

/// A character an op token may carry.
fn is_op(c: char) -> bool {
    c.is_ascii_lowercase() || c == '-'
}
