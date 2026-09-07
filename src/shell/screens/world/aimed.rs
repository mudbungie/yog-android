//! **The aimed bar** (DESIGN §13.14, bl-6e8a): the one way in to the seven
//! surfaces that name THIS workspace, as a single row of equal chips over the
//! conversation list.
//!
//! It was seven buttons of seven different widths, wrapped over four bands,
//! taking a third of a phone screen off the list they sit above. The operator
//! called it incoherent and was right: the bands were the growth of a ledger,
//! not a shape anybody chose, and the widths said the labels were the point.

use eframe::egui;

use super::{World, admin, candidates, clients, fleet, signin, work};
use crate::shell::app::Shell;

/// The hairline between two chips, and the breathing room inside one. Both
/// are tighter than egui's defaults for the reason `aimed_entries` states.
const GAP: f32 = 4.0;
const PAD: f32 = 2.0;

/// The seven, in the order a thumb meets them: what the workspace is DOING
/// first — its obligations, what its attempts cost, what they changed — then
/// the machinery that runs them, then the wiring underneath.
///
/// Each row is the op token the wire and the parity gate know it by, and the
/// short noun a person reads. The two are separate on purpose: the token is
/// the one name (`act:<op>`, the harness's rectangle, the screen it opens) and
/// the label is prose, so `workspace-balls` reads as *balls* on a surface
/// where every entry is already this workspace's and the prefix says nothing.
const AIMED: [(&str, &str, World); 7] = [
    (
        crate::codec::View::Here.screen(),
        "balls",
        World::Balls(crate::codec::View::Here),
    ),
    (candidates::SCREEN, "science", World::Candidates),
    (work::SCREEN, "diff", World::Work),
    (fleet::SCREEN, "fleet", World::Fleet),
    (clients::SCREEN, "clients", World::Clients),
    (admin::SCREEN, "config", World::Admin),
    (signin::SCREEN, "login", World::SignIn),
];

impl Shell {
    /// **One bar, seven equal chips, and no scroller** (§13.14).
    ///
    /// The width is divided rather than measured, and that is the whole
    /// argument. A control off the glass records no `act:` tag (`shell::act`
    /// returns on `is_rect_visible`), so a horizontal scroller and an overflow
    /// menu both promise a parity inventory neither can keep — *unproven is
    /// red* (PARITY §5), and a chip five points past the edge would redden the
    /// gate on a screen the walk did visit. Equal division is the one shape in
    /// which the COUNT of controls is a fact the layout cannot lose: all seven
    /// are laid out, all seven are on the glass, and a label too long for its
    /// share elides instead of pushing its neighbours off.
    ///
    /// It costs one band where four stood, which is the list's to keep.
    pub(in crate::shell) fn aimed_entries(&mut self, ui: &mut egui::Ui) {
        ui.scope(|ui| {
            // A strip of chips reads as ONE control when the gaps between
            // them are hairlines and every chip is the same width; at egui's
            // default spacing the same seven read as seven buttons that
            // happen to be adjacent. The tighter numbers are also what buys
            // the widest label its room — at the default padding `science`
            // was the one word that had to elide, on a 1080-pixel phone.
            ui.spacing_mut().item_spacing.x = GAP;
            ui.spacing_mut().button_padding.x = PAD;
            let each = AIMED.len() as f32;
            let wide = ((ui.available_width() - GAP * (each - 1.0)) / each).max(0.0);
            let band = egui::vec2(ui.available_width(), crate::shell::mark::TOUCH);
            ui.allocate_ui_with_layout(
                band,
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    for (op, label, world) in AIMED {
                        self.chip(ui, op, label, world, wide);
                    }
                },
            );
        });
    }

    /// One chip, sized before it is filled: the label is laid out inside a
    /// child of exactly its share, so `TextWrapMode::Truncate` elides against
    /// that share rather than against whatever is left of the row.
    fn chip(
        &mut self,
        ui: &mut egui::Ui,
        op: &'static str,
        label: &'static str,
        world: World,
        wide: f32,
    ) {
        let control = crate::shell::theme::chip(ui, label.into(), wide, true);
        crate::shell::act::act(ui, &control, op);
        self.note_control(op, ui, control.rect);
        if control.clicked() {
            self.open_world(world);
        }
    }
}
