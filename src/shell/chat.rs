//! **The chat screen**: one conversation, painted from [`crate::rows`].
//!
//! The projection is pure and tested elsewhere; this file is the paint and
//! the one piece of state a projection cannot hold — which rows the operator
//! has flipped by hand. That split is the whole design: what a row IS answers
//! to the suite, and what it looks like answers to the glass.
//!
//! **The fold overrides are ephemeral, as they are upstream.** The two auto
//! knobs are policy and would be durable if this seat had a durable place to
//! put them (REMOTE §7 puts per-seat state on the engine, which is a later
//! ball); a hand-flip is a viewport fact and dies with the screen.
//!
//! **Two lines for a speaking row, one for machinery** — the desktop's own
//! shape. Line one is the coloured stripe and the speaker; line two is the
//! toggle and the payload. Both lines allocate the stripe seat so the
//! toggles stay in one column, which is what makes a tool-heavy transcript
//! scannable at a phone's width.

use eframe::egui;

use crate::rows::{Fold, Role, Row};
use crate::seat::Snapshot;

/// The fold triangles, and the mark a row with nothing to fold shows in their
/// place so every payload starts at the same x.
const GLYPH_COLLAPSED: &str = "▶";
const GLYPH_EXPANDED: &str = "▼";
const NO_FOLD_MARK: &str = "·";

/// The stripe's width in points, and the gap after it.
const STRIPE: f32 = 3.0;

/// How dim an abridged preview paints — the desktop's own solidity for a
/// payload that is not all there, so "there is more" is visible before the
/// triangle is read.
const ABRIDGED: f32 = 0.55;

/// Paint one row and answer whether its toggle was pressed. The caller owns
/// the override set, so this function stays a pure paint over a `&Row`.
pub(crate) fn row(ui: &mut egui::Ui, row: &Row) -> bool {
    let inline = row.fold == Fold::Steps || row.body.is_empty() || !row.expanded;
    let abridged = inline && !row.body.is_empty();
    if let Some(role) = row.role {
        ui.horizontal(|ui| {
            stripe(ui, Some(role));
            ui.colored_label(role_hue(role), &row.prefix);
        });
    }
    let mut toggled = false;
    ui.horizontal(|ui| {
        stripe(ui, None);
        toggled = toggle(ui, row);
        if row.role.is_none() {
            ui.colored_label(tone_hue(ui, row.tone), &row.prefix);
        }
        if inline && !row.preview.is_empty() {
            preview(ui, &row.preview, abridged);
        }
    });
    if !inline {
        ui.horizontal(|ui| {
            stripe(ui, None);
            ui.label(&row.body);
        });
    }
    toggled
}

/// The role stripe, or the blank seat of the same width that keeps every
/// later line's toggle in the same column.
fn stripe(ui: &mut egui::Ui, role: Option<Role>) {
    let height = ui.text_style_height(&egui::TextStyle::Body);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(STRIPE, height), egui::Sense::hover());
    if let Some(role) = role {
        ui.painter().rect_filled(rect, 0.0, role_hue(role));
    }
}

/// The fold triangle — a touch target, not a glyph: a phone has no hover and
/// no Tab, so the whole mark is the button and it is sized for a fingertip.
fn toggle(ui: &mut egui::Ui, row: &Row) -> bool {
    if row.body.is_empty() && row.fold != Fold::Steps {
        ui.monospace(NO_FOLD_MARK);
        return false;
    }
    let glyph = if row.expanded {
        GLYPH_EXPANDED
    } else {
        GLYPH_COLLAPSED
    };
    ui.add(egui::Button::new(egui::RichText::new(glyph).monospace()).frame(false))
        .clicked()
}

/// The contracted payload. An abridged one is dimmed, so a run that ends in
/// `…` reads as incomplete before its triangle is noticed — the desktop's own
/// invariant that anything hidden is hidden behind a triangle.
fn preview(ui: &mut egui::Ui, text: &str, abridged: bool) {
    let mut rich = egui::RichText::new(text);
    if abridged {
        rich = rich.color(ui.visuals().text_color().gamma_multiply(ABRIDGED));
    }
    ui.add(egui::Label::new(rich).truncate());
}

/// A role's hue. The values are the desktop's own, so a transcript read on
/// the phone and on the laptop is the same colour vocabulary.
fn role_hue(role: Role) -> egui::Color32 {
    match role {
        Role::User => egui::Color32::from_rgb(160, 112, 240),
        Role::Model => egui::Color32::from_rgb(118, 188, 242),
        Role::Peer => egui::Color32::from_rgb(232, 176, 96),
        Role::Ended => egui::Color32::from_rgb(184, 152, 104),
    }
}

/// A tone's ink. `Plain` and `Weak` defer to the theme's own text colours;
/// the other four are the desktop's constants.
fn tone_hue(ui: &egui::Ui, tone: crate::codec::Tone) -> egui::Color32 {
    use crate::codec::Tone;
    match tone {
        Tone::Plain => ui.visuals().text_color(),
        Tone::Weak => ui.visuals().weak_text_color(),
        Tone::Good => egui::Color32::from_rgb(110, 222, 148),
        Tone::Bad => egui::Color32::from_rgb(242, 108, 120),
        // A live row and an in-flight one share the spectral blue; the
        // desktop pulses the second, which a phone's repaint budget does not
        // buy back — the word "running" in the label already says it.
        Tone::Live | Tone::InFlight => egui::Color32::from_rgb(118, 188, 242),
    }
}

/// The conversation's display name — **who** the model turns are, since a
/// speaker is an agent and not a model id. The roster row already carries it;
/// falling back to the addressed id is what a transcript opened before its
/// list arrived shows.
pub(crate) fn speaker_of(snap: &Snapshot, agent: &str) -> String {
    snap.conversations
        .iter()
        .find(|row| row.root_id == agent)
        .map_or_else(|| agent.to_owned(), |row| row.display.clone())
}
