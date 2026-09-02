//! The application mark — the congeries as a circuit, ported from the yog
//! crate's own generator (its DESIGN §11; removed from that tree with the
//! window and recovered from its history, bl-ff27).
//!
//! Three circles sit tangent to a central one, 120° apart with one at bottom
//! dead centre. From each, an arm runs 60° of arc to a further circle; the
//! circles between ride that arc, joined by a trace of dim casing under a
//! bright phosphor conductor. The centre carries a slit pupil in the void's
//! own black. Two hues, both derived from one palette entry, on transparency.
//!
//! **Everything is compass work.** No spline, no easing: an arc is named by
//! two endpoints and the height of its apex above the chord (`arc::arc`).
//! The pupil is a *lune*, the sliver between two arcs sharing a pair of
//! endpoints, pointed at each end because that is where the arcs meet — and
//! it is **filled, not punched**: cut as negative space it showed the traces
//! converging beneath it.
//!
//! **This port is the mark at rest, and it is pure.** Upstream walks the
//! same list three ways — raster, SVG, live egui layer — and tints each
//! circle by what an agent is doing. This seat needs one emission (the
//! standing mark control) and no tints, and the host gate never compiles
//! egui (Cargo.toml's target gate), so the walk here speaks tuples and raw
//! RGBA and the egui conversion is `shell/mark.rs`'s ten lines. The dropped
//! machinery grows back if a surface needs it; it is not an omission.

mod arc;
pub mod drawable;

use arc::lune;
pub use arc::{Rib, Shape};

#[cfg(test)]
mod tests;

/// A flat fill, straight-alpha RGBA — the walk's whole colour vocabulary.
pub type Hue = [u8; 4];

/// The one palette entry the mark is built from, and the void its pupil is
/// filled with — the yog theme's `HYDRA` and `VOID_DEEP`, restated here
/// because this crate has no theme module to derive them from. If a theme
/// ever lands, these move into it; two homes for one hue is the defect.
const HYDRA: Hue = [110, 222, 148, 255];
const VOID_DEEP: Hue = [10, 8, 15, 255];

/// Arms, and the degrees between them — the triskele's three-fold turn.
const ARMS: u8 = 3;
const TURN: f32 = 120.0;
/// Points sampled along one arc. Divisible by `CIRCLES - 1`, so the seats
/// fall on sampled points exactly rather than between them.
pub(crate) const STEPS: u16 = 48;
/// Where the first arm's tangent circle sits, in degrees: bottom dead centre.
const BASE: f32 = 90.0;
/// The middle circle, and the radius of every other circle — all equal.
const MAIN_R: f32 = 0.132;
const NODE_R: f32 = 0.048;
/// An arm ends this far out, and this many degrees of arc from where it
/// began — the pair that puts the top-right arm's last circle straight up.
const END_R: f32 = 0.398;
const SWEEP: f32 = 60.0;
/// Circles an arm, counting the tangent one it starts on and the one it ends
/// on.
const CIRCLES: u8 = 3;
/// **The only free parameter.** The sagitta of an arm's arc, as a fraction
/// of the chord between its two pinned ends: at 0.448 the arm leaves its
/// tangent circle at exactly 45° off the radial.
const SWELL: f32 = 0.448;
/// The dim casing, and the bright conductor laid down its middle.
const CASING_W: f32 = 0.054;
const CONDUCTOR_W: f32 = 0.016;
/// The pupil's half-height and bulge, as fractions of the middle circle.
const PUPIL_LONG: f32 = 0.78;
const PUPIL_BULGE: f32 = 0.30;
/// How hard [`deep`] drives the one hue: the conductor past the hue's own
/// peak until it is phosphor, the casing well under it.
const PHOSPHOR: f32 = 1.15;
const CASING: f32 = 0.45;
/// How much of the palette hue's white component [`deep`] strips.
const WHITE_CUT: f32 = 0.85;

/// The palette hue with its white component stripped and its value scaled.
/// The badge hue is pastel — tuned to read as a small mark on a dark panel —
/// and a logo wants it saturated; driven past 1.0 the green goes phosphor.
fn deep(hue: Hue, value: f32) -> Hue {
    let (red, green, blue) = (f32::from(hue[0]), f32::from(hue[1]), f32::from(hue[2]));
    let high = red.max(green).max(blue);
    let low = red.min(green).min(blue) * WHITE_CUT;
    let pure =
        |channel: f32| (high * (channel - low) / (high - low) * value).clamp(0.0, 255.0) as u8;
    [pure(red), pure(green), pure(blue), 255]
}

/// The unit-square point `radius` out from the centre at `degrees`.
fn polar(radius: f32, degrees: f32) -> (f32, f32) {
    let turn = degrees.to_radians();
    (
        radius.mul_add(turn.cos(), 0.5),
        radius.mul_add(turn.sin(), 0.5),
    )
}

/// One arm's two **pinned** points: a circle tangent to the middle one, and
/// a circle [`SWEEP`] degrees of arc counter-clockwise of it at [`END_R`].
fn pins(turn: u8) -> ((f32, f32), (f32, f32)) {
    let base = BASE + TURN * f32::from(turn);
    (polar(MAIN_R + NODE_R, base), polar(END_R, base - SWEEP))
}

/// The arc between them, bowed by [`SWELL`] of the chord.
fn trace(turn: u8) -> Vec<(f32, f32)> {
    let (from, to) = pins(turn);
    let chord = (to.0 - from.0).hypot(to.1 - from.1);
    arc::arc(from, to, SWELL * chord)
}

/// Where an arm's circles sit: [`CIRCLES`] points spaced evenly along its
/// arc. Even spacing on a circular arc means equal chords, so the legs
/// between them come out equal without anyone asking.
fn seats(turn: u8) -> Vec<(f32, f32)> {
    let path = trace(turn);
    let stride = usize::from(STEPS / u16::from(CIRCLES - 1));
    let mut out = Vec::new();
    for step in 0..CIRCLES {
        if let Some(seat) = path.get(usize::from(step) * stride) {
            out.push(*seat);
        }
    }
    out
}

/// The whole mark at rest, back to front — **by layer, not by arm**: every
/// casing goes down, then every conductor, then every circle, then the eye
/// over the place the arms converge. Drawing each arm complete in turn looks
/// equivalent and is not — the arms overlap near the middle, so a later
/// one's casing would paint over an earlier one's conductor.
///
/// Coordinates are the unit square; a renderer maps them to whatever rect it
/// holds, keeping the square (the mark is round, and a stretched one is a
/// different picture).
pub fn mark() -> Vec<Shape> {
    let (conductor, casing) = (deep(HYDRA, PHOSPHOR), deep(HYDRA, CASING));
    let traces: Vec<Vec<(f32, f32)>> = (0..ARMS).map(trace).collect();
    let mut out = Vec::new();
    for path in &traces {
        out.push(Shape::Trace {
            path: path.clone(),
            width: CASING_W,
            fill: casing,
        });
    }
    for path in &traces {
        out.push(Shape::Trace {
            path: path.clone(),
            width: CONDUCTOR_W,
            fill: conductor,
        });
    }
    for seat in (0..ARMS).flat_map(seats) {
        out.push(Shape::Disc {
            cx: seat.0,
            cy: seat.1,
            radius: NODE_R,
            fill: conductor,
        });
    }
    out.push(Shape::Disc {
        cx: 0.5,
        cy: 0.5,
        radius: MAIN_R,
        fill: conductor,
    });
    let reach = MAIN_R * PUPIL_LONG;
    out.push(lune(
        (0.5, 0.5 - reach),
        (0.5, 0.5 + reach),
        MAIN_R * PUPIL_BULGE,
        -MAIN_R * PUPIL_BULGE,
        VOID_DEEP,
    ));
    out
}
