//! **The status rule** (DESIGN §22 items 1, 2 and 10, bl-79bb; operator
//! ruling 2026-10-12; §23 item 14; §24 item 4): one touch-tall line of runs
//! at the glass's bottom edge, under the box, that says what the focused
//! conversation runs on and how it stands. Pure and host-tested —
//! which segments, in which order, what each says, which drop as the width
//! runs out, what the banner line says and whether a dropup's list is fresh;
//! `shell/status.rs` paints them. The desktop's line (lernie `ui::status`,
//! §4.44 item 3), byte for byte in its segments.
//!
//! **Left to right**: the three selectors — provider, model, effort — and
//! priority where the provider's row takes it; then this conversation's
//! diagnostics — its state and how long since it acted, tokens in and out,
//! what it has spent and how full its context is where the server's `agent`
//! row says — and its name, the drawer's door. **Only facts about this
//! conversation**: the seat-wide waiting count and the servers count are not
//! here, and an unpriced spend says nothing rather than `$0`. **The
//! workspace is not here either** (§24 item 4; lernie §4.44): changing it is
//! a whole new list of conversations, providers and roles, not a choice made
//! in the chat, so its knob is the drawer head's and the mark went to the
//! drawer's foot with it.
//!
//! **One line where everything fits, else two** (bl-96fc, [`layout`]): the
//! selectors on the touch-tall rule, the diagnostics on a small line under
//! it. Provider and model never drop: they share what is left and elide
//! inside it; priority, then effort, leave the rule before them. The name
//! never drops either (DESIGN §23 item 14).

use crate::theme::{State, TOUCH};

/// Where a segment stands on the rule, which is also what it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    /// **Never on the rule** (§24 item 4): the drawer head's knob, kept here
    /// as the slot `/workspace` names (`verbs::Does::Knob`), which opens the
    /// drawer with its list open.
    Workspace,
    Provider,
    Model,
    Effort,
    Priority,
    Tokens,
    Spend,
    Context,
    State,
    Name,
}

/// The order segments leave a one-line rule in as the width runs out — the
/// diagnostics from the right, then priority, then effort (the desktop's
/// rule) — and a diagnostic leaving is what puts the diagnostics on a line
/// of their own. The name is not here: it never drops.
pub const DROPS: [Slot; 6] = [
    Slot::Context,
    Slot::Spend,
    Slot::Tokens,
    Slot::State,
    Slot::Priority,
    Slot::Effort,
];

/// The selectors that never drop, and elide inside their share instead.
const KEPT: [Slot; 2] = [Slot::Provider, Slot::Model];

/// **The narrowest a selector is let fall to before a knob leaves**: a few
/// letters and the drop mark. Below it a selector says nothing worth reading,
/// and a knob is the cheaper thing to lose. Read off the bundled faces'
/// advance widths at `SMALL` type, not rounded to touch targets (bl-9817's
/// rule; DESIGN §23 item 14): the door's air either side (8), its mark
/// ` ⏶` (13.5: a space, and the glyph at the icon face's nine-tenths scale)
/// and four letters with their ellipsis (`clau…` is 33.5). The chip it
/// replaces spent 16 of padding and 20 of caret room, which made this 70.
pub const SHARE: f32 = 56.0;

/// The banner's word while the wire is being re-dialled.
pub const RECONNECTING: &str = "reconnecting…";

/// **Which segments stand, and how wide each is**, at `room` points with
/// `gap` between neighbours. `wants` is each segment's own width: its words,
/// the air inside its door and the mark it carries. A dropped segment is
/// `None`. `held` never drops: it is the knob whose list is open, or that a
/// verb just asked to open (`/effort` on a rule too narrow to carry it).
///
/// **The drop is the desktop's** (lernie §4.42 item 11, DESIGN §23 item
/// 14): the diagnostics from the right, then priority, then effort; the
/// two selectors and the name never drop. **The width is paid in two
/// passes**: every other segment that stands gets its words, and the
/// selectors are water-filled into what is left — a short one keeps its
/// words, and the rest share one cut, never below [`SHARE`] where the room
/// holds both. Every segment that stands is at least the touch floor wide.
pub fn fit(
    slots: &[Slot],
    wants: &[f32],
    room: f32,
    gap: f32,
    held: Option<Slot>,
) -> Vec<Option<f32>> {
    let claim: Vec<f32> = wants.iter().map(|want| want.max(TOUCH)).collect();
    let floor = |shown: &[bool]| -> f32 {
        let standing = shown.iter().filter(|s| **s).count();
        let gaps = gap * standing.saturating_sub(1) as f32;
        slots
            .iter()
            .zip(&claim)
            .zip(shown)
            .filter(|(_, s)| **s)
            .map(|((slot, want), _)| if KEPT.contains(slot) { SHARE } else { *want })
            .sum::<f32>()
            + gaps
    };
    let mut shown = vec![true; slots.len()];
    for drop in DROPS.into_iter().filter(|drop| Some(*drop) != held) {
        if floor(&shown) <= room {
            break;
        }
        for (slot, s) in slots.iter().zip(shown.iter_mut()) {
            if *slot == drop {
                *s = false;
            }
        }
    }
    let standing: Vec<(Slot, f32)> = slots
        .iter()
        .zip(&claim)
        .zip(&shown)
        .filter(|(_, s)| **s)
        .map(|((slot, want), _)| (*slot, *want))
        .collect();
    let mut widths = paid(&standing, room, gap, floor(&shown) <= room).into_iter();
    shown
        .iter()
        .map(|s| if *s { widths.next() } else { None })
        .collect()
}

/// **The two passes**, over what stands: where the floor fits, the others
/// at their words and the selectors water-filled into the rest; where even
/// it does not (a glass narrower than the two selectors), one max-min share
/// over everything, the selectors asking for all of it.
fn paid(standing: &[(Slot, f32)], room: f32, gap: f32, fits: bool) -> Vec<f32> {
    let kept = |slot: &Slot| KEPT.contains(slot);
    if !fits {
        let wants: Vec<f32> = standing
            .iter()
            .map(|(slot, want)| if kept(slot) { f32::INFINITY } else { *want })
            .collect();
        return crate::shell::place::row::widths(room, gap, &wants);
    }
    let others: f32 = standing
        .iter()
        .filter(|(s, _)| !kept(s))
        .map(|(_, w)| w)
        .sum();
    let gaps = gap * standing.len().saturating_sub(1) as f32;
    let selectors: Vec<f32> = standing
        .iter()
        .filter(|(s, _)| kept(s))
        .map(|(_, w)| *w)
        .collect();
    let mut filled =
        crate::shell::place::row::widths((room - others - gaps).max(0.0), 0.0, &selectors)
            .into_iter();
    standing
        .iter()
        .map(|(slot, want)| {
            if kept(slot) {
                filled.next().unwrap_or(0.0)
            } else {
                *want
            }
        })
        .collect()
}

/// **The banner line over the rule**: a sentence this seat said about the
/// operator's own act (a verb's refusal, a dark knob's reason) in the
/// annotation accent; else the server's failure in the error accent unless
/// it was put down; else the redial in the working accent unless it was.
pub fn banner(
    said: Option<&str>,
    error: Option<&str>,
    reconnecting: bool,
    dismissed: Option<&str>,
) -> Option<(String, State)> {
    let standing = |text: &str| dismissed != Some(text);
    said.map(|said| (said.to_owned(), State::Annotation))
        .or_else(|| {
            error
                .filter(|e| standing(e))
                .map(|e| (e.to_owned(), State::Error))
        })
        .or_else(|| {
            (reconnecting && standing(RECONNECTING))
                .then(|| (RECONNECTING.to_owned(), State::Working))
        })
}

/// **What a dropup's reload mark says**: still reloading while the
/// watermark it asked at has not moved, fresh once it has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reload {
    Reloading,
    Fresh,
}

pub fn reload(asked_at: usize, now: usize) -> Reload {
    if now > asked_at {
        Reload::Fresh
    } else {
        Reload::Reloading
    }
}

mod lines;
mod said;
pub use lines::{Place, layout};
pub use said::{Said, count, diagnostics, tokens};

#[cfg(test)]
mod tests;
