//! **The QR decoder** (bl-d815): one camera luminance frame in, the text a
//! symbol carries out — and nothing else.
//!
//! **This module is a PRODUCER, not a second sink.** The enroll envelope has
//! exactly one reader ([`crate::envelope::read`]) and exactly one landing
//! ([`crate::envelope::land`]), both proved against real minted material, and
//! a scan produces the same string a paste produces. Nothing here validates,
//! nothing here writes a file, and nothing here knows what an envelope is —
//! `crate::envelope`'s module doc already ruled it: *"a decoder, when one is
//! adjudicated (bl-d815), is only a producer of the same string."* The
//! grade-versus-certificate law is untouched because this layer never sees a
//! certificate.
//!
//! **The decoder is `rxing`**, the Rust port of `ZXing`, by operator ruling —
//! the reasoning and the two refused alternatives are recorded at the
//! manifest line, which is where the cost lands.
//!
//! **The frame the camera bridge hands over is the frame the operator sees.**
//! The Android side (`crate::shell::camera`) runs camera2 into an
//! `ImageReader` and hands the Y plane straight across, and the scan screen
//! paints its preview from that same buffer — so there is no second image
//! path that could show one thing and decode another.
//!
//! It is plain host-testable Rust under the 100% floor: the platform is on
//! the other side of a byte slice.

use rxing::common::HybridBinarizer;
use rxing::qrcode::QRCodeReader;
use rxing::{BinaryBitmap, Luma8LuminanceSource, Reader};

/// Read a symbol out of one frame as the camera bridge writes it: a
/// big-endian `u16` width, a big-endian `u16` height, then `width × height`
/// bytes of 8-bit luminance.
///
/// The header rides in the buffer rather than in a second JNI call because
/// the two must describe the SAME frame — a width fetched separately is a
/// width from whichever frame the camera happened to be on when it was
/// asked, and the mismatch it produces is a decode that silently never
/// succeeds. Two bytes per side is not a limit anything meets: the bridge
/// asks for 1280×720.
pub fn read(frame: &[u8]) -> Option<String> {
    let (width, height, luma) = split(frame)?;
    decode(luma, width, height)
}

/// One frame, split into its two sides and its plane.
///
/// The scan screen paints its preview through this same call, so the header's
/// layout has one definition and the pixels shown are the pixels read. Not
/// `pub`: it hands back a borrow of its argument, which is exactly the shape
/// AGENTS.md rule 2 keeps off the crate's public surface.
pub(crate) fn split(frame: &[u8]) -> Option<(u32, u32, &[u8])> {
    let [w0, w1, h0, h1, luma @ ..] = frame else {
        return None;
    };
    Some((
        u32::from(u16::from_be_bytes([*w0, *w1])),
        u32::from(u16::from_be_bytes([*h0, *h1])),
        luma,
    ))
}

/// Decode a luminance plane. `None` is every failure there is — no symbol in
/// frame, a symbol too blurred to read, a plane whose length and stated
/// dimensions disagree — because a scan loop has one thing to do with all
/// three, which is look at the next frame.
pub fn decode(luma: &[u8], width: u32, height: u32) -> Option<String> {
    let source = Luma8LuminanceSource::new(luma.to_vec(), width, height).ok()?;
    let mut image = BinaryBitmap::new(HybridBinarizer::new(source));
    let mut reader = QRCodeReader;
    reader
        .decode(&mut image)
        .ok()
        .map(|found| found.getText().to_owned())
}

/// **What the camera bridge answered when asked whether it can scan.** Four
/// words and a failure, which is the whole vocabulary the Java side speaks —
/// see [`state`] for why the failure is folded in here rather than polled
/// separately.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Camera {
    /// The permission is held; frames can be asked for.
    Granted,
    /// The system dialog is up and the operator has not answered it.
    Asking,
    /// The operator answered, and the answer was no.
    Denied,
    /// Nothing has been asked yet — the state a first-run device is in.
    Unasked,
    /// The camera itself failed: no back lens, a device policy, another app
    /// holding it, a session the framework refused.
    Broken(String),
}

/// The prefix the bridge puts in front of a sentence, mirroring the interface
/// tools' two-line answer protocol (`crate::tools::ui`) one bridge over.
const ERR: &str = "err\n";

/// Read the bridge's answer.
///
/// **The camera's own failures ride the same answer as the permission**, and
/// deliberately: a scan screen asks one question — *can this device scan
/// right now?* — and a second poll for "did the session break" would be a
/// second place for the answer to be no. An answer this build has no word for
/// is [`Camera::Broken`] quoting it, never a silent fifth state.
pub fn state(answer: &str) -> Camera {
    match answer.trim_end() {
        "granted" => Camera::Granted,
        "asking" => Camera::Asking,
        "denied" => Camera::Denied,
        "unasked" => Camera::Unasked,
        said => Camera::Broken(match said.strip_prefix(ERR) {
            Some(why) => why.to_owned(),
            None => format!("the camera bridge answered {said:?}"),
        }),
    }
}

/// **The sentence that closes the scanner and hands the screen back to the
/// paste field**, when there is one.
///
/// `None` is every state the scan screen may stay up in — including
/// [`Camera::Asking`], where the operator is looking at the system dialog and
/// the honest screen behind it is the one they will come back to. A refusal
/// and a broken camera both end the same way, because a dead scan screen with
/// a spinner on it is the failure this whole path exists to avoid: the paste
/// field is the degraded path and it always works.
pub fn refusal(state: &Camera) -> Option<String> {
    match state {
        Camera::Granted | Camera::Asking | Camera::Unasked => None,
        Camera::Denied => Some(
            "the camera permission was refused, so there is nothing to scan with — \
             paste the envelope instead."
                .to_owned(),
        ),
        Camera::Broken(why) => Some(format!(
            "the camera is unavailable: {why} — paste the envelope instead."
        )),
    }
}

#[cfg(test)]
mod tests;
