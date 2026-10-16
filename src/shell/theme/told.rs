//! **Every control tells, on a long press** (DESIGN §23 item 10; lernie
//! §4.46 item 2): the desktop's hover has no phone, and the long press is the
//! gesture a phone already uses for *what is this*. Every painter in this
//! directory takes the control's help sentence beside its word and ends in
//! [`told`], which records a long press on the control; the end of the pass
//! paints the sentence in the overlay's box directly above the control, where
//! it stands until the next tap anywhere — a pointer cannot leave a phone.
//!
//! **The sentence is a fact about the app and never about the world**, so
//! it may be said on the probe (`yog.told <words>`, `app/probe.rs`'s
//! channel): the walk's `told` beat long-presses a control and reads it
//! there, because the host compiles none of this and the emulator is the one
//! eye on it.

use eframe::egui;

use super::rgb;
use crate::theme::{self, space};

/// The marker the probe line begins with, beside `app::probe::MARKER`.
pub(crate) const MARKER: &str = "yog.told";

/// What a long press left for the end of the pass: where the control
/// stands, and its sentence.
#[derive(Clone)]
struct Told {
    rect: egui::Rect,
    words: String,
    /// Recorded this pass: the press that raised it is not the one that
    /// puts it down.
    fresh: bool,
}

/// Where the pass keeps it, in egui's own memory.
fn held() -> egui::Id {
    egui::Id::new("the control that told")
}

/// **Record a long press on `response`** as the control's sentence. Every
/// painter calls it last, so a control painted here cannot fail to tell.
pub(crate) fn told(ui: &egui::Ui, response: &egui::Response, help: &str) {
    if !response.long_touched() {
        return;
    }
    let said = Told {
        rect: response.rect,
        words: help.to_owned(),
        fresh: true,
    };
    ui.ctx().data_mut(|d| d.insert_temp(held(), said));
}

/// **Paint the sentence a long press left**, if one stands and `tells` is on
/// (the `help` knob, §23 item 11), and put it down on the next press. Called
/// once at the end of every pass, after the screens.
pub(crate) fn paint(ctx: &egui::Context, tells: bool) {
    let pressed = ctx.input(|i| i.pointer.any_pressed());
    let Some(said) = ctx.data(|d| d.get_temp::<Told>(held())) else {
        return;
    };
    // The next press anywhere puts it down — never the long one that raised
    // it, which `told` recorded this very pass.
    if pressed && !said.fresh {
        ctx.data_mut(|d| d.remove::<Told>(held()));
        return;
    }
    if said.fresh {
        ctx.data_mut(|d| {
            d.insert_temp(
                held(),
                Told {
                    fresh: false,
                    ..said.clone()
                },
            );
        });
    }
    if !tells {
        return;
    }
    probe(ctx, &said.words);
    let wide = ctx.content_rect().width() - space::S * 2.0;
    egui::Area::new(held().with("box"))
        .order(egui::Order::Foreground)
        .fixed_pos(egui::pos2(space::S, said.rect.top()))
        .pivot(egui::Align2::LEFT_BOTTOM)
        .interactable(false)
        .show(ctx, |ui| {
            egui::Frame::NONE
                .fill(rgb(theme::SURFACE))
                .stroke(egui::Stroke::new(1.0, rgb(theme::HAIRLINE)))
                .corner_radius(egui::CornerRadius::same(theme::RADIUS))
                .inner_margin(space::S)
                .show(ui, |ui| {
                    ui.set_max_width(wide);
                    ui.add(
                        egui::Label::new(egui::RichText::new(&said.words).color(rgb(theme::INK)))
                            .wrap(),
                    );
                });
        });
}

/// Say the sentence on the probe once per change: a repaint is not news.
fn probe(ctx: &egui::Context, words: &str) {
    let key = held().with("said");
    let last = ctx.data(|d| d.get_temp::<String>(key));
    if last.as_deref() == Some(words) {
        return;
    }
    log::info!("{MARKER} {words}");
    ctx.data_mut(|d| d.insert_temp(key, words.to_owned()));
}
