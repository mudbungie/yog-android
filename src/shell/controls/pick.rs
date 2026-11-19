//! **The three selectors** (bl-0267; DESIGN §25, bl-5e0b): which machine
//! serves the worker's provider, which of that machine's providers, and
//! which of its models. Split from the row that carries them — the row is
//! layout and gating, these are a list each and the tap that picks from it.
//!
//! **Each level is its own list, and a pick opens the next** (§25, closing
//! bl-c64c): a machine pick opens that machine's providers, a provider pick
//! opens its models, and the model pick is the act — the one write, naming
//! the provider whole. The first two write nothing: they are a path.
//!
//! **Each selector's own node carries its tags** (PARITY §4). Opening a list
//! is the read that fills it (`providers`, `clients`, `models`) and choosing
//! a model is the act (`model`), and they are one control to a thumb. The
//! tags ride the control rather than the items because the items exist only
//! while the popup is open: a node that appears for one gesture is not a
//! *discoverable* interactable, and §5's claim is presence of the
//! affordance, not of every transient child. The `model` tag is on the model
//! selector, which is the affordance an operator reaches the assignment
//! through.
//!
//! All three are `controls/drop.rs`'s drop-down rather than egui's
//! `ComboBox` (bl-78c2), for one reason and only one: a combo's list is
//! placed against the display, and the display includes the gesture-nav zone.

use eframe::egui;

use super::super::act;
use super::super::app::Shell;
use super::Seat;
use crate::codec::pick::machine::{self, Aim};
use crate::seat::Snapshot;
use crate::status::Slot;

impl Shell {
    /// **The machine selector** (§25): the server itself, then each machine
    /// the listing names. A pick writes nothing — it says whose providers
    /// the next list shows, and opens that list.
    pub(super) fn machines(
        &mut self,
        ui: &mut egui::Ui,
        snap: &Snapshot,
        seat: Seat,
    ) -> egui::Response {
        let open = seat.open;
        let (clients, aim) = aimed(self, snap);
        let mut picked = None;
        let opened = super::drop::drop_down(
            ui,
            (
                "machine",
                "opens the list of machines the worker's provider may be served by",
            ),
            seat,
            |ui| {
                for machine in machine::machines(&snap.providers, &clients) {
                    let label = machine.label();
                    if crate::shell::theme::pick(ui, label, "lists this machine's providers")
                        .clicked()
                    {
                        picked = Some(machine);
                    }
                }
            },
        );
        if let Some(machine) = picked {
            // The machine already in force keeps its provider; another one
            // is browsed, and has none until one is picked.
            if aim.machine.as_ref() != Some(&machine) {
                self.machine = Some(machine);
                self.provider = None;
                self.model = None;
            }
            self.open_next(ui, Slot::Provider);
        }
        act::acts(ui, &opened, &["providers", "clients"]);
        // The reads are asked for by the tap that opened the list, not by
        // the frame: a read per frame while a popup is open would be a
        // gesture per frame. What was already known paints meanwhile (§14).
        if (open || opened.clicked())
            && let Some(model) = self.model()
        {
            model.list_providers();
            model.list_clients();
        }
        opened
    }

    /// The provider selector: the chosen machine's rows, each shown without
    /// its qualification. A row that is blocked is greyed **by the fact it
    /// states about itself** — still tappable, because the operator may be
    /// about to sign it in and a control that vanishes teaches nothing.
    pub(super) fn providers(
        &mut self,
        ui: &mut egui::Ui,
        snap: &Snapshot,
        seat: Seat,
    ) -> egui::Response {
        let open = seat.open;
        let (clients, aim) = aimed(self, snap);
        let Some(on) = aim.machine.clone() else {
            let control = dark(ui, &seat, "the provider: dark until a machine is chosen");
            act::act(ui, &control, "providers");
            return control;
        };
        let mut picked = None;
        let opened = super::drop::drop_down(
            ui,
            (
                "provider",
                "opens the list of providers the chosen machine serves",
            ),
            seat,
            |ui| {
                for (shown, row) in machine::offered(&snap.providers, &clients, &on) {
                    let label = format!("{shown} · {}", row.fact);
                    let label = if row.blocked.is_some() {
                        egui::RichText::new(label).color(ui.visuals().weak_text_color())
                    } else {
                        egui::RichText::new(label)
                    };
                    if crate::shell::theme::pick(ui, label, "lists this provider's models")
                        .clicked()
                    {
                        picked = Some(row.name);
                    }
                }
            },
        );
        if let Some(name) = picked {
            if aim.provider.as_deref() != Some(name.as_str()) {
                self.provider = Some(name);
                self.model = None;
            }
            // The machine is the provider's from here (§25).
            self.machine = None;
            self.open_next(ui, Slot::Model);
        }
        act::act(ui, &opened, "providers");
        if (open || opened.clicked())
            && let Some(model) = self.model()
        {
            model.list_providers();
        }
        opened
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
        seat: Seat,
    ) -> egui::Response {
        let open = seat.open;
        let Some(provider) = aimed(self, snap).1.provider else {
            let control = dark(ui, &seat, "the model: dark until the worker has a provider");
            act::acts(ui, &control, &["models", "model"]);
            return control;
        };
        // **What this conversation is actually ON** (bl-809d) is the face the
        // row handed in: `codec::pick::face` made it, under the coverage
        // floor, and it shows the workspace's model only under the
        // workspace's own provider, because a model belongs to the provider
        // that serves it.
        let mut picked = None;
        let opened = super::drop::drop_down(
            ui,
            ("model", "opens the list of models the provider offers"),
            seat,
            |ui| {
                for name in snap.models.get(&provider).into_iter().flatten() {
                    if crate::shell::theme::pick(ui, name, "sets the worker's model").clicked() {
                        picked = Some(name.clone());
                    }
                }
            },
        );
        act::acts(ui, &opened, &["models", "model"]);
        if let Some(name) = picked {
            self.model = Some(name.clone());
            if let Some(model) = self.model() {
                model.pick_model(provider.clone(), name);
            }
            return opened;
        }
        if (open || opened.clicked())
            && let Some(model) = self.model()
        {
            model.list_models(provider);
        }
        opened
    }

    /// **A pick opens the next list** (§25): asked for on the next frame,
    /// because this frame's tap is still closing the list it landed in.
    fn open_next(&mut self, ui: &egui::Ui, slot: Slot) {
        self.rule.next = Some(slot);
        ui.ctx().request_repaint();
    }
}

/// A selector with nothing above it to list under: dark, and opening
/// nothing, so no mark (§23 item 14).
fn dark(ui: &mut egui::Ui, seat: &Seat, help: &str) -> egui::Response {
    crate::shell::theme::door(
        ui,
        (
            &seat.face,
            crate::shell::theme::rgb(crate::theme::INK_FAINT),
        ),
        None,
        egui::vec2(seat.wide, crate::theme::TOUCH),
        help,
    )
}

/// **What these controls are pointed at**, and the roster the names are
/// read against: the machine being browsed, else the optimistic provider,
/// else what the workspace is actually set to (bl-e9f9; §25).
pub(super) fn aimed(shell: &Shell, snap: &Snapshot) -> (Vec<String>, Aim) {
    let clients = machine::roster(snap.clients.as_ref(), snap.focus.workspace.as_deref());
    let set = crate::codec::pick::worker(&snap.roles);
    let aim = machine::aim(
        shell.machine.as_ref(),
        shell.provider.as_deref(),
        set.as_ref(),
        &clients,
    );
    (clients, aim)
}
