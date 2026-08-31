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
use crate::bootstrap::Offer;

/// The enrollment screen. Returns whether the material should be re-read —
/// either because an envelope just landed, or because the operator asked.
pub(super) fn screen(
    ui: &mut egui::Ui,
    offer: &Offer,
    dir: &str,
    refusal: Option<&String>,
    text: &mut String,
    said: &mut Option<String>,
) -> bool {
    ui.strong("this device needs");
    for file in crate::material::WANTED {
        ui.label(format!("  · {file}"));
    }
    ui.add_space(4.0);
    ui.strong("put them at");
    ui.label(format!("  {dir}"));
    ui.add_space(8.0);
    ui.label(&offer.how);
    ui.add_space(8.0);
    ui.strong("how it gets here");
    for channel in crate::bootstrap::channels() {
        ui.label(format!("  · {channel}"));
    }
    ui.add_space(12.0);
    ui.separator();
    let landed = envelope(ui, dir, text, said);
    ui.separator();
    if let Some(why) = refusal {
        ui.colored_label(egui::Color32::LIGHT_RED, why);
    } else {
        ui.weak("nothing has arrived yet.");
    }
    ui.add_space(4.0);
    // A read of this app's own storage, not a dial: it is the act that makes a
    // cable push land without killing and relaunching the app.
    landed || ui.button("check for material").clicked()
}

/// The envelope field and its one button. Returns whether material landed.
fn envelope(ui: &mut egui::Ui, dir: &str, text: &mut String, said: &mut Option<String>) -> bool {
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
    let landed =
        ui.add_enabled(ready, egui::Button::new("enroll")).clicked() && land(dir, text, said);
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
