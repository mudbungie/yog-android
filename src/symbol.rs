//! **The QR symbol this seat draws** (REMOTE §8.4, DESIGN §13.18): the module
//! matrix an enroll envelope encodes to, and the two rules about how big it is
//! painted.
//!
//! **The encoder is `rxing`'s, and that is a feature flag rather than a
//! dependency.** This crate already links rxing to DECODE a symbol (§12), and
//! its `encoders` feature adds no crate to the lockfile at all — measured, not
//! assumed. The manifest's own note said *"no `encoders`: this app reads
//! symbols and never draws one"*, which was true under the chat-first framing
//! and is what the full-seat ruling reverses (DESIGN §16.2). lernie wrote its
//! own encoder because its manifest had no QR library in it; this one does,
//! and a second implementation of a fully specified algorithm would be code
//! this repository maintains forever for nothing.
//!
//! **Level M, because REMOTE §8.4 measures the envelope and rules it.** A real
//! §8.4 envelope is 1567 bytes; a version-40 symbol carries 2331 at M and 1663
//! at Q, so *"PEM as minted, at level M or lower"* — and the margin is asked
//! for as zero here because the quiet zone is the PAINT's, laid out with the
//! rest of the square.
//!
//! **The pitch is a whole number of device pixels** (lernie DESIGN §4.29's
//! sibling ruling, bl-5e0e, whose reason transfers exactly). egui feathers
//! every fill by a device pixel, half of it proud of the edge, so at a
//! fractional origin a module's own edge pixels come out grey and the contrast
//! a decoder needs is spent on anti-aliasing. And the symbol is drawn **as
//! large as the surface allows**, because its whole job is to be read by a
//! camera once, before the material is forgotten.

use rxing::Writer;

/// The quiet zone, in modules, on every side. It is the format's own — a
/// symbol without one is a symbol a decoder may not find.
pub const QUIET: usize = 4;

/// The smallest module a symbol may be drawn at, in device pixels. Below one
/// pixel there is nothing to draw.
const FLOOR: f32 = 1.0;

/// One symbol as modules — square, quiet zone excluded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    /// How many modules across (and down) the symbol itself is.
    pub modules: usize,
    dark: Vec<bool>,
}

impl Symbol {
    /// Whether one module is dark. Out of range is light, which is what the
    /// quiet zone is: a paint that walks the padded square asks about every
    /// cell of it and this is the honest answer for the ones outside.
    #[must_use]
    pub fn dark(&self, x: usize, y: usize) -> bool {
        if x >= self.modules || y >= self.modules {
            return false;
        }
        self.dark
            .get(y * self.modules + x)
            .copied()
            .unwrap_or(false)
    }
}

/// Encode one payload as a symbol at level M.
///
/// It refuses rather than truncating: a payload too large for the largest
/// symbol at this level is a picture nobody could scan, and REMOTE §8.4's own
/// rule is that the envelope fits at M.
pub fn encode(text: &str) -> Result<Symbol, String> {
    let hints = rxing::EncodeHints {
        ErrorCorrection: Some("M".to_owned()),
        Margin: Some("0".to_owned()),
        ..rxing::EncodeHints::default()
    };
    let matrix = rxing::qrcode::QRCodeWriter
        .encode_with_hints(text, &rxing::BarcodeFormat::QR_CODE, 0, 0, &hints)
        .map_err(|why| format!("the envelope will not encode as a symbol: {why}"))?;
    let modules = usize::try_from(matrix.getWidth())
        .map_err(|_| "the symbol is wider than this device can address".to_owned())?;
    let mut dark = Vec::with_capacity(modules * modules);
    for y in 0..matrix.getHeight() {
        for x in 0..matrix.getWidth() {
            dark.push(matrix.get(x, y));
        }
    }
    Ok(Symbol { modules, dark })
}

/// **How big one module is painted**, in points: as large as the square
/// allows, floored at one device pixel, and always a whole number of them.
///
/// It is pure and host-tested for `shell::place`'s reason exactly — the rule
/// is what is right or wrong, and the paint that spends it is a `Mesh`.
#[must_use]
pub fn pitch(side: f32, points_per_pixel: f32, modules: usize) -> f32 {
    let across = modules.saturating_add(QUIET * 2);
    let ppp = if points_per_pixel > 0.0 {
        points_per_pixel
    } else {
        FLOOR
    };
    let across = if across == 0 { 1 } else { across };
    let want = side * ppp / f64::from(u32::try_from(across).unwrap_or(u32::MAX)) as f32;
    let pixels = if want.is_finite() {
        want.floor()
    } else {
        FLOOR
    };
    pixels.max(FLOOR) / ppp
}

#[cfg(test)]
mod tests;
