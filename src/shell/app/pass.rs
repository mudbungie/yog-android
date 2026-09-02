//! **One frame**: boot eframe over the Activity, then, per pass, run the IME
//! mirror, re-probe the platform insets, paint the screens and schedule the
//! next repaint.
//!
//! A child of `app` so it can reach the `Shell`'s private state without that
//! state becoming an interface. (Nothing to do with `crate::frame`, which is
//! the wire's length-prefixed framing.)

use std::time::Duration;

use eframe::egui;
use winit::platform::android::activity::AndroidApp;

use super::{COMPOSER, ENVELOPE, Shell};

/// The least clearance the bottom of the glass gets when the platform reports
/// no inset at all — a display with no gesture bar and no keyboard up still
/// should not put a control on the physical edge.
const GUTTER: f32 = 8.0;

/// Boot eframe over the Activity. `sys::android_main` is the only caller;
/// everything before this call is sys.rs's.
pub(crate) fn run(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );
    let handle = app.clone();
    let options = eframe::NativeOptions {
        android_app: Some(app),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    if let Err(e) = eframe::run_native(
        "yog",
        options,
        Box::new(move |_| Ok(Box::new(Shell::new(handle)))),
    ) {
        // The process is over either way; logcat is the only witness.
        log::error!("eframe: {e}");
    }
}

impl Shell {
    fn refresh_insets(&mut self, now: u128) {
        if now.saturating_sub(self.inset_at) < 200 {
            return;
        }
        self.inset_at = now;
        match crate::shell::inset::probe(&self.android) {
            Ok(px) => self.inset = px,
            Err(e) => log::warn!("inset probe: {e}"),
        }
    }
}

impl eframe::App for Shell {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let now = self.t0.elapsed().as_millis();

        // The bridge runs FIRST, on egui's settled focus, so the widgets
        // below lay out from text that is already current.
        {
            let mut fields = [
                (COMPOSER, &mut self.composer),
                (ENVELOPE, &mut self.envelope),
            ];
            self.bridge.run(&ctx, &self.android, &mut fields, now);
        }
        self.refresh_insets(now);
        // The platform's back press, read once and offered to the screens:
        // whatever has a depth to walk takes it (`shell::back`).
        self.back = crate::shell::back::pressed(&ctx);

        let ppp = ctx.pixels_per_point();
        ui.add_space(self.inset.top as f32 / ppp);
        // **The bottom inset is a hard floor, and it is spent HERE — once,
        // for every screen** (bl-9cfd). It used to be an `add_space` at the
        // top of the two screens that remembered it, which made the floor a
        // discipline rather than a fact: a screen that forgot it painted into
        // the gesture-nav bar, and even one that spent it could be overflowed
        // from the inside (a `ScrollArea` will not go below
        // `min_scrolled_height` no matter how little room it is given — see
        // `chat::composer`). Shrinking the rect every screen is painted into
        // makes the floor structural: a bottom-up layout anchors to it, a
        // top-down scroller ends at it, and nothing downstream has to
        // remember anything.
        let mut inside = ui.available_rect_before_wrap();
        inside.max.y -= (self.inset.bottom as f32 / ppp).max(GUTTER);
        // A gutter on both sides. The platform insets carry only top and
        // bottom (`inset.rs`: the status bar and the taller of keyboard and
        // gesture-nav), so nothing else was holding content off the display's
        // own edge — the first-run heading was painting with its first glyph
        // half off the glass, and every full-width button ran edge to edge.
        ui.scope_builder(egui::UiBuilder::new().max_rect(inside), |ui| {
            egui::Frame::NONE
                .inner_margin(egui::Margin::symmetric(10, 0))
                .show(ui, |ui| self.screens(ui));
        });
        // Nothing took it, so there was no depth to walk: the top of the app,
        // where back means what the platform means by it (bl-550e).
        if std::mem::take(&mut self.back) {
            crate::shell::back::leave(&self.android);
        }

        // The input-wake ruling (DESIGN §3, decided under bl-c761): no
        // vendored winit, so the commit wake winit drops (bl-2958) is
        // replaced by a focus-gated fast repaint. The focus is read HERE, at
        // the decision, from egui's settled memory — resetting a focus flag
        // before this read silently reverted the fix once (the bl-c761
        // dossier's named trap). Consuming a winit release with the wake arm
        // dissolves this poll.
        let focused = ctx.memory(egui::Memory::focused).is_some();
        let delay = if focused { 16 } else { 250 };
        ctx.request_repaint_after(Duration::from_millis(delay));
    }
}
