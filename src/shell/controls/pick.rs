//! **The two selectors** (bl-0267): which provider answers this workspace,
//! and which of its models. Split from the row that carries them — the row is
//! layout and gating, these two are a list and the tap that assigns from it.
//!
//! **Each selector's own node carries two tags** (PARITY §4). Opening the list
//! is the read that fills it (`providers`, `models`) and choosing a row inside
//! is the act (`model`), and they are one control to a thumb. The tags ride
//! the `ComboBox` rather than the items because the items exist only while the
//! popup is open: a node that appears for one gesture is not a *discoverable*
//! interactable, and §5's claim is presence of the affordance, not of every
//! transient child. The `model` tag is on the model selector, which is the
//! affordance an operator reaches the assignment through.

use eframe::egui;

use super::super::act;
use super::super::app::Shell;
use crate::codec::RoleRow;
use crate::seat::Snapshot;

impl Shell {
    /// The provider selector. Its list is the engine's own rows, and a row
    /// that is blocked is greyed **by the fact it states about itself** —
    /// still tappable, because the operator may be about to sign it in and a
    /// control that vanishes teaches nothing.
    pub(super) fn providers(
        &mut self,
        ui: &mut egui::Ui,
        snap: &Snapshot,
        set: Option<&RoleRow>,
        wide: f32,
    ) {
        let shown = provider(self, set).unwrap_or_else(|| "provider".to_owned());
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
        act::act(ui, &opened.response, "providers");
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
    /// provider is not an assignment the wire can state. It is tagged in
    /// either state, because a disabled control is still the discoverable
    /// affordance for the act (and still a node in the tree).
    pub(super) fn models(
        &mut self,
        ui: &mut egui::Ui,
        snap: &Snapshot,
        set: Option<&RoleRow>,
        wide: f32,
    ) {
        let Some(provider) = provider(self, set) else {
            let control = ui.add_enabled(
                false,
                egui::Button::new("model").min_size(egui::vec2(wide, 0.0)),
            );
            act::acts(ui, &control, &["models", "model"]);
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
        act::acts(ui, &opened.response, &["models", "model"]);
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

/// **The provider these controls are pointed at**: the optimistic pick if one
/// is standing, else what the workspace is actually set to (bl-e9f9).
pub(super) fn provider(shell: &Shell, set: Option<&RoleRow>) -> Option<String> {
    shell
        .provider
        .clone()
        .or_else(|| set.map(|row| row.provider.clone()))
}
