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
//! **Order is a list's own business, not world state.** The wire's order is
//! §2.3's descent — id-sorted siblings, pre-order within a subtree — which is
//! a tree's order, and this app paints a flat list with no indent to make a
//! tree of. Ordering that flat list by recency is a presentation choice over
//! values the engine already carried, and the sort is stable, so rows sharing
//! a stamp keep the engine's own order underneath.

use crate::codec::ConvRow;

/// The list, newest first.
pub fn ordered(mut rows: Vec<ConvRow>) -> Vec<ConvRow> {
    rows.sort_by_key(|row| std::cmp::Reverse(row.last_active_unix));
    rows
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
