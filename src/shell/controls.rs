//! **The controls row** (DESIGN §13.2, bl-0267): one row under the composer
//! carrying the acts that are about the CONVERSATION rather than about the
//! message being typed — which model answers it, and (as later balls land)
//! what to do with a turn already running.
//!
//! **Under the composer, inside the same floor.** It is the last thing added
//! to the bottom-up layout before the composer, so it sits between the
//! composer and the platform's floor (bl-9cfd) and rides the keyboard with
//! it. Its own height is the §13.2 touch floor, spent both as the row's
//! height and as the minimum interact size inside it — a control a thumb
//! misses is a defect, not a style.
//!
//! **Tap is the act; there is no apply.** Picking a model IS the assignment
//! (§13.2), so nothing here holds a draft the operator could leave unsent,
//! and an engine that refuses one says so in the banner the model already
//! publishes. What the selectors show is what this device SET — never a
//! guess at what is set: no shape on the wire states a workspace's current
//! assignment, and DESIGN §8's rule is that a client which re-derived world
//! state would be inventing it.

use eframe::egui;

use super::app::Shell;
use crate::seat::Snapshot;

impl Shell {
    /// The row. Painted only where a workspace is focused, because every
    /// control in it acts on one.
    pub(super) fn controls(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        let Some(workspace) = snap.focus.workspace.clone() else {
            return;
        };
        // The selection belongs to the workspace it was made in: another
        // workspace's pick is not a fact about this one, so it goes when the
        // focus does.
        if self.picked_in.as_deref() != Some(workspace.as_str()) {
            self.picked_in = Some(workspace);
            self.provider = None;
            self.model = None;
        }
        ui.scope(|ui| {
            ui.spacing_mut().interact_size.y = super::mark::TOUCH;
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), super::mark::TOUCH),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    let wide = (ui.available_width() - ui.spacing().item_spacing.x) / 2.0;
                    self.providers(ui, snap, wide);
                    self.models(ui, snap, wide);
                },
            );
        });
    }

    /// The provider selector. Its list is the engine's own rows, and a row
    /// that is blocked is greyed **by the fact it states about itself** —
    /// still tappable, because the operator may be about to sign it in and a
    /// control that vanishes teaches nothing.
    fn providers(&mut self, ui: &mut egui::Ui, snap: &Snapshot, wide: f32) {
        let shown = self
            .provider
            .clone()
            .unwrap_or_else(|| "provider".to_owned());
        let opened = egui::ComboBox::from_id_salt("provider")
            .selected_text(shown)
            .width(wide)
            .show_ui(ui, |ui| {
                for row in &snap.providers {
                    let label = format!("{} · {}", row.name, row.fact);
                    let label = if row.blocked.is_some() {
                        egui::RichText::new(label).color(ui.visuals().weak_text_color())
                    } else {
                        egui::RichText::new(label)
                    };
                    if ui.selectable_label(false, label).clicked() {
                        self.provider = Some(row.name.clone());
                        self.model = None;
                    }
                }
            });
        // The read is asked for by the tap that opened the list, not by the
        // frame: a read per frame while a popup is open would be a gesture
        // per frame. What was already known paints meanwhile (§14).
        if opened.response.clicked()
            && let Some(model) = self.model()
        {
            model.list_providers();
        }
    }

    /// The model selector: this provider's models, and the tap that assigns
    /// one. Disabled until a provider is chosen — a model without its
    /// provider is not an assignment the wire can state.
    fn models(&mut self, ui: &mut egui::Ui, snap: &Snapshot, wide: f32) {
        let Some(provider) = self.provider.clone() else {
            ui.add_enabled(
                false,
                egui::Button::new("model").min_size(egui::vec2(wide, 0.0)),
            );
            return;
        };
        let shown = self.model.clone().unwrap_or_else(|| "model".to_owned());
        let mut picked = None;
        let opened = egui::ComboBox::from_id_salt("model")
            .selected_text(shown)
            .width(wide)
            .show_ui(ui, |ui| {
                for name in snap.models.get(&provider).into_iter().flatten() {
                    if ui.selectable_label(false, name).clicked() {
                        picked = Some(name.clone());
                    }
                }
            });
        if let Some(name) = picked {
            self.model = Some(name.clone());
            if let Some(model) = self.model() {
                model.pick_model(provider.clone(), name);
            }
            return;
        }
        if opened.response.clicked()
            && let Some(model) = self.model()
        {
            model.list_models(provider);
        }
    }
}
