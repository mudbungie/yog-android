//! **The overlay frame** (DESIGN §22 item 6, bl-3f9a): the one box every
//! surface that used to paint its own bar over a depth now floats in. The
//! decisions — which overlay stands, what its name line says, what closing it
//! yields — are `crate::overlay`'s, pure and host-tested; this file paints
//! them and performs the close.
//!
//! **A screen keeps its body and loses its bar.** The box is `SURFACE` with a
//! `HAIRLINE` frame (STYLE §1's one exception: a stroke marks what floats),
//! from `S` under the top inset to the floor; its first line is a
//! `TOUCH`-tall row — the name, what it is open on in weak ink, and `close`
//! at the other end — and the whole line closes it. **The platform's back
//! closes it too**, through `shell::back`'s rule unchanged: the line is the
//! thing with a depth to walk, so it takes a pending press.
//!
//! **What stands down under it.** Nothing else is painted while one stands:
//! the composer is not placed, so its platform field comes off the glass
//! (`shell::field`'s *a field nothing placed is hidden*), and the keyboard
//! goes with it — unless the overlay holds an egui field of its own (the
//! needle, the admin editor), which rides the IME mirror as before.
//!
//! **A control scrolled out of the box is reachable, not lost** (lernie
//! bl-eb29's reading): the box's rect is written to egui's memory while the
//! body lays out, and `shell::act` records a tag for a control laid inside it
//! whether or not the scroll is showing it.

use eframe::egui;

use super::app::Shell;
use super::theme::rgb;
use crate::overlay::{Close, Line, Overlay};
use crate::theme::{self, space};

/// Where the box's rect is held while its body lays out.
fn slot() -> egui::Id {
    egui::Id::new("yog.overlay.box")
}

/// The box standing this pass, if one is — `shell::act`'s question.
pub(super) fn boxed(ui: &egui::Ui) -> Option<egui::Rect> {
    ui.data(|data| data.get_temp::<egui::Rect>(slot()))
}

/// **The box itself**, at `rect`: `SURFACE` with a `HAIRLINE` frame, `body`
/// laid top-down inside it, and the box's rect held in memory while `body`
/// lays out — so a control scrolled out of it still counts as reachable. The
/// overlay and the verb sheet both float in it (§22 items 5 and 6).
pub(super) fn framed(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    salt: &str,
    body: impl FnOnce(&mut egui::Ui),
) {
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(egui::Layout::top_down(egui::Align::Min))
            .id_salt(("framed", salt)),
    );
    let margin = egui::Margin::symmetric(space::S as i8, space::XS as i8);
    egui::Frame::NONE
        .fill(rgb(theme::SURFACE))
        .stroke(egui::Stroke::new(1.0, rgb(theme::HAIRLINE)))
        .corner_radius(theme::RADIUS)
        .inner_margin(margin)
        .show(&mut child, |ui| {
            let inner = rect.size() - margin.sum();
            ui.set_min_size(inner);
            ui.set_max_size(inner);
            ui.data_mut(|data| data.insert_temp(slot(), ui.max_rect()));
            body(ui);
            ui.data_mut(|data| data.remove::<egui::Rect>(slot()));
        });
    ui.advance_cursor_after_rect(rect);
}

impl Shell {
    /// Paint the frame over everything this `ui` was given, the name line at
    /// its head, and `body` in what is left. Closing is performed after the
    /// body has painted, so nothing it laid out addresses a surface that went
    /// away mid-pass.
    pub(super) fn float(
        &mut self,
        ui: &mut egui::Ui,
        line: &Line,
        body: impl FnOnce(&mut Self, &mut egui::Ui),
    ) {
        let room = ui.available_rect_before_wrap();
        let rect = egui::Rect::from_min_max(room.min + egui::vec2(0.0, space::S), room.max);
        let mut closed = false;
        framed(ui, rect, line.name, |ui| {
            closed = self.name_line(ui, line);
            body(self, ui);
        });
        if closed {
            self.close(line.close);
        }
    }

    /// The first line: one control, the touch floor tall, and the whole of it
    /// closes. Returns whether it was asked to — by a tap or by the platform's
    /// back, which this line takes because it is the depth to walk; except on
    /// the switcher, where back is left to leave the app (§22 item 7).
    fn name_line(&mut self, ui: &mut egui::Ui, line: &Line) -> bool {
        let wide = ui.available_width();
        let (rect, hit) =
            ui.allocate_exact_size(egui::vec2(wide, theme::TOUCH), egui::Sense::click());
        super::act::act(ui, &hit, line.reaches);
        self.note_control("close", ui, rect);
        let font = egui::TextStyle::Body.resolve(ui.style());
        let close = ui.painter().layout_no_wrap(
            crate::overlay::DONE.to_owned(),
            font,
            rgb(theme::INK_WEAK),
        );
        let mut named = super::theme::line(
            ui,
            &[
                (line.title.to_owned(), rgb(theme::INK)),
                (format!("  {}", line.subject), rgb(theme::INK_WEAK)),
            ],
        );
        named.wrap.max_width = (wide - close.size().x - space::M).max(0.0);
        let named = ui.ctx().fonts_mut(|fonts| fonts.layout_job(named));
        let middle = |size: egui::Vec2| rect.center().y - size.y / 2.0;
        let painter = ui.painter();
        painter.galley(
            egui::pos2(rect.left(), middle(named.size())),
            named,
            rgb(theme::INK),
        );
        let at = egui::pos2(rect.right() - close.size().x, middle(close.size()));
        painter.galley(at, close, rgb(theme::INK_WEAK));
        painter.hline(
            rect.x_range(),
            rect.bottom(),
            egui::Stroke::new(1.0, rgb(theme::HAIRLINE)),
        );
        hit.clicked() || (line.back && std::mem::take(&mut self.back))
    }

    /// **Open one** — and for the reads nobody holds standing, opening IS the
    /// ask (§13.8, §13.9, §13.11): a surface nobody has opened costs this
    /// device no radio. The picks of any earlier visit go with the opening,
    /// because an act addresses a row and a row nobody can see is not one.
    pub(in crate::shell) fn open(&mut self, overlay: Overlay) {
        self.drop_picks();
        self.overlay = Some(overlay);
        let Some(model) = self.model() else { return };
        match overlay {
            Overlay::Trail => model.list_trail(),
            Overlay::Balls(view) => model.list_balls(view),
            Overlay::Candidates => model.list_candidates(),
            Overlay::Clients => model.list_clients(),
            Overlay::Work => model.open_work(None),
            // The two reads the admin screen can make with no pick at all; a
            // config file is read by tapping its destination (§13.17).
            Overlay::Admin => {
                model.read_marks();
                model.list_proposals(None);
            }
            Overlay::SignIn => model.list_providers(),
            Overlay::Records => model.open_records(),
            Overlay::Files => model.open_files(None),
            // The queue is the held lane's (§14.1), the fleet reads nothing
            // (§13.13), the op table is compiled in (§13.14), the hits are
            // the search answer's own standing, and the switcher's two reads
            // are the standing set's (§14).
            Overlay::Fleet
            | Overlay::Help
            | Overlay::Queue
            | Overlay::Search
            | Overlay::Switcher
            | Overlay::Settings => (),
        }
    }

    /// What the name line or back asked for, performed.
    pub(in crate::shell) fn close(&mut self, close: Close) {
        match close {
            Close::Leave => {
                self.drop_picks();
                self.overlay = None;
            }
            Close::Forget => {
                if let Some(model) = self.model() {
                    model.forget();
                }
            }
            // The empty needle is the clear and crosses no wire
            // (`seat::asks::search`), so a search is left with the engine
            // unreachable.
            Close::Unsearch => {
                self.needle = String::new();
                if let Some(model) = self.model() {
                    model.search(String::new());
                }
            }
            Close::Exit => super::back::leave(&self.android),
        }
    }

    /// Every pick an overlay's acts address, and the arm beside them. A tail
    /// followed on one visit is not still followed on the next (§13.19), and
    /// it is the WORKER that holds which one, so that is a command.
    fn drop_picks(&mut self) {
        if let Some(model) = self.model() {
            model.watch_login(None);
        }
        self.armed = false;
        self.ball = None;
        self.candidate = None;
        self.destination = None;
        self.seeded = None;
        self.proposal = None;
        self.step = None;
        self.from = None;
    }
}
