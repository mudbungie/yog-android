//! Compass work — the only geometry the mark is made of, and the flat-filled
//! shapes it emits. Ported with the walk (bl-ff27, see `super`).
//!
//! Every curve is named the way you would name it with a compass: two
//! endpoints and a **sagitta**, the height of the arc's apex above the chord
//! between them. That fixes one circle exactly, so nothing in the mark is a
//! tuned spline. A [`lune`] is the region between *two* arcs drawn on the
//! same pair of endpoints: pointed at both ends because the arcs meet there,
//! fat in the middle by exactly the difference of their sagittas.

use super::{Hue, STEPS};

/// One cross-section of a ribbon — the two points its edges pass through.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rib {
    pub out: (f32, f32),
    pub back: (f32, f32),
}

/// Everything the mark is made of. Three primitives, one flat fill each;
/// every rendering walks one list of these in one order — which is what
/// makes two renderers the same picture rather than two approximations.
///
/// A [`Trace`](Shape::Trace) is named by its **centreline and width**, not
/// by its edges: that lets a renderer say it in its own primitives (egui's
/// stroker does the joins and the antialiasing). All coordinates are the
/// unit square.
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    /// A constant-width band down the middle of a path.
    Trace {
        path: Vec<(f32, f32)>,
        width: f32,
        fill: Hue,
    },
    /// The crescent between two arcs on one pair of endpoints — a ribbon
    /// whose width varies, so it is named by its ribs and cannot be a stroke.
    Lune { ribs: Vec<Rib>, fill: Hue },
    /// A flat circle.
    Disc {
        cx: f32,
        cy: f32,
        radius: f32,
        fill: Hue,
    },
}

/// The arc from `from` to `to` whose apex stands `bulge` off the chord —
/// positive to the left of the direction of travel — sampled into [`STEPS`]
/// spans. The radius is `(h² + s²) / 2s` for half-chord `h` and sagitta `s`;
/// the centre sits `radius − s` back from the chord's midpoint; and the
/// swept angle is `2·atan2(h, radius − s)`, which needs no special case for
/// the major arc because `radius − s` simply goes negative there.
pub(super) fn arc(from: (f32, f32), to: (f32, f32), bulge: f32) -> Vec<(f32, f32)> {
    let (side, sag) = (bulge.signum(), bulge.abs());
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let span = dx.hypot(dy);
    let half = span / 2.0;
    let radius = half.mul_add(half, sag * sag) / (2.0 * sag);
    let (nx, ny) = (-dy / span * side, dx / span * side);
    let centre = (
        (from.0 + to.0).mul_add(0.5, -nx * (radius - sag)),
        (from.1 + to.1).mul_add(0.5, -ny * (radius - sag)),
    );
    let start = (from.1 - centre.1).atan2(from.0 - centre.0);
    let sweep = 2.0 * half.atan2(radius - sag);
    (0..=STEPS)
        .map(|step| {
            let along = f32::from(step) / f32::from(STEPS);
            let angle = side.mul_add(-(sweep * along), start);
            (
                radius.mul_add(angle.cos(), centre.0),
                radius.mul_add(angle.sin(), centre.1),
            )
        })
        .collect()
}

/// The crescent between two arcs on one pair of endpoints. Both are sampled
/// the same way, so rib `i` simply joins their `i`th points — and at the
/// ends those points coincide, which is what brings the shape to a true
/// point rather than a rounded stub.
pub(super) fn lune(from: (f32, f32), to: (f32, f32), out: f32, back: f32, fill: Hue) -> Shape {
    Shape::Lune {
        ribs: arc(from, to, out)
            .into_iter()
            .zip(arc(from, to, back))
            .map(|(out, back)| Rib { out, back })
            .collect(),
        fill,
    }
}
