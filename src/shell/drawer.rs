//! **The drawer** (DESIGN §24 item 2; STYLE §5, *the drawer*): a `SURFACE`
//! panel sliding in from the left edge over the chat, `crate::drawer::width`
//! wide, a scrim over what is left — laid on egui's foreground layer so
//! nothing under it is pressed through it, its edge easing in and out over
//! `drawer::EASE` on the input clock. What it holds, in what order, what a
//! tap yields and the gestures that move it are `crate::drawer`, pure and
//! host-tested; this file and its children paint them and perform the taps.
//!
//! **Four ways in, one way out.** It opens from the chat's `≡`
//! (`shell/tabs.rs`), from the left-edge drag (`screens.rs` reads it), from
//! `/conversations` and from the rule's name. It closes on a tap on the
//! scrim, on a drag from inside it that travels left past `drawer::REACH`,
//! on the platform's back, and on every navigation it performs — and it
//! **cannot close while nothing is focused** (`drawer::stands`): there is no
//! chat under it then, so back falls through to leaving the app, the
//! platform's own meaning (§24 item 2).
//!
//! **One scroll, and a fixed foot**: the head (`head.rs`), the find field,
//! the conversation tree and the places (`list.rs`), then the foot under the
//! scroll (`foot.rs`) — `settings`, and the seat panel's lines with the mark
//! at their left.

use eframe::egui;

use super::app::Shell;
use super::place::Band;
use super::theme::rgb;
use crate::seat::Snapshot;
use crate::theme::{self, space};

mod foot;
mod head;
mod list;

pub(in crate::shell) use head::square;

/// **The drawer between frames**: opened by a gesture or a verb and closed by
/// every navigation it performs; the head's list, toggled; the watermark the
/// head's reload was asked at; and two frame-scoped facts the pass sets
/// before the chat paints — whether it stands this pass (the composer's
/// field steps off the glass while it does) and whether the platform's back
/// asked it to close.
#[derive(Default)]
pub(crate) struct Drawer {
    pub(crate) open: bool,
    pub(crate) list: bool,
    asked: Option<usize>,
    pub(crate) standing: bool,
    pub(crate) closing: bool,
}

/// The edge's eased position, in egui's animation memory.
fn edge() -> egui::Id {
    egui::Id::new("yog.drawer.edge")
}

/// **Settle the edge shut without painting** — under an overlay, so the
/// drawer does not travel out from under it the frame the overlay closes.
pub(super) fn settle(ctx: &egui::Context) {
    ctx.animate_value_with_time(edge(), 0.0, 0.0);
}

impl Shell {
    /// **The drawer over the chat**, painted after it. Nothing while it is
    /// shut and its edge has come home.
    pub(super) fn drawer(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        let ctx = ui.ctx().clone();
        let stands = self.drawer.standing;
        let screen = ctx.content_rect();
        let wide = crate::drawer::width(screen.width());
        let x = ctx.animate_value_with_time(
            edge(),
            if stands { wide } else { 0.0 },
            crate::drawer::EASE,
        );
        if x <= 0.0 {
            return;
        }
        if !stands {
            self.painted(&ctx, screen, x, wide, snap);
            return;
        }
        // The drawer is the screen while it stands (§15.2): the walk's
        // `seated`, `switching` and `row-menu` beats name it.
        self.note_screen("conversations");
        let mut closing = std::mem::take(&mut self.drawer.closing);
        let (origin, now) = ctx.input(|i| {
            let down = i.pointer.any_down();
            (
                i.pointer.press_origin().filter(|_| down).map(|p| p.x),
                i.pointer.latest_pos().map(|p| p.x),
            )
        });
        closing |= crate::drawer::closes(origin, now, x);
        closing |= self.painted(&ctx, screen, x, wide, snap);
        if closing {
            self.drawer.open = false;
            self.drawer.asked = None;
        }
    }

    /// The scrim and the panel at the edge's position `x`. Answers whether
    /// anything inside asked to close.
    fn painted(
        &mut self,
        ctx: &egui::Context,
        screen: egui::Rect,
        x: f32,
        wide: f32,
        snap: &Snapshot,
    ) -> bool {
        let alpha = (f32::from(crate::drawer::SCRIM) * x / wide.max(1.0)) as u8;
        let ppp = ctx.pixels_per_point();
        let floor = super::place::floor(self.inset.bottom as f32 / ppp);
        let top = self.inset.top as f32 / ppp;
        let mut closing = false;
        egui::Area::new(edge().with("panel"))
            .order(egui::Order::Foreground)
            .fixed_pos(screen.left_top())
            .movable(false)
            .show(ctx, |ui| {
                ui.set_clip_rect(screen);
                let panel =
                    egui::Rect::from_min_size(screen.left_top(), egui::vec2(x, screen.height()));
                let rest = egui::Rect::from_min_max(egui::pos2(x, screen.top()), screen.max);
                // **The scrim takes no tap but the one that closes the
                // drawer**: a click-sensed rect over the rest of the glass.
                let scrim = ui.allocate_rect(rest, egui::Sense::click());
                super::theme::told::told(ui, &scrim, "closes the drawer");
                ui.painter()
                    .rect_filled(rest, 0, super::theme::scrim(alpha));
                self.note_control("scrim", ui, rest);
                closing |= scrim.clicked();
                ui.painter().rect_filled(panel, 0, rgb(theme::SURFACE));
                // **The panel's own ground takes a tap too**, so nothing
                // between its rows is pressed through to the chat: egui hands
                // a press to the topmost widget under it in a layer, and the
                // rows painted after this stand over it.
                ui.interact(panel, edge().with("ground"), egui::Sense::click());
                ui.painter().vline(
                    panel.right(),
                    panel.y_range(),
                    egui::Stroke::new(1.0, rgb(theme::HAIRLINE)),
                );
                let inner = egui::Rect::from_min_max(
                    egui::pos2(panel.left() + space::M, panel.top() + top),
                    egui::pos2(panel.right() - space::M, panel.bottom() - floor),
                );
                let builder = egui::UiBuilder::new()
                    .max_rect(inner)
                    .layout(egui::Layout::bottom_up(egui::Align::Min));
                ui.scope_builder(builder, |ui| {
                    closing |= self.drawer_body(ui, snap);
                });
            });
        closing
    }

    /// **What it holds**, bottom-up: the foot fixed at the floor, then — in
    /// what is left — one scroll of the head, the find field, the
    /// conversations and the places. Answers whether a navigation was
    /// performed.
    fn drawer_body(&mut self, ui: &mut egui::Ui, snap: &Snapshot) -> bool {
        let mut closing = self.drawer_foot(ui);
        ui.add_space(space::S);
        hairline(ui);
        // The row menu is a popup and obeys `shell::place` against the band
        // the list is painted in (bl-78c2), read before the scroller narrows
        // it.
        let area = Band {
            top: ui.max_rect().top(),
            bottom: ui.available_rect_before_wrap().bottom(),
        };
        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            egui::ScrollArea::vertical()
                .id_salt("the drawer")
                .min_scrolled_height(0.0)
                .auto_shrink(false)
                .show(ui, |ui| {
                    closing |= self.drawer_head(ui, snap);
                    closing |= self.drawer_find(ui);
                    closing |= self.drawer_rows(ui, snap, area);
                    hairline(ui);
                    closing |= self.drawer_places(ui, snap);
                });
        });
        closing
    }
}

/// A `HAIRLINE` across the drawer, `XS` of air either side.
fn hairline(ui: &mut egui::Ui) {
    ui.add_space(space::XS);
    let (rect, _) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0, rgb(theme::HAIRLINE));
    ui.add_space(space::XS);
}
