//! **The four bubbles, by role** (DESIGN §23 item 7; lernie §4.45 item 7;
//! `docs/STYLE.md` §5, *a block*): the fill that stands under each block of
//! the transcript, named once here and derived from the palette — no new
//! literal.
//!
//! The operator's is the one hued fill, the brand at a faint alpha: *I'm
//! reading the message I wrote* is answered by hue. The model's three are the
//! ground ladder read as an elevation ramp rising toward what is read — a
//! trail on the ground inside a hairline, the lowest step, because its calls
//! are bookkeeping once they have run; intermediate prose one rung up; the
//! turn's final prose on the top rung. Every one is faint, which is the
//! ruling's *not jarring*: a bubble says a role and is read through, never
//! read.
//!
//! **Which bubble a block gets is decided in the projection**
//! (`crate::rows::blocks`), where the turn's shape is known; this file only
//! says what each one looks like. Pure, so the four fills answer to the
//! suite; `shell::theme::anatomy::bubbled` paints one.

use super::{BRAND, GROUND, HAIRLINE, RAISED, Rgb, SURFACE};

/// The role a block of the transcript stands on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bubble {
    /// The operator's own turn: the brand's tint.
    Operator,
    /// The working — the calls, their results, the reasoning: the ground
    /// inside a hairline.
    Trail,
    /// The model's prose a further working follows within the same turn,
    /// and the live turn's prose while it streams: `SURFACE`.
    Intermediate,
    /// The turn's final prose: `RAISED`.
    Final,
}

/// **How faint the operator's tint is**: the brand's alpha out of 255, the
/// desktop's half-tint (lernie `theme::bubble`, `TINT_ALPHA / 2`).
const OPERATOR_ALPHA: u8 = 20;

/// **The fill and the stroke a bubble wears**, the fill as straight RGBA.
/// Three of the four carry no stroke — a block is raised by a tint, never
/// boxed — and the trail's hairline is what keeps the lowest step visible at
/// all on the ground it stands on.
#[must_use]
pub fn bubble(role: Bubble) -> ([u8; 4], Option<Rgb>) {
    let opaque = |rgb: Rgb| [rgb[0], rgb[1], rgb[2], 255];
    match role {
        Bubble::Operator => ([BRAND[0], BRAND[1], BRAND[2], OPERATOR_ALPHA], None),
        Bubble::Trail => (opaque(GROUND), Some(HAIRLINE)),
        Bubble::Intermediate => (opaque(SURFACE), None),
        Bubble::Final => (opaque(RAISED), None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_operators_is_the_one_hued_fill_and_the_trail_the_one_stroked() {
        let (fill, stroke) = bubble(Bubble::Operator);
        assert_eq!(&fill[..3], &BRAND[..]);
        assert!(fill[3] < 64, "faint, never read");
        assert_eq!(stroke, None);
        assert_eq!(bubble(Bubble::Trail), ([10, 8, 15, 255], Some(HAIRLINE)));
        assert_eq!(bubble(Bubble::Intermediate).0, [22, 20, 30, 255]);
        assert_eq!(bubble(Bubble::Final).0, [36, 33, 48, 255]);
    }

    #[test]
    fn the_models_three_rise_toward_what_is_read() {
        let lum = |role| u32::from(bubble(role).0[2]);
        assert!(lum(Bubble::Trail) < lum(Bubble::Intermediate));
        assert!(lum(Bubble::Intermediate) < lum(Bubble::Final));
    }
}
