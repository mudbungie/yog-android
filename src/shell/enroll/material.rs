//! **The enrollment screen**: what material this device needs, where it goes,
//! how it gets there — and the two controls that act (bl-dd7b).
//!
//! Both controls act on this app's own storage and neither dials, which is
//! what keeps REMOTE §1.4 exactly where it was:
//!
//! - **paste an envelope** — the material a trusted operator-grade seat minted
//!   over *its* authenticated channel, carried here out of channel by eye.
//!   DESIGN §5's third delivery channel; `crate::envelope` is the whole
//!   contract and every refusal it can give is painted here verbatim.
//! - **check for material** — re-run the boot derivation, for the leaf that
//!   arrived by cable or by an already-trusted device's tools while this
//!   screen was open.
//!
//! The field holds a private key while it is full, so it is emptied the moment
//! it has been landed and on the way back out — see `Shell::forget_envelope`.
//! Nothing here logs it.

use eframe::egui;

use super::super::app::ENVELOPE;
use super::{Landing, Scanner};
use crate::bootstrap::Offer;

/// The enrollment screen. Returns whether the material should be re-read —
/// either because an envelope just landed, or because the operator asked.
pub(super) fn screen(
    ui: &mut egui::Ui,
    offer: &Offer,
    landing: &Landing,
    text: &mut String,
    said: &mut Option<String>,
    scanner: &mut Scanner,
) -> bool {
    ui.strong("this device needs");
    for file in crate::material::WANTED {
        ui.label(format!("  · {file}"));
    }
    ui.add_space(4.0);
    ui.strong("put them at");
    ui.label(format!("  {}", landing.dir));
    ui.add_space(8.0);
    ui.label(&offer.how);
    ui.add_space(8.0);
    ui.strong("how it gets here");
    for channel in crate::bootstrap::channels() {
        ui.label(format!("  · {channel}"));
    }
    ui.add_space(12.0);
    ui.separator();
    let landed = envelope(ui, landing, text, said, scanner);
    ui.separator();
    if let Some(why) = &landing.refusal {
        ui.colored_label(egui::Color32::LIGHT_RED, why);
    } else {
        ui.weak("nothing has arrived yet.");
    }
    ui.add_space(4.0);
    // A read of this app's own storage, not a dial: it is the act that makes a
    // cable push land without killing and relaunching the app.
    landed || ui.button("check for material").clicked()
}

/// **The scan screen, and what it produces** (bl-d815). Returns whether
/// material landed.
///
/// The decoded string goes into the paste field and is spent by [`land`] —
/// the paste field's own sink, unchanged. That is the whole relationship
/// between the two controls: a camera is a faster way to fill one text field,
/// and every refusal an envelope can earn is still earned in one place.
pub(super) fn scanned(
    ui: &mut egui::Ui,
    dir: &str,
    text: &mut String,
    said: &mut Option<String>,
    scanner: &mut Scanner,
) -> bool {
    let Some(found) = scanner.run(ui, said) else {
        return false;
    };
    *text = found;
    land(dir, text, said)
}

/// The envelope field and the two controls that spend it. Returns whether
/// material landed.
fn envelope(
    ui: &mut egui::Ui,
    landing: &Landing,
    text: &mut String,
    said: &mut Option<String>,
    scanner: &mut Scanner,
) -> bool {
    ui.strong("or paste the envelope a seat minted");
    ui.weak(format!(
        "one line of JSON beginning {{\"{}\": {}",
        crate::envelope::TAG,
        crate::envelope::VERSION
    ));
    // The field is capped and scrolls INSIDE that cap. A full envelope is a
    // couple of kilobytes of PEM, and a text edit that grows to fit it pushes
    // the button that spends it off the bottom of the screen — under the
    // keyboard, on the one screen where the keyboard is certainly up.
    egui::ScrollArea::vertical()
        .id_salt(ENVELOPE.id)
        .max_height(96.0)
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(text)
                    .id(egui::Id::new(ENVELOPE.id))
                    .desired_width(f32::INFINITY)
                    .desired_rows(3)
                    .hint_text("paste here"),
            );
        });
    let ready = !text.trim().is_empty();
    // **What landing costs, said before the control that spends it**
    // (bl-f12d). One device holds one leaf (§9), so an envelope does not join
    // this device's identities — it REPLACES the one running, overwriting the
    // material in place and destroying a private key nothing on this device
    // or the engine can hand back. Not a dialog and not a confirmation:
    // §13.2 keeps this app unmodal, and a consequence an operator reads
    // beside the button is worth more than one they dismiss to reach it.
    if let Some(running) = &landing.replacing {
        ui.add_space(4.0);
        ui.colored_label(
            egui::Color32::LIGHT_YELLOW,
            format!(
                "this device is {running}. Landing an envelope replaces that \
                 identity: this device's certificate and its private key are \
                 overwritten in place, the old key is destroyed, and getting it \
                 back is a fresh mint from the engine.",
            ),
        );
    }
    ui.add_space(4.0);
    let mut landed = false;
    ui.horizontal(|ui| {
        landed = ui.add_enabled(ready, egui::Button::new("enroll")).clicked()
            && land(&landing.dir, text, said);
        // Beside paste, never instead of it (bl-d815). The camera is one way
        // to fill this field; a laptop screen read by eye is the other, and
        // the second one works on a device with no camera, no permission and
        // no light.
        if ui.button("scan QR").clicked() {
            *said = None;
            scanner.open();
        }
    });
    if let Some(why) = said.as_ref() {
        ui.colored_label(egui::Color32::LIGHT_RED, why);
    }
    landed
}

/// Read the pasted text and write what it carries. Every refusal is the
/// envelope's own sentence — this layer adds none of its own, because a screen
/// that reworded a refusal would be a second place to keep them right.
fn land(dir: &str, text: &str, said: &mut Option<String>) -> bool {
    let outcome = crate::envelope::read(text)
        .and_then(|envelope| crate::envelope::land(std::path::Path::new(dir), &envelope));
    match outcome {
        Ok(()) => {
            *said = None;
            true
        }
        Err(why) => {
            *said = Some(why);
            false
        }
    }
}
