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

use super::{COMPOSER, NEEDLE, PASTED, REGISTER, Shell};

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
        "lernie",
        options,
        Box::new(move |cc| {
            // The language, installed once and before the first frame
            // (`shell::theme`, docs/STYLE.md).
            super::super::theme::install(&cc.egui_ctx);
            Ok(Box::new(Shell::new(handle)))
        }),
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
    /// **The platform's insets become egui's safe area, here and nowhere
    /// else** (bl-f557). egui has a first-class notion of the part of the
    /// viewport that is safe for content and derives everything from it —
    /// its content rect, the rect an `Area` is constrained to, where a
    /// popup may open — but `egui-winit` fills it in on iOS only, so on this
    /// platform it was zero and the shell subtracted what it knew by hand:
    /// the top and the bottom, at one site. A navigation bar on a SIDE of
    /// the glass, which is where landscape puts a three-button one, was
    /// subtracted by nothing. Feeding the four numbers in before the pass
    /// begins is the general path the hand-subtraction was a special case
    /// of.
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        self.refresh_insets(self.t0.elapsed().as_millis());
        // The scale this pass will be laid out at, which on the first pass
        // the context has not seen yet.
        let ppp = raw.viewport().native_pixels_per_point.map_or_else(
            || ctx.pixels_per_point(),
            |native| native * ctx.zoom_factor(),
        );
        let edges = self.inset.points(ppp);
        raw.safe_area_insets = Some(egui::SafeAreaInsets(egui::epaint::MarginF32 {
            left: edges.left,
            right: edges.right,
            top: edges.top,
            bottom: edges.bottom,
        }));
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let now = self.t0.elapsed().as_millis();

        // The bridge runs FIRST, on egui's settled focus, so the widgets
        // below lay out from text that is already current.
        {
            let mut fields = [
                (COMPOSER, &mut self.composer),
                (NEEDLE, &mut self.needle),
                (REGISTER, &mut self.door.url),
                (PASTED, &mut self.pasted),
            ];
            self.bridge.run(&ctx, &self.android, &mut fields, now);
        }
        // The platform's back press, read once and offered to the screens:
        // whatever has a depth to walk takes it (`shell::back`).
        self.back = crate::shell::back::pressed(&ctx);

        // **Everything is laid out in egui's content rect, which is the safe
        // area** (`raw_input_hook` above, bl-f557) — the root `Ui` itself is
        // the whole glass, bars and all. The platform's insets are spent
        // there, once, on all four sides, so every edge is structural rather
        // than a discipline — a bottom-up layout anchors to the floor, a
        // top-down scroller ends at it, a right-anchored control stops short
        // of a navigation bar standing on that side, and nothing downstream
        // has to remember anything (bl-9cfd is how the floor learned it).
        //
        // **What is left to subtract is this interface's own margin**: the
        // `place::GUTTER` it keeps above whatever the platform put at the
        // floor (bl-0691 — added to the inset, never maxed against it), and
        // a gutter on both sides, kept from the safe area's edge whether
        // that is the display's or a system bar's.
        let mut inside = ctx.content_rect();
        inside.max.y -= crate::shell::place::GUTTER;
        ui.scope_builder(egui::UiBuilder::new().max_rect(inside), |ui| {
            egui::Frame::NONE
                .inner_margin(egui::Margin::symmetric(10, 0))
                .show(ui, |ui| self.screens(ui));
        });
        // **A field nothing placed comes off the glass** (bl-8bbb). The
        // composer is a platform view drawn over the GL surface by the window
        // compositor, so it is not painted out of existence by a screen that
        // stops drawing it — it has to be told. Frame-scoped like `back`
        // above and taken here for the same reason: a screen that stops
        // painting a composer stops having one.
        self.field.rest(&self.android);
        // Nothing took it, so there was no depth to walk: the top of the app,
        // where back means what the platform means by it (bl-550e).
        if std::mem::take(&mut self.back) {
            crate::shell::back::leave(&self.android);
        }
        // **Every copy this pass asked for, handed to the platform**
        // (`shell::clip`, bl-7781). Before the probe and after the screens,
        // because it is the screens that ask: eframe's own clipboard is a
        // desktop one and does nothing here, so a command left in the output
        // would be a copy an operator watched fail silently.
        crate::shell::clip::copied(&ctx);
        // **The sentence a long press left** (§23 item 10), over everything
        // the screens painted, while the `help` knob is on.
        crate::shell::theme::told::paint(&ctx, self.knobs.on(crate::settings::Toggle::Help));
        // What this pass painted, said once per change (`app/probe.rs`): the
        // seam a headless-emulator harness reads reachability out of, because
        // the platform's accessibility tree carries nothing about an egui
        // surface. After the screens, because that is when they have said it.
        self.probe();
        // And what it could REACH, when a harness has armed the file
        // (`shell/act.rs`): the parity gate's inventory, self-reported only
        // because the platform route aborts the process (DESIGN §15.1).
        self.note_acts(&ctx);

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
