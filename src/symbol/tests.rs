//! The symbol, proved by the decoder this app already links.
//!
//! **An encoder that agrees with itself proves nothing**, which is lernie's
//! own reason for pinning its encoder against an independent implementation.
//! Here the independent reading is free: this crate decodes symbols (§12), so
//! what is asserted is a ROUND TRIP through rxing's reader over the module
//! matrix — the same path a camera takes, minus the camera.

use super::{QUIET, encode, pitch};

/// Read a symbol back the way `crate::scan` reads a camera frame: one
/// luminance byte per pixel, dark modules black, with the quiet zone the
/// paint would draw.
fn decoded(symbol: &super::Symbol) -> String {
    let scale = 4;
    let across = (symbol.modules + QUIET * 2) * scale;
    let mut luma = vec![0xFF_u8; across * across];
    for y in 0..symbol.modules {
        for x in 0..symbol.modules {
            if !symbol.dark(x, y) {
                continue;
            }
            for dy in 0..scale {
                for dx in 0..scale {
                    let py = (y + QUIET) * scale + dy;
                    let px = (x + QUIET) * scale + dx;
                    if let Some(cell) = luma.get_mut(py * across + px) {
                        *cell = 0;
                    }
                }
            }
        }
    }
    let width = u32::try_from(across).unwrap();
    crate::scan::decode(&luma, width, width).expect("the symbol reads back")
}

#[test]
fn a_payload_encodes_to_a_symbol_that_reads_back_as_itself() {
    let payload = r#"{"yog-enroll":1,"grade":"foot","name":"phone-1"}"#;
    let symbol = encode(payload).unwrap();
    assert!(
        symbol.modules >= 21 && symbol.modules % 4 == 1,
        "a version's side is 4v+17"
    );
    assert_eq!(decoded(&symbol), payload);
}

#[test]
fn a_realistic_envelope_still_fits_at_level_m() {
    // REMOTE §8.4 measures a real mint at 1567 bytes and rules "PEM as minted,
    // at level M or lower" — level M carries 2331. This is that measurement
    // taken against the encoder this seat actually uses.
    let payload = "x".repeat(1567);
    let symbol = encode(&payload).unwrap();
    assert_eq!(decoded(&symbol), payload);
}

#[test]
fn a_payload_too_large_for_the_format_refuses_rather_than_truncating() {
    let why = encode(&"x".repeat(4000)).unwrap_err();
    assert!(
        why.starts_with("the envelope will not encode as a symbol"),
        "{why}"
    );
}

#[test]
fn nothing_outside_the_symbol_is_dark_and_that_is_what_a_quiet_zone_is() {
    let symbol = encode("hello").unwrap();
    assert!(!symbol.dark(symbol.modules, 0));
    assert!(!symbol.dark(0, symbol.modules));
}

#[test]
fn the_pitch_is_whole_device_pixels_and_as_large_as_the_square_allows() {
    // 21 modules plus two quiet zones is 29 across. At 400 points and two
    // device pixels per point that is 800 pixels: 27 whole pixels each, which
    // is 13.5 points — a fractional POINT pitch, which is the point of the
    // rule: what has to be whole is the pixel.
    let pitch = pitch(400.0, 2.0, 21);
    assert!((pitch * 2.0 - 27.0).abs() < f32::EPSILON, "{pitch}");
    assert!(pitch * 29.0 <= 400.0, "it fits: {pitch}");
}

#[test]
fn a_square_of_no_finite_size_still_answers_a_pixel() {
    // egui hands out an infinite `available_width` in an unbounded layout, and
    // a pitch derived from one is not a number. The floor is the answer there
    // too: a symbol at one pixel a module is small, and a symbol at NaN paints
    // nothing at all.
    assert!((pitch(f32::INFINITY, 2.0, 21) - 0.5).abs() < f32::EPSILON);
}

#[test]
fn a_square_too_small_for_one_pixel_a_module_still_answers_one() {
    // The floor is a pixel: below it there is nothing to draw, and a zero
    // pitch would paint an invisible symbol rather than a clipped one.
    assert!((pitch(1.0, 1.0, 177) - 1.0).abs() < f32::EPSILON);
    // A device that reports no scale at all is read as one pixel per point
    // rather than dividing by zero.
    assert!((pitch(29.0, 0.0, 21) - 1.0).abs() < f32::EPSILON);
}
