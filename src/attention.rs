//! **The scheduled fetch** (DESIGN §17; yog REMOTE §14 rung 1): one ordinary
//! ask, performed on the platform's own schedule, so attention reaches a
//! pocketed phone with no engine work and no push path.
//!
//! **What it asks, and why that one.** `Query::Workspaces` — the roster read
//! this seat already performs at cadence. Its rows carry `attention` (yog's
//! `ws_row` spelling, mirrored in [`crate::codec::WsRow`]), which is the same
//! per-workspace count the roster screen paints its attention mark from. It is the
//! cheapest attention-shaped read the vendored corpus answers: one
//! connection, one frame, rows a handful long, and nothing derived here that
//! the engine did not already say. REMOTE §14.1's `Query::Attention` lane is
//! upstream's ball (yog bl-09aa) and rung 2's; this rung waits on nobody.
//!
//! **Silence is the failure mode, deliberately.** No material, an engine that
//! will not answer, an answer this end cannot read — every one of them ends
//! the run with no notification and no state written, and the next schedule
//! tries again. A phone in a pocket must never nag about network: the
//! operator did not ask for a fetch report, they asked to be told when
//! something wants them.
//!
//! **What is worth waking a human for is a RISE.** A count that stayed put
//! was already announced; a count that fell is the operator having dealt with
//! it. Only a workspace whose attention is higher than the number last
//! announced for it earns a notification, and every run — silent or not —
//! records what it saw, so a count that drops and climbs again wakes the
//! operator a second time.
//!
//! **This module never writes the paint-first cache** (§14). That cache has
//! one writer, the seat model's worker, and a pass that stored a roster over
//! a focus the operator had taken deeper would paint the wrong screen on the
//! next resume. The fetch's own memory is its own file, one writer and one
//! reader, and nothing else reads it.

use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{Value, json};

use crate::codec::reply::Reply;
use crate::codec::{Ask, Gesture, WsRow, encode};
use crate::transport::Seat;

/// The material directory, as [`crate::shell`] spells it: the fetch is handed
/// this app's private files directory and finds the wire under it, exactly
/// where the enrollment channels write it.
pub(crate) const WIRE: &str = "wire";
/// The fetch's own memory — a sibling of `wire/` and of `cache/`, never
/// inside either. Nothing here is a key and nothing here is the world.
const MEMORY: &str = "attention";
const SEEN: &str = "seen.json";

/// The file's marker and its layout version, in one field — the shape
/// [`crate::cache`] and [`crate::envelope`] both take, for its reason: a
/// version with no name is read out of whatever JSON happens to be there.
const TAG: &str = "yog-attention";
const VERSION: u64 = 1;

/// What a wake says: the line the operator reads on the lock screen, and the
/// line under it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    pub title: String,
    pub text: String,
}

/// What each workspace's attention stood at — absent means none, which is why
/// a zero is never stored.
type Counts = BTreeMap<String, usize>;

/// **One scheduled run**, whole: ask, decide, remember. `None` is silence,
/// and it is what every failure answers.
///
/// `dir` is this app's private files directory — the platform's, handed in by
/// the caller, because a path is a fact about the device and this crate holds
/// no device facts of its own.
pub fn sweep(dir: &Path) -> Option<Notice> {
    let now = counts(&asked(dir)?);
    let notice = risen(&now, &read_seen(dir));
    write_seen(dir, &now);
    notice
}

/// The ask. Every way it can fail is one answer — *nothing to say this run* —
/// because a scheduled fetch has nobody to tell and a next run either way.
fn asked(dir: &Path) -> Option<Vec<WsRow>> {
    let material = crate::material::read_dir(&dir.join(WIRE)).ok().flatten()?;
    let seat = Seat::open(&material).ok()?;
    match seat.answered(&encode(&Gesture::Ask(Ask::Workspaces))) {
        Ok(Reply::Workspaces { rows, .. }) => Some(rows),
        _ => None,
    }
}

/// The rows as the fact the fetch keeps. A workspace wanting nothing is
/// absent rather than zero: the two say the same thing, and the smaller file
/// is the one that stays readable.
fn counts(rows: &[WsRow]) -> Counts {
    rows.iter()
        .filter(|row| row.attention > 0)
        .map(|row| (row.workspace.clone(), row.attention))
        .collect()
}

/// **The decision**: what rose above what was last announced.
fn risen(now: &Counts, seen: &Counts) -> Option<Notice> {
    let rows: Vec<(String, usize)> = now
        .iter()
        .filter(|(workspace, count)| **count > seen.get(*workspace).copied().unwrap_or(0))
        .map(|(workspace, count)| (workspace.clone(), *count))
        .collect();
    let (first, _) = rows.first()?;
    let title = if rows.len() == 1 {
        format!("{first} wants attention")
    } else {
        format!("{} workspaces want attention", rows.len())
    };
    Some(Notice {
        title,
        text: rows
            .iter()
            .map(|(workspace, count)| format!("{workspace} {count}"))
            .collect::<Vec<String>>()
            .join(", "),
    })
}

/// What the last run announced. **Every doubt reads as nothing announced**,
/// which costs one notification the operator may have seen before and can
/// never cost a wake that does not happen — the direction a fetch that exists
/// to wake somebody must fail in.
fn read_seen(dir: &Path) -> Counts {
    let Ok(text) = std::fs::read_to_string(dir.join(MEMORY).join(SEEN)) else {
        return Counts::new();
    };
    let Ok(value) = serde_json::from_str::<Value>(&text) else {
        return Counts::new();
    };
    let Some(object) = value.as_object() else {
        return Counts::new();
    };
    if object.get("tag") != Some(&json!(TAG)) || object.get("version") != Some(&json!(VERSION)) {
        return Counts::new();
    }
    let Some(seen) = object.get("seen").and_then(Value::as_object) else {
        return Counts::new();
    };
    seen.keys()
        .filter_map(|key| Some((key.clone(), crate::codec::fields::usize_of(seen, key).ok()?)))
        .collect()
}

/// Record what this run saw. **A write that fails is not reported and not
/// retried**: its whole cost is that the next rise the operator already knows
/// about wakes them once more, and there is no surface a scheduled job could
/// report a disk error to that is not itself the nagging this rung refuses.
fn write_seen(dir: &Path, now: &Counts) {
    let at = dir.join(MEMORY);
    let _ = std::fs::create_dir_all(&at);
    let _ = std::fs::write(
        at.join(SEEN),
        json!({ "tag": TAG, "version": VERSION, "seen": now }).to_string(),
    );
}

#[cfg(test)]
mod tests;
