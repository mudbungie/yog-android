//! The frame loop: eframe boot, the inset pads, the IME mirror, and the
//! skeleton screen the seat model (bl-5a98) replaces with real rows. Every
//! frame renders owned state and blocks on nothing — the wire never runs on
//! this thread.

use eframe::egui;
use winit::platform::android::activity::AndroidApp;

use super::bridge::{Bridge, Field, FieldKind};
use super::inset::InsetPx;

/// The one editable field the skeleton carries. The id string is the egui
/// widget id AND the bridge's address for it — one definition, used twice.
const COMPOSER: Field = Field {
    id: "composer",
    kind: FieldKind::Composer,
};

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

struct Shell {
    android: AndroidApp,
    bridge: Bridge,
    composer: String,
    /// Placeholder transcript pane until the seat model paints real rows.
    submitted: Vec<String>,
    t0: std::time::Instant,
    /// The inset pads and when they were last probed — the JNI walk is
    /// throttled to 200ms for numbers that change only when the keyboard
    /// slides (bl-014e).
    inset: InsetPx,
    inset_at: u128,
}

impl Shell {
    fn new(android: AndroidApp) -> Self {
        Self {
            android,
            bridge: Bridge::default(),
            composer: String::new(),
            submitted: Vec::new(),
            t0: std::time::Instant::now(),
            inset: InsetPx::default(),
            inset_at: 0,
        }
    }

    fn refresh_insets(&mut self, now: u128) {
        if now.saturating_sub(self.inset_at) < 200 {
            return;
        }
        self.inset_at = now;
        match super::inset::probe(&self.android) {
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
            let mut fields = [(COMPOSER, &mut self.composer)];
            self.bridge.run(&ctx, &self.android, &mut fields, now);
        }
        self.refresh_insets(now);

        let ppp = ctx.pixels_per_point();
        ui.add_space(self.inset.top as f32 / ppp);
        ui.heading("yog");
        ui.separator();

        // Bottom-up: the composer rides above the keyboard (or the
        // gesture-nav bar when the keyboard is down), then the transcript
        // placeholder takes whatever height remains.
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.add_space((self.inset.bottom as f32 / ppp).max(8.0));
            let r = ui.add(
                egui::TextEdit::singleline(&mut self.composer)
                    .id(egui::Id::new(COMPOSER.id))
                    .desired_width(f32::INFINITY)
                    .hint_text("message"),
            );
            if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                let taken = std::mem::take(&mut self.composer);
                if !taken.is_empty() {
                    self.submitted.push(taken);
                }
                r.request_focus();
            }
            ui.add_space(4.0);
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for line in &self.submitted {
                        ui.label(line);
                    }
                });
        });

        // The input-wake ruling (DESIGN §3, decided under bl-c761): no
        // vendored winit, so the commit wake winit drops (bl-2958) is
        // replaced by a focus-gated fast repaint. The focus is read HERE, at
        // the decision, from egui's settled memory — resetting a focus flag
        // before this read silently reverted the fix once (the bl-c761
        // dossier's named trap). Consuming a winit release with the wake arm
        // dissolves this poll.
        let focused = ctx.memory(egui::Memory::focused).is_some();
        let delay = if focused { 16 } else { 250 };
        ctx.request_repaint_after(std::time::Duration::from_millis(delay));
    }
}
