//! **The one adapter from the language to egui** (`docs/STYLE.md`,
//! bl-549b): `crate::theme`'s bytes become `Visuals` once, at app start, and a
//! `Color32` at the paint sites that colour a thing by its state. Nothing else
//! in the shell constructs a colour (`rules/no-literal-colour.yml`); a screen
//! that wants one asks this file by the state's name.
//!
//! **No outline anywhere.** Every widget stroke egui would draw at rest is
//! `NONE`, and a block is raised from the ground by its fill: a row is bare
//! ground until a thumb is on it, a field and a popup are `SURFACE`, a pressed
//! or opened control is `RAISED`. The one stroke that survives is the
//! noninteractive hairline, which is what `ui.separator()` and a resting
//! field's edge read, and it is one point of `HAIRLINE` — the most a boundary
//! may be. The focus ring is the brand, because a field with the caret in it
//! is the one thing on the glass that is the operator's, now.

use eframe::egui;

use crate::codec::Tone;
use crate::rows::Role;
use crate::theme::{self, State, space, type_scale};

/// A token as egui's colour.
pub(crate) fn rgb(rgb: theme::Rgb) -> egui::Color32 {
    egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2])
}

/// The mark's walk carries straight alpha (`crate::icon`); this is the one
/// place it is read into egui.
pub(crate) fn rgba(fill: [u8; 4]) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(fill[0], fill[1], fill[2], fill[3])
}

/// A token as the platform's own packed ARGB. `from_be_bytes` rather than a
/// cast: `0xFF00_0000` does not fit a positive `i32`, and a `u32 as i32` is
/// exactly the wrap `clippy::pedantic` refuses.
fn argb(rgb: theme::Rgb) -> i32 {
    i32::from_be_bytes([255, rgb[0], rgb[1], rgb[2]])
}

/// **The composer's native field, dressed in the language** (bl-8bbb;
/// STYLE.md, *the composer*): a `SURFACE` ground with no stroke at rest and
/// the brand ring when it holds the caret, its words in `INK` and its hint in
/// `INK_FAINT`. This file is the one adapter for a platform view exactly as
/// it is for egui — the difference is only that the platform takes packed
/// ARGB and device pixels where egui takes a `Color32` and points.
///
/// **The ring's width is one point**, the most a boundary may be on this
/// glass (STYLE.md §2): what a focused field changes is the brand, never the
/// weight. `pad` is the composer's own derived padding, so the native field
/// rests at the touch floor for the same reason every egui field does.
pub(crate) fn skin(pad: egui::Margin, ppp: f32) -> super::field::Skin {
    let px = |v: f32| (v * ppp).round().max(1.0) as i32;
    super::field::Skin {
        ground: argb(theme::SURFACE),
        ink: argb(theme::INK),
        faint: argb(theme::INK_FAINT),
        ring: argb(theme::BRAND),
        radius: px(f32::from(theme::RADIUS)),
        pad_x: px(f32::from(pad.left)),
        pad_y: px(f32::from(pad.top)),
        text: px(type_scale::BODY),
        ring_px: px(1.0),
        scale: ppp,
    }
}

/// A state's accent, for ink or a rule.
pub(crate) fn ink(state: State) -> egui::Color32 {
    rgb(theme::accent(state))
}

/// A row tone's ink (REMOTE §11), one map for the transcript and the list.
pub(crate) fn tone(tone: &Tone) -> egui::Color32 {
    rgb(theme::tone(tone))
}

/// A speaker's rule.
pub(crate) fn speaker(role: Role) -> egui::Color32 {
    rgb(theme::speaker(role))
}

/// Install the language into the context. Called once, from
/// `app::pass::run`'s creation closure, before the first frame; every screen
/// then reads it through `ui.visuals()` and `ui.style()`.
pub(crate) fn install(ctx: &egui::Context) {
    // The language is dark and has no light face: a preference pinned here
    // is what keeps a system theme flip from swapping in egui's own light
    // palette under these tokens.
    ctx.options_mut(|options| options.theme_preference = egui::ThemePreference::Dark);
    // **Built in ONE initializer, never assigned onto a default** (bl-3d34).
    // `clippy::field_reassign_with_default` is `deny` through the manifest's
    // pedantic tier and this module is android-only, so the host gate cannot
    // see the site — the release workflow's cross-clippy step is the only
    // thing that lints it, and it was red on every push from the moment this
    // function landed, which stops the APK job before it builds anything.
    // Struct-update for what the language does not state: egui's defaults are
    // the rest of the style, and naming them here would be a second copy of
    // egui's own answer.
    let style = egui::Style {
        text_styles: [
            (
                egui::TextStyle::Small,
                egui::FontId::proportional(type_scale::SMALL),
            ),
            (
                egui::TextStyle::Body,
                egui::FontId::proportional(type_scale::BODY),
            ),
            (
                egui::TextStyle::Button,
                egui::FontId::proportional(type_scale::BODY),
            ),
            (
                egui::TextStyle::Heading,
                egui::FontId::proportional(type_scale::HEADING),
            ),
            (
                egui::TextStyle::Monospace,
                egui::FontId::monospace(type_scale::MONO),
            ),
        ]
        .into(),
        spacing: egui::style::Spacing {
            item_spacing: egui::vec2(space::S, space::S),
            button_padding: egui::vec2(space::M, space::S),
            menu_margin: egui::Margin::same(space::S as i8),
            window_margin: egui::Margin::same(space::M as i8),
            ..Default::default()
        },
        visuals: visuals(),
        ..Default::default()
    };
    ctx.set_style_of(egui::Theme::Dark, style);
}

/// The palette as egui's `Visuals`: dark, outline-free, tinted by elevation.
fn visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();
    let radius = egui::CornerRadius::same(theme::RADIUS);
    let text = egui::Stroke::new(1.0, rgb(theme::INK));
    let hairline = egui::Stroke::new(1.0, rgb(theme::HAIRLINE));
    let face = |fill: theme::Rgb, stroke: egui::Stroke| egui::style::WidgetVisuals {
        bg_fill: rgb(fill),
        weak_bg_fill: rgb(fill),
        bg_stroke: stroke,
        corner_radius: radius,
        fg_stroke: text,
        expansion: 0.0,
    };
    visuals.widgets.noninteractive = face(theme::GROUND, hairline);
    visuals.widgets.inactive = face(theme::SURFACE, egui::Stroke::NONE);
    visuals.widgets.hovered = face(theme::RAISED, egui::Stroke::NONE);
    visuals.widgets.active = face(theme::RAISED, egui::Stroke::NONE);
    visuals.widgets.open = face(theme::RAISED, egui::Stroke::NONE);
    visuals.weak_text_color = Some(rgb(theme::INK_WEAK));
    visuals.selection.bg_fill = rgb(theme::BRAND).gamma_multiply(0.35);
    visuals.selection.stroke = egui::Stroke::new(1.0, rgb(theme::BRAND));
    visuals.hyperlink_color = rgb(theme::BRAND);
    visuals.panel_fill = rgb(theme::GROUND);
    visuals.faint_bg_color = rgb(theme::SURFACE);
    visuals.extreme_bg_color = rgb(theme::SURFACE);
    visuals.text_edit_bg_color = Some(rgb(theme::SURFACE));
    visuals.code_bg_color = rgb(theme::SURFACE);
    visuals.window_fill = rgb(theme::RAISED);
    visuals.window_stroke = egui::Stroke::NONE;
    visuals.window_corner_radius = radius;
    visuals.menu_corner_radius = radius;
    visuals.warn_fg_color = ink(State::Annotation);
    visuals.error_fg_color = ink(State::Error);
    let shadow = egui::epaint::Shadow {
        offset: [0, 4],
        blur: 14,
        spread: 0,
        color: egui::Color32::from_black_alpha(140),
    };
    visuals.window_shadow = shadow;
    visuals.popup_shadow = shadow;
    visuals
}
/// The anatomy `docs/STYLE.md` §5 spells — the row, the line, the chip —
/// split out at the 300-line cap (bl-0691) on the seam this file already had:
/// above is the LANGUAGE (the palette as egui's `Visuals`, installed once,
/// and a colour by a state's name), below is what is BUILT out of it.
mod anatomy;

pub(crate) use anatomy::{chip, dark_chip, line, marked, row, share};
