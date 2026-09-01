//! The walk, verified where upstream verified its raster: the census of
//! shapes, the layer order, the geometry's own promises (tangency, equal
//! legs, the pinched lune) and the hue drive. No renderer runs here — the
//! host gate never compiles egui — so what is asserted is the one list every
//! renderer walks.

use super::arc::Shape;
use super::{CASING_W, CONDUCTOR_W, HYDRA, MAIN_R, NODE_R, PHOSPHOR, STEPS, VOID_DEEP, deep, mark};

const EPS: f32 = 1e-4;

fn traces(walk: &[Shape]) -> Vec<(usize, f32)> {
    walk.iter()
        .filter_map(|shape| match shape {
            Shape::Trace { path, width, .. } => Some((path.len(), *width)),
            _ => None,
        })
        .collect()
}

fn discs(walk: &[Shape]) -> Vec<(f32, f32, f32, [u8; 4])> {
    walk.iter()
        .filter_map(|shape| match shape {
            Shape::Disc {
                cx,
                cy,
                radius,
                fill,
            } => Some((*cx, *cy, *radius, *fill)),
            _ => None,
        })
        .collect()
}

#[test]
fn the_walk_is_six_traces_ten_discs_and_one_lune_in_layer_order() {
    let walk = mark();
    let laid = traces(&walk);
    assert_eq!(laid.len(), 6, "three casings and three conductors");
    // Every trace carries the whole sampled arc: STEPS spans, STEPS+1 points.
    let points = usize::from(STEPS) + 1;
    assert!(laid.iter().all(|(len, _)| *len == points), "{laid:?}");
    // Casing first, conductor over it — never the other way round.
    assert!(laid.iter().take(3).all(|(_, w)| (w - CASING_W).abs() < EPS));
    assert!(
        laid.iter()
            .skip(3)
            .all(|(_, w)| (w - CONDUCTOR_W).abs() < EPS)
    );
    assert_eq!(discs(&walk).len(), 10, "nine seats and the eye");
    // The lune is last: the pupil rides over everything.
    assert!(matches!(walk.last(), Some(Shape::Lune { .. })));
}

#[test]
fn the_eye_is_the_largest_disc_and_sits_dead_centre() {
    let painted = discs(&mark());
    let eye = painted
        .iter()
        .copied()
        .reduce(|best, one| if one.2 > best.2 { one } else { best })
        .unwrap_or_default();
    assert!((eye.0 - 0.5).abs() < EPS && (eye.1 - 0.5).abs() < EPS);
    assert!((eye.2 - MAIN_R).abs() < EPS);
}

#[test]
fn every_circle_is_the_one_phosphor_hue_and_it_is_driven_not_pasted() {
    let painted = discs(&mark());
    let phosphor = deep(HYDRA, PHOSPHOR);
    assert!(painted.iter().all(|(_, _, _, fill)| *fill == phosphor));
    assert_ne!(phosphor, HYDRA, "the hue is driven, never the raw entry");
    assert_eq!(phosphor[1], 255, "driven past its peak the green is full");
}

#[test]
fn the_casing_is_the_same_hue_held_under_the_conductor() {
    let walk = mark();
    let fills: Vec<[u8; 4]> = walk
        .iter()
        .filter_map(|shape| match shape {
            Shape::Trace { fill, .. } => Some(*fill),
            _ => None,
        })
        .collect();
    let (casing, conductor) = (fills[0], fills[3]);
    assert!(casing[1] < conductor[1], "the casing sits under the wire");
    assert!(
        casing[1] > casing[0] && casing[1] > casing[2],
        "and is still green"
    );
}

#[test]
fn the_pupil_is_a_pinched_lune_filled_with_the_void() {
    let walk = mark();
    let Some(Shape::Lune { ribs, fill }) = walk.last() else {
        panic!("the walk ends on the pupil");
    };
    assert_eq!(*fill, VOID_DEEP, "the pupil is the void, not a hole");
    assert_eq!(ribs.len(), usize::from(STEPS) + 1);
    for end in [ribs.first(), ribs.last()].into_iter().flatten() {
        let (dx, dy) = (end.out.0 - end.back.0, end.out.1 - end.back.1);
        assert!(dx.hypot(dy) < EPS, "the arcs meet at a true point");
    }
}

#[test]
fn the_whole_mark_stays_inside_the_unit_square() {
    let inside = |(x, y): (f32, f32)| (0.0..=1.0).contains(&x) && (0.0..=1.0).contains(&y);
    for shape in mark() {
        match shape {
            Shape::Trace { path, .. } => assert!(path.into_iter().all(inside)),
            Shape::Lune { ribs, .. } => {
                assert!(
                    ribs.into_iter()
                        .all(|rib| inside(rib.out) && inside(rib.back))
                );
            }
            Shape::Disc { cx, cy, radius, .. } => {
                assert!(inside((cx - radius, cy - radius)) && inside((cx + radius, cy + radius)));
            }
        }
    }
}

/// The first arm's first seat is the circle tangent to the eye at bottom
/// dead centre, and even spacing along its arc gives equal legs — the two
/// sentences the walk's doc makes, checked off the emitted discs.
#[test]
fn an_arm_starts_tangent_to_the_eye_and_its_legs_come_out_equal() {
    let painted = discs(&mark());
    let (x0, y0) = (painted[0].0, painted[0].1);
    assert!((x0 - 0.5).abs() < EPS);
    assert!((y0 - (0.5 + MAIN_R + NODE_R)).abs() < EPS, "tangent below");
    let leg =
        |a: (f32, f32, f32, [u8; 4]), b: (f32, f32, f32, [u8; 4])| (a.0 - b.0).hypot(a.1 - b.1);
    let (first, second) = (leg(painted[0], painted[1]), leg(painted[1], painted[2]));
    assert!((first - second).abs() < 1e-3, "{first} vs {second}");
}

#[test]
fn deepening_the_palette_hue_takes_the_pastel_out_of_it() {
    let full = deep(HYDRA, 1.0);
    assert!(full[0] < HYDRA[0] / 3, "the white component goes");
}
