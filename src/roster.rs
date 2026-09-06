//! **The conversation list's two readings of one carried fact** (REMOTE
//! §9.9, bl-e837): what order the rows go in, and what each one says about
//! *when*.
//!
//! Both spend `last_active_unix` — the engine's own epoch second for the
//! subtree's last action — and neither derives it. §9.9 is explicit about
//! why it is carried rather than computed from `age_secs`: *"A seat holding
//! `age_secs` alone could subtract it from its own clock, and then every
//! client says a different time for one instant."*
//!
//! **Why the stamp and not the age, when the age is right there.** The age is
//! the distance from the ENGINE's clock *at answer time*, which is exactly
//! right for the instant it was answered and wrong a minute later — and this
//! app paints a cached roster before any answer at all (§14), where an age is
//! frozen and a stamp keeps ticking. So the label is the stamp against this
//! device's clock, which is the one thing that ages correctly while the
//! engine is unreachable.
//!
//! **Order is a list's own business, not world state — but the tree is world
//! state** (bl-06d3). The wire's order is §2.3's descent: id-sorted siblings,
//! pre-order within a subtree, so a row's parent is the nearest row above it
//! one `depth` shallower. Sorting that flat by recency destroyed it — a fan's
//! children scattered up and down the list among conversations an operator
//! had started, with nothing saying which were which. Three sibling rows for
//! a root and its two dispatched children read as three separate pieces of
//! work.
//!
//! So the recency order is taken **one rung up**: the SUBTREES are ordered
//! newest first, and each keeps the engine's own descent inside it. Nothing is
//! derived to do it — a root row's `last_active_unix` is already *the
//! subtree's* last action, which is exactly the key a subtree is ordered by.
//! The sort is stable, so subtrees sharing a stamp keep the engine's order
//! underneath. And a child now always sits directly under its parent, which is
//! what makes [`indent`] mean anything.

use crate::codec::ConvRow;

/// The list, newest subtree first, each subtree still in the engine's descent.
///
/// A row at depth 0 opens a subtree and everything under it belongs to that
/// one. The first row opening a subtree even when it is deeper than 0 is not a
/// special case: it is a read whose root this answer did not carry, and a
/// member of nothing is a subtree of one.
pub fn ordered(rows: Vec<ConvRow>) -> Vec<ConvRow> {
    let mut trees: Vec<Vec<ConvRow>> = Vec::new();
    for row in rows {
        if row.depth == 0 {
            trees.push(Vec::new());
        }
        match trees.last_mut() {
            Some(tree) => tree.push(row),
            None => trees.push(vec![row]),
        }
    }
    trees.sort_by_key(|tree| {
        std::cmp::Reverse(tree.first().map_or(i64::MIN, |row| row.last_active_unix))
    });
    trees.into_iter().flatten().collect()
}

/// How far a row hangs under its root, in points — the desktop seat's own
/// spelling (lernie `ui/convs.rs`), because two seats reading one `depth` off
/// one wire should not disagree about what a subagent looks like.
///
/// Added rather than multiplied, over a **bounded** count: there is no cast
/// from the wire's own width to a screen coordinate, so there is nothing to
/// truncate. The cap is not a special case either — past it a phone's list is
/// unreadable whatever the indent says, and the row's own text is what the
/// extra width would have cost.
pub fn indent(depth: usize) -> f32 {
    const STEP: f32 = 16.0;
    const DEEPEST: usize = 8;
    (0..depth.min(DEEPEST)).fold(0.0, |at, _| at + STEP)
}

/// Seconds in the units a roster row speaks.
const MINUTE: i64 = 60;
const HOUR: i64 = 60 * MINUTE;
const DAY: i64 = 24 * HOUR;
const WEEK: i64 = 7 * DAY;

/// How long ago, in the shortest true thing that can be said about it. A
/// clock ahead of the engine's reads as `now` rather than as a negative age:
/// the two clocks disagreeing is not a fact about the conversation.
pub fn stamp(last_active_unix: i64, now_unix: i64) -> String {
    ago(now_unix.saturating_sub(last_active_unix))
}

/// The same spelling over an age the engine already took (§13.8): a queue row
/// carries `age_secs` rather than a stamp, and the two must read alike or one
/// screen's *4h* is another's *4 hours ago*. One home, spent twice — [`stamp`]
/// is this function with the subtraction in front of it.
pub fn ago(ago: i64) -> String {
    if ago < MINUTE {
        return "now".to_owned();
    }
    let (unit, each) = if ago < HOUR {
        ('m', MINUTE)
    } else if ago < DAY {
        ('h', HOUR)
    } else if ago < WEEK {
        ('d', DAY)
    } else {
        ('w', WEEK)
    };
    format!("{}{unit}", ago / each)
}

/// This device's clock as an epoch second, for [`stamp`]'s other half. A
/// clock before the epoch is a device with no clock at all, and `0` makes
/// every row read `now` — which is the honest answer when this end cannot
/// say what time it is.
pub fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| i64::try_from(since.as_secs()).unwrap_or(0))
}

#[cfg(test)]
mod tests;
