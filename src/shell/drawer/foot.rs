//! **The drawer's foot** (DESIGN §24 item 2), fixed under the scroll:
//! `settings` as a row (its overlay, §23 item 11), and the seat panel's lines
//! in `INK_WEAK` — the identity, the `tools:` line, the update offer in the
//! brand (§20.4; its once-per-process check starts on the drawer's first
//! paint) — with **the mark** at the row's left, its picture at the line's
//! height in a `TOUCH`-square seat: still the configuration door and still
//! the walk's `mark` rectangle. The seat panel moved here whole from the
//! switcher's head; nothing about what it says changes.

use eframe::egui;

use super::super::app::Shell;
use super::super::theme::rgb;
use crate::overlay::Overlay;
use crate::theme;

/// What the settings row does, told on a long press (§23 item 10).
const SETTINGS_HELP: &str = "opens this device's own knobs";

impl Shell {
    /// **The foot**, added to a bottom-up block: the seat panel first, so it
    /// takes the floor, then `settings` over it. Answers whether a navigation
    /// was performed — the settings overlay, or the mark's configuration
    /// surface.
    pub(super) fn drawer_foot(&mut self, ui: &mut egui::Ui) -> bool {
        let mut closing = false;
        let was = self.settings;
        ui.horizontal(|ui| {
            let line = ui.text_style_height(&egui::TextStyle::Body);
            self.mark_drawn(ui, line);
            ui.vertical(|ui| {
                let who = egui::RichText::new(self.identity()).color(rgb(theme::INK_WEAK));
                ui.add(egui::Label::new(who).truncate());
                Self::hosting(ui);
                self.update_entry(ui);
            });
        });
        // The mark toggled the configuration surface: a navigation, so the
        // drawer is shut behind it and the chat is what the way out lands on.
        closing |= self.settings != was;
        let row = super::super::theme::row(
            ui,
            egui::RichText::new(crate::drawer::SETTINGS)
                .color(rgb(theme::INK))
                .into(),
            SETTINGS_HELP,
        );
        self.note_control("drawer-settings", ui, row.rect);
        if row.clicked() {
            self.open(Overlay::Settings);
            closing = true;
        }
        closing
    }
}
