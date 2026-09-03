//! **The drop-down** (DESIGN §13.2, bl-78c2): a selector whose opened list
//! lands inside the tappable area, above the control when that is where the
//! room is.
//!
//! **Why not `ComboBox`.** Its popup is an `Area` positioned by
//! `RectAlign::find_best_align` against `Context::content_rect` — the
//! viewport minus egui's safe area, which `egui-winit` fills in on iOS and
//! nowhere else, so on Android it is the whole display and the gesture-nav
//! zone is fair game. `ComboBox` exposes neither the alignment nor a
//! constraint rect, so there is no setter to reach for; the popup is
//! assembled here from the same pieces it uses one layer down — `Popup::menu`
//! over a button — given the side and the height `shell::place` decided.
//! Everything else about it is the combo's own recipe, including the
//! scrolled menu and the `Extend` wrap its items lay out under.
//!
//! **The rule is not here.** This file is the adapter: it reads the two rects
//! and spends the answer. `shell::place` decides, is pure, and is the only
//! half a host test can reach — which is the whole reason the seam is drawn
//! at this line (`tarpaulin.toml`).
//!
//! What changes for an operator is the direction a list opens and nothing
//! else. The `act:` tag still rides the face, because the affordance is the
//! control and not the transient list (`controls/pick.rs`).

use eframe::egui;

use crate::shell::place::{Band, fit};

/// The space between a control and its list. `Popup::menu`'s own default is
/// zero — a menu hangs off its button — and it is named here because the
/// placement arithmetic must be told the same number egui is.
const GAP: f32 = 0.0;

/// A selector: the current value, and a list of what it may become.
///
/// `area` is the tappable band (`Shell::controls` reads it off the rect
/// `app::pass` shrank). The returned response is the control's own — what
/// the caller tags, and what it asks its wire read on.
pub(super) fn drop_down(
    ui: &mut egui::Ui,
    area: Band,
    id: &str,
    shown: String,
    width: f32,
    add: impl FnOnce(&mut egui::Ui),
) -> egui::Response {
    // `push_id` rather than the button's own: a widget id derived from
    // position moves when the stop controls come and go, and a popup keyed
    // to it would close itself when a turn ends. The salt is the caller's
    // stable name, exactly as `ComboBox::from_id_salt` took it.
    let face = ui
        .push_id(id, |ui| {
            ui.add(
                egui::Button::new(shown)
                    .min_size(egui::vec2(width, 0.0))
                    .truncate(),
            )
        })
        .inner;
    caret(ui, face.rect);
    let popup = egui::Popup::menu(&face)
        .width(width)
        .gap(GAP)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClick);
    // What the list took last time it was laid out. Before the first one
    // there is no answer, and infinity is the honest one: an unmeasured list
    // wants everything, so it is handed the room and capped to it.
    let wanted = popup
        .get_expected_size()
        .map_or(f32::INFINITY, |size| size.y);
    let anchor = Band {
        top: face.rect.top(),
        bottom: face.rect.bottom(),
    };
    let Some(placed) = fit(area, anchor, GAP, wanted) else {
        // Neither side of this control has room for a list. Opening one
        // would put it where taps do not land, which is the defect, so
        // nothing opens.
        return face;
    };
    // The cap is on the whole rect; the scroller inside it gets what is left
    // once the popup's own frame has taken its margins.
    let chrome = egui::Frame::popup(ui.style()).total_margin().sum().y;
    popup
        .align(if placed.above {
            egui::RectAlign::TOP_START
        } else {
            egui::RectAlign::BOTTOM_START
        })
        // No alternatives: egui's fallback search judges fit against the
        // display, and the display is what put the list under the nav bar.
        .align_alternatives(&[])
        .show(|ui| {
            ui.set_min_width(ui.available_width());
            egui::ScrollArea::vertical()
                .max_height((placed.height - chrome).max(0.0))
                .show(ui, |ui| {
                    // A narrow popup would otherwise wrap its labels almost
                    // at once; the items widen the menu instead. `ComboBox`
                    // does the same, for the same reason.
                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                    add(ui);
                });
        });
    face
}

/// The triangle that says *this control opens a list*, drawn rather than
/// typed: the glyphs that carry one are not in the default font set, and a
/// tofu box is worse than no caret at all.
fn caret(ui: &egui::Ui, rect: egui::Rect) {
    let at = egui::Align2::RIGHT_CENTER
        .align_size_within_rect(egui::vec2(8.0, 5.0), rect.shrink2(egui::vec2(8.0, 0.0)));
    ui.painter().add(egui::Shape::convex_polygon(
        vec![at.left_top(), at.right_top(), at.center_bottom()],
        ui.visuals().text_color(),
        egui::Stroke::NONE,
    ));
}
