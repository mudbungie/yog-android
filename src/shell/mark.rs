//! **The yog mark**: the application mark itself, a control at the top-left
//! of every screen, and the one way into the configuration surface (bl-387f;
//! the drawn mark replaced the interim wordmark under bl-ff27).
//!
//! The operator's sighting behind the control: a device enrolled as a seat
//! had no path back to the first-run surface. Breadcrumbs were considered
//! and rejected — a trail requires every path worked out, and the paths are
//! not. One standing control that is always there asks nothing of the
//! screens beneath it.
//!
//! The mark TOGGLES. Into the configuration when the app is showing a
//! component, back out when the configuration is open — and the chooser also
//! states its own `< back` (bl-e192), because a toggle nobody can see is not
//! an affordance.
//!
//! **The picture is `crate::icon`'s walk, said in egui's words.** The walk
//! is pure and host-tested; this file only chooses egui's primitive for each
//! shape — a trace goes out as its centreline and width so egui's own
//! stroker does the joins, the lune as one convex polygon, a disc as a
//! circle. Nothing here re-derives an edge or re-decides a hue.

use eframe::egui;

use super::app::Shell;
use crate::icon::Shape;

/// The §13.2 touch floor, in points: every navigation row and action
/// control stands at least this tall. The mark's own tap target too.
pub(crate) const TOUCH: f32 = 44.0;

impl Shell {
    /// The standing bar (§13.2): the mark, the screen's back control when it
    /// has a parent, and its title — one row, painted before every screen's
    /// body, and the ONLY place a heading or a back gesture is spelled.
    /// Returns whether back was asked for; the caller owns what one depth up
    /// means, because the bar has no idea and must not grow one.
    ///
    /// **The platform's own back control comes through here too** (bl-550e):
    /// where this bar paints a back control it is the thing with a depth to
    /// walk, so it takes a pending press — and every caller's existing arm
    /// runs unchanged, which is the whole reason the wiring is one line here
    /// rather than a second walk somewhere else. A bar with no back control
    /// takes nothing: there is no depth, and the press goes on to mean
    /// leaving the app.
    pub(super) fn bar(&mut self, ui: &mut egui::Ui, title: &str, backable: bool) -> bool {
        let mut back = false;
        ui.horizontal(|ui| {
            self.mark(ui);
            if backable {
                let control = egui::Button::new("<").min_size(egui::vec2(TOUCH, TOUCH));
                back = ui.add(control).clicked() || std::mem::take(&mut self.back);
            }
            ui.heading(title);
        });
        back
    }

    /// The mark control. Painted before anything else on every screen, whatever
    /// the component — that unconditionality is the whole feature.
    pub(super) fn mark(&mut self, ui: &mut egui::Ui) {
        let (rect, hit) = ui.allocate_exact_size(egui::vec2(TOUCH, TOUCH), egui::Sense::click());
        paint(ui.painter(), rect.shrink(4.0));
        if !hit.clicked() {
            return;
        }
        if self.settings {
            // The way out drops whatever the configuration held: a pasted
            // envelope is a private key and an open camera is a running
            // capture session, and this tap is the last thing that knows the
            // screen is going away.
            self.forget_envelope();
            self.settings = false;
        } else {
            self.settings = true;
        }
    }
}

/// The walk's straight-alpha RGBA as an egui colour.
fn hue(fill: [u8; 4]) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(fill[0], fill[1], fill[2], fill[3])
}

/// Paint the mark into `rect`. The unit square the walk works in maps to the
/// largest centred square `rect` holds — the mark is round, and a stretched
/// one is a different picture.
fn paint(painter: &egui::Painter, rect: egui::Rect) {
    let side = rect.width().min(rect.height());
    let origin = rect.center() - egui::vec2(side, side) / 2.0;
    let at = |(x, y): (f32, f32)| origin + egui::vec2(x * side, y * side);
    for shape in crate::icon::mark() {
        painter.add(match &shape {
            Shape::Trace { path, width, fill } => egui::Shape::line(
                path.iter().map(|point| at(*point)).collect(),
                egui::Stroke::new(width * side, hue(*fill)),
            ),
            Shape::Lune { ribs, fill } => {
                // Out along one arc, home along the other — the ribs' own
                // two edges, which meet at both ends and so close the lens
                // with no seam to hide.
                let mut points: Vec<egui::Pos2> = ribs.iter().map(|rib| at(rib.out)).collect();
                points.extend(ribs.iter().rev().map(|rib| at(rib.back)));
                egui::Shape::convex_polygon(points, hue(*fill), egui::Stroke::NONE)
            }
            Shape::Disc {
                cx,
                cy,
                radius,
                fill,
            } => egui::Shape::circle_filled(at((*cx, *cy)), radius * side, hue(*fill)),
        });
    }
}
