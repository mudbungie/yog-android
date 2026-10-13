//! **One line or two** (DESIGN §22 item 2, bl-96fc). At a phone's own width
//! the selectors fill the rule, and a single line's drop order would take the
//! conversation's state, tokens and name off the glass — the facts the
//! operator ruled the bottom is for. So where one line will not hold both
//! halves, the selectors keep the touch-tall rule and the diagnostics stand
//! on a small line directly under it; where it will (a tablet, landscape),
//! everything stays on one line.
//!
//! **The second line lets go of the spend, then the context, then the
//! tokens** — the spend (bl-a7ec) first, so the ruled order of the other two
//! stands as it was; the state and the name never leave it and elide inside
//! what is left, because they are what the line exists to keep.

use super::{Slot, fit};

/// **Where a segment stands**, and how wide it is there.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Place {
    /// On the touch-tall rule.
    Rule(f32),
    /// On the small line under it.
    Under(f32),
    /// Nowhere: the width let it go.
    Off,
}

/// The order the second line lets diagnostics go in.
const UNDER_DROPS: [Slot; 3] = [Slot::Spend, Slot::Context, Slot::Tokens];

/// Whether a segment is one of the conversation's diagnostics (or its name)
/// rather than a selector.
fn diagnostic(slot: Slot) -> bool {
    matches!(
        slot,
        Slot::Tokens | Slot::Spend | Slot::Context | Slot::State | Slot::Name
    )
}

/// **Where every segment stands**: all on the rule when one line of `room`
/// points holds them all (nothing dropped); else the selectors fitted to the
/// rule ([`fit`]) and the diagnostics to a line of `under` points beneath it.
/// `held` is [`fit`]'s: the knob whose list is open never drops.
pub fn layout(
    slots: &[Slot],
    wants: &[f32],
    room: f32,
    under: f32,
    gap: f32,
    held: Option<Slot>,
) -> Vec<Place> {
    let one = fit(slots, wants, room, gap, held);
    if one.iter().all(Option::is_some) {
        return one.into_iter().flatten().map(Place::Rule).collect();
    }
    let pick = |low: bool| -> (Vec<Slot>, Vec<f32>) {
        slots
            .iter()
            .zip(wants)
            .filter(|(slot, _)| diagnostic(**slot) == low)
            .map(|(slot, want)| (*slot, *want))
            .unzip()
    };
    let (top, top_wants) = pick(false);
    let (low, low_wants) = pick(true);
    let mut top = fit(&top, &top_wants, room, gap, held).into_iter();
    let mut low = below(&low, &low_wants, under, gap).into_iter();
    slots
        .iter()
        .map(|slot| {
            let placed = if diagnostic(*slot) {
                low.next().flatten().map(Place::Under)
            } else {
                top.next().flatten().map(Place::Rule)
            };
            placed.unwrap_or(Place::Off)
        })
        .collect()
}

/// The second line: the spend, the context, then the tokens, leave while what stands
/// would overrun `room`; what is left shares it, the longest eliding first.
fn below(slots: &[Slot], wants: &[f32], room: f32, gap: f32) -> Vec<Option<f32>> {
    let need = |shown: &[bool]| -> f32 {
        let standing: Vec<f32> = wants
            .iter()
            .zip(shown)
            .filter(|(_, s)| **s)
            .map(|(want, _)| *want)
            .collect();
        standing.iter().sum::<f32>() + gap * standing.len().saturating_sub(1) as f32
    };
    let mut shown = vec![true; slots.len()];
    for drop in UNDER_DROPS {
        if need(&shown) <= room {
            break;
        }
        for (slot, s) in slots.iter().zip(shown.iter_mut()) {
            if *slot == drop {
                *s = false;
            }
        }
    }
    let kept: Vec<f32> = wants
        .iter()
        .zip(&shown)
        .filter(|(_, s)| **s)
        .map(|(want, _)| *want)
        .collect();
    let mut widths = crate::shell::place::row::widths(room, gap, &kept).into_iter();
    shown
        .iter()
        .map(|s| if *s { widths.next() } else { None })
        .collect()
}

#[cfg(test)]
mod tests;
