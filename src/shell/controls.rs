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
use crate::codec::RoleRow;
use crate::seat::Snapshot;

/// The effort selector's width class. Narrower than a provider or a model
/// selector because its whole vocabulary is four short words.
const EFFORT: f32 = 76.0;

/// One row of controls, given its own height for the reason every row in this
/// app is (bl-193c): a `left_to_right(Center)` layout handed the rest of the
/// screen centres its widgets in it.
fn band(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), super::mark::TOUCH),
        egui::Layout::left_to_right(egui::Align::Center),
        add,
    );
}

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
            self.effort = None;
            self.priority = None;
        }
        // **Truth overtakes the guess** (bl-e9f9): every act on this row is
        // optimistic — the control shows the pick the moment it is tapped,
        // because the round trip is seconds — and the assignments read is
        // what settles it. When the seat's read count moves, whatever was
        // standing optimistically goes and what the workspace ACTUALLY has
        // is what paints. That covers a refusal for free: the engine never
        // took it, so the read never carries it, so the control snaps back
        // and the banner says why.
        if self.tuned_at != snap.roles_read {
            self.tuned_at = snap.roles_read;
            self.provider = None;
            self.model = None;
            self.effort = None;
            self.priority = None;
        }
        let set = crate::codec::pick::worker(&snap.roles);
        // **What the picked provider will take** (bl-dfbb), read off its own
        // row in covered code: the paint asks, it never derives.
        let (effort, priority) =
            crate::codec::pick::tunable(&snap.providers, provider(self, set.as_ref()).as_deref());
        ui.scope(|ui| {
            ui.spacing_mut().interact_size.y = super::mark::TOUCH;
            band(ui, |ui| {
                // The stop controls first: they are the acts an operator
                // reaches for while something is running, and they are
                // only there while it is (bl-48fa).
                self.stops(ui, snap);
                let wide = (ui.available_width() - ui.spacing().item_spacing.x) / 2.0;
                self.providers(ui, snap, set.as_ref(), wide);
                self.models(ui, snap, set.as_ref(), wide);
            });
            // **A second band, and only when there is something in it.** The
            // tuning controls cannot share the first: a selector narrow
            // enough to fit beside two others at a 320-point width is one an
            // operator cannot read a model name in (measured). egui's own
            // wrapping layout does not answer this — a `ComboBox` does not
            // declare its width to the wrap check, so it overflows the edge
            // instead of moving down (measured: 418 points in a 390-point
            // column) — so the second row is allocated rather than wrapped
            // into. It is the same controls block under the composer, not a
            // new place to look (§13.2).
            if effort || priority {
                band(ui, |ui| {
                    if effort {
                        self.effort(ui, set.as_ref());
                    }
                    if priority {
                        self.priority(ui, set.as_ref());
                    }
                });
            }
        });
    }

    /// **The effort selector** (REMOTE §9.4): how much reasoning the worker's
    /// model calls request. The vocabulary is closed and no wire read backs
    /// it, so the options are the codec's own constant; `off` is one of them
    /// and rides as the real null the engine reads.
    fn effort(&mut self, ui: &mut egui::Ui, set: Option<&RoleRow>) {
        // The read carries the FILE's own word, which may be one the gesture
        // vocabulary does not spell (bl-e9f9). It is shown as itself — an
        // operator seeing `extreme` is being told the truth, and the four
        // words below are what they may change it TO.
        let shown = self
            .effort
            .clone()
            .or_else(|| set.and_then(|row| row.effort.clone()))
            .unwrap_or_else(|| "effort".to_owned());
        let mut picked = None;
        egui::ComboBox::from_id_salt("effort")
            .selected_text(shown)
            .width(EFFORT)
            .show_ui(ui, |ui| {
                for level in crate::codec::pick::LEVELS {
                    let label = crate::codec::Effort::label(level);
                    if ui.selectable_label(false, &label).clicked() {
                        picked = Some((level, label));
                    }
                }
            });
        if let Some((level, label)) = picked {
            self.effort = Some(label);
            if let Some(model) = self.model() {
                model.set_effort(level);
            }
        }
    }

    /// **The priority toggle**: ask the provider's priority lane for this
    /// role's calls, or stop asking. A toggle and not a tri-state — `off`
    /// removes the line, and asking for the *standard* lane is a different
    /// intent no config key expresses (REMOTE §9.4).
    fn priority(&mut self, ui: &mut egui::Ui, set: Option<&RoleRow>) {
        let mut on = self
            .priority
            .unwrap_or_else(|| set.is_some_and(|row| row.priority));
        if ui.toggle_value(&mut on, "priority").clicked() {
            self.priority = Some(on);
            if let Some(model) = self.model() {
                model.set_priority(on);
            }
        }
    }

    /// **The stop controls** (REMOTE §3.1, bl-48fa): shown by the gates the
    /// engine puts ON the row, never by a reading taken here — §9.4's rule is
    /// that a gate which is not derivable from a row goes on the row, and
    /// both of these are. They are independent: `stoppable` is true iff this
    /// conversation holds the executor lock, `stop_children` iff some other
    /// agent's id extends this one — so a quiet root with a working child
    /// offers *stop all* and no *stop*, which is exactly right and is why
    /// two gates cross rather than one.
    ///
    /// **The gesture is the op.** A deposited `/stop` line is content, and
    /// content wakes the very driver it meant to kill; the seat model sends
    /// the wire's own act.
    fn stops(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        let Some(row) = focused_row(snap) else {
            return;
        };
        let (stoppable, children) = (row.stoppable, row.stop_children);
        if stoppable && ui.button("stop").clicked() {
            self.stop_turn(false);
        }
        if children && ui.button("stop all").clicked() {
            self.stop_turn(true);
        }
        // **Nudge is the other half of the same question** (bl-d09e): stop is
        // for a turn that is running, nudge for a branch that stopped
        // advancing — so it is offered exactly when nothing is in flight,
        // read off the row's own `flight` (its `None` IS "at rest"). The
        // engine's own `nudgeable` gate rides the agent view this codec does
        // not spell; if the row's reading proves too coarse, the fix is that
        // gate on the row rather than a second derivation here.
        if row.flight.is_none()
            && ui.button("nudge").clicked()
            && let Some(model) = self.model()
        {
            model.nudge();
        }
    }

    /// Ask the worker to stop the focused turn.
    fn stop_turn(&self, children: bool) {
        if let Some(model) = self.model() {
            model.stop_turn(children);
        }
    }

    /// The provider selector. Its list is the engine's own rows, and a row
    /// that is blocked is greyed **by the fact it states about itself** —
    /// still tappable, because the operator may be about to sign it in and a
    /// control that vanishes teaches nothing.
    fn providers(&mut self, ui: &mut egui::Ui, snap: &Snapshot, set: Option<&RoleRow>, wide: f32) {
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
    fn models(&mut self, ui: &mut egui::Ui, snap: &Snapshot, set: Option<&RoleRow>, wide: f32) {
        let Some(provider) = provider(self, set) else {
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

/// The focused conversation's row, which is where every conversation-level
/// gate rides (REMOTE §9.4). A conversation the list has not caught up with
/// yet has no row and therefore no gates — the honest reading, and the same
/// one the roster's own display name falls back through.
fn focused_row(snap: &Snapshot) -> Option<crate::codec::ConvRow> {
    let agent = snap.focus.agent.as_deref()?;
    snap.conversations
        .iter()
        .find(|row| row.root_id == agent)
        .cloned()
}

/// **The provider these controls are pointed at**: the optimistic pick if one
/// is standing, else what the workspace is actually set to (bl-e9f9).
fn provider(shell: &Shell, set: Option<&RoleRow>) -> Option<String> {
    shell
        .provider
        .clone()
        .or_else(|| set.map(|row| row.provider.clone()))
}
