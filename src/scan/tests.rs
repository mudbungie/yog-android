//! The decoder, against a symbol this repo did not encode.
//!
//! **The fixture is foreign on purpose.** A decoder tested against its own
//! library's writer proves the pair agree, not that either is right — so the
//! symbol is checked in as its module matrix, encoded by `segno` (an
//! independent ISO/IEC 18004 implementation) at the exact bar REMOTE §8.4
//! measures: **1567 bytes of compact JSON, version 33, error-correction level
//! M, 149 × 149 modules**. Not a toy string; the real payload, at the real
//! size, in the real version.
//!
//! **Nothing in it is real material.** The envelope carries a throwaway
//! `notreal-` PKI minted for the fixture — two CERTIFICATES, which are public
//! material — and its `key` field is a fabricated filler with no private-key
//! banner, the same discipline REMOTE §8.4 gives the wire corpus. The
//! disclosure gate reads both fixture files on every commit.
//!
//! Regenerating the pair, if the payload contract ever moves (both files are
//! one artifact and must be rewritten together):
//!
//! ```text
//! openssl req -x509 -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 -nodes \
//!   -days 825 -keyout ca.key -out ca.pem -subj "/CN=notreal-ca"
//! openssl req -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 -nodes \
//!   -keyout phone.key -out phone.csr -subj "/CN=notreal-phone/OU=foot"
//! openssl x509 -req -in phone.csr -CA ca.pem -CAkey ca.key -CAcreateserial \
//!   -days 825 -out phone.pem
//! python -c "import json,segno; ...; segno.make(payload, error='m',
//!            mode='byte', boost_error=False)"   # write '#'/'.' per module
//! ```

use super::{Camera, decode, read, refusal, state};
use crate::test_support::scratch;

/// The symbol, one line per module row.
const SYMBOL: &str = include_str!("../../tests/fixtures/enroll-v33m-symbol.txt");
/// The bytes that symbol encodes.
const PAYLOAD: &str = include_str!("../../tests/fixtures/enroll-v33m-payload.json");

/// The fixture as module rows.
fn modules() -> Vec<Vec<bool>> {
    SYMBOL
        .lines()
        .filter(|row| !row.is_empty())
        .map(|row| row.chars().map(|c| c == '#').collect())
        .collect()
}

/// The fixture painted into a camera-shaped luminance frame: a card lighter
/// than the room, dark modules on it, `scale` pixels per module and the
/// four-module quiet zone the format requires, offset so the symbol is not
/// axis-aligned with the frame's own origin.
fn framed(width: usize, height: usize, scale: usize, at: (usize, usize)) -> Vec<u8> {
    const QUIET: usize = 4;
    let rows = modules();
    let side = (rows.len() + 2 * QUIET) * scale;
    let mut luma = vec![190u8; width * height];
    let mut ink = |x: usize, y: usize, v: u8| {
        if x < width
            && y < height
            && let Some(slot) = luma.get_mut(y * width + x)
        {
            *slot = v;
        }
    };
    for y in 0..side {
        for x in 0..side {
            ink(at.0 + x, at.1 + y, 230);
        }
    }
    for (my, row) in rows.iter().enumerate() {
        for (mx, on) in row.iter().enumerate() {
            if !on {
                continue;
            }
            for dy in 0..scale {
                for dx in 0..scale {
                    ink(
                        at.0 + (mx + QUIET) * scale + dx,
                        at.1 + (my + QUIET) * scale + dy,
                        30,
                    );
                }
            }
        }
    }
    luma
}

/// One frame in the bridge's own wire shape: the two big-endian sides, then
/// the plane.
fn frame(width: usize, height: usize, luma: &[u8]) -> Vec<u8> {
    let sides = [width as u16, height as u16];
    let mut out: Vec<u8> = sides.iter().flat_map(|s| s.to_be_bytes()).collect();
    out.extend_from_slice(luma);
    out
}

/// **The fixture is what it says it is.** A guard on the fixture rather than
/// on the code: a regenerated pair that quietly dropped to a smaller version
/// would turn every test below into a test of an easier problem.
#[test]
fn the_fixture_is_a_version_33_symbol_of_a_full_envelope() {
    assert_eq!(modules().len(), 149);
    assert!(modules().iter().all(|row| row.len() == 149));
    assert_eq!(PAYLOAD.trim_end().len(), 1567);
}

/// **The whole path, photons to component.** A camera frame carrying the
/// envelope decodes to the exact bytes, and those bytes go into the paste
/// sink unchanged — the same [`crate::envelope::read`] and
/// [`crate::envelope::land`] the enroll button spends, with the
/// grade-versus-certificate law doing its own work — and what comes up is the
/// component the certificate says, not the one the envelope claims.
#[test]
fn a_scanned_frame_lands_material_the_boot_derivation_reads_back() {
    let luma = framed(1280, 720, 4, (300, 20));
    let text = read(&frame(1280, 720, &luma)).expect("no symbol found");
    assert_eq!(text, PAYLOAD.trim_end());

    let envelope = crate::envelope::read(&text).unwrap();
    assert_eq!(envelope.grade, crate::leaf::Grade::Foot);
    assert_eq!(envelope.name, "notreal-phone");

    let wire = scratch().join("wire");
    crate::envelope::land(&wire, &envelope).unwrap();
    let crate::bootstrap::Standing::Enrolled(enrolled) = crate::bootstrap::standing(&wire).unwrap()
    else {
        panic!("not enrolled");
    };
    assert_eq!(enrolled.component, crate::bootstrap::Component::Foot);
    assert_eq!(enrolled.client, "notreal-phone");
}

/// The emulator's own geometry, and the smallest frame worth pointing at a
/// 149-module symbol: 640 × 480 leaves barely two pixels per module, which is
/// the floor the format allows and the one a hand-held scan actually lands
/// on.
#[test]
fn the_symbol_reads_at_two_pixels_per_module() {
    let luma = framed(640, 480, 2, (150, 70));
    assert_eq!(decode(&luma, 640, 480).as_deref(), Some(PAYLOAD.trim_end()));
}

/// An empty room is not an error — it is the frame before the next one.
#[test]
fn a_frame_with_no_symbol_in_it_is_simply_not_a_read() {
    assert_eq!(decode(&vec![200u8; 640 * 480], 640, 480), None);
}

/// A plane whose length contradicts its stated sides is refused rather than
/// read out of bounds — rxing's own dimension check, which is why this layer
/// does no arithmetic of its own.
#[test]
fn a_plane_that_does_not_match_its_dimensions_is_refused() {
    assert_eq!(decode(&[0u8; 16], 640, 480), None);
    assert_eq!(read(&frame(640, 480, &[0u8; 16])), None);
}

/// A buffer too short to carry even the two sides. The slice pattern is what
/// makes this a refusal instead of an index.
#[test]
fn a_frame_shorter_than_its_own_header_is_refused() {
    assert_eq!(read(&[]), None);
    assert_eq!(read(&[0, 1, 2]), None);
}

/// Every word the bridge speaks, and the two ways it can say something this
/// build has no word for.
#[test]
fn the_bridge_vocabulary_reads_back() {
    assert_eq!(state("granted"), Camera::Granted);
    assert_eq!(state("asking\n"), Camera::Asking);
    assert_eq!(state("denied"), Camera::Denied);
    assert_eq!(state("unasked"), Camera::Unasked);
    assert_eq!(
        state("err\nno back-facing lens"),
        Camera::Broken("no back-facing lens".to_owned())
    );
    assert_eq!(
        state("wat"),
        Camera::Broken("the camera bridge answered \"wat\"".to_owned())
    );
}

/// Which states hand the screen back to the paste field, and which let it
/// stay up. Asking is the one that could plausibly be either, and it is not a
/// refusal: the operator is looking at the system dialog.
#[test]
fn only_a_refusal_or_a_broken_camera_closes_the_scanner() {
    assert_eq!(refusal(&Camera::Granted), None);
    assert_eq!(refusal(&Camera::Asking), None);
    assert_eq!(refusal(&Camera::Unasked), None);
    let denied = refusal(&Camera::Denied).unwrap();
    assert!(denied.contains("paste the envelope instead"), "{denied}");
    let broken = refusal(&Camera::Broken("in use".to_owned())).unwrap();
    assert!(broken.contains("in use"), "{broken}");
    assert!(broken.contains("paste the envelope instead"), "{broken}");
}
