//! The frame loop: eframe boot, the inset pads, the IME mirror, and the
//! seat model the screens paint from. Every frame renders owned state and
//! blocks on nothing — the wire runs only on the model's worker thread.

use std::time::Duration;

use eframe::egui;
use winit::platform::android::activity::AndroidApp;

use super::bridge::{Bridge, Field, FieldKind};
use super::inset::InsetPx;
use crate::seat::Model;

/// The one editable field the shell carries. The id string is the egui
/// widget id AND the bridge's address for it — one definition, used twice.
pub(crate) const COMPOSER: Field = Field {
    id: "composer",
    kind: FieldKind::Composer,
};

/// How long the model rests between unprompted refreshes — the human
/// cadence of a chat glanced at, not a terminal streamed to.
const CADENCE: Duration = Duration::from_secs(2);

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

pub(crate) struct Shell {
    android: AndroidApp,
    bridge: Bridge,
    /// The seat model, or the sentence explaining why there is none
    /// (unprovisioned material; provisioning is an operator act, DESIGN §5).
    pub(crate) model: Result<Model, String>,
    pub(crate) composer: String,
    t0: std::time::Instant,
    /// The inset pads and when they were last probed — the JNI walk is
    /// throttled to 200ms for numbers that change only when the keyboard
    /// slides (bl-014e).
    pub(crate) inset: InsetPx,
    inset_at: u128,
}

impl Shell {
    fn new(android: AndroidApp) -> Self {
        Self {
            model: open_model(&android),
            android,
            bridge: Bridge::default(),
            composer: String::new(),
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

/// The seat over the provisioned material in the app's private `files/wire`
/// (DESIGN §5: adb, remote exec or QR put it there; this app never mints).
fn open_model(android: &AndroidApp) -> Result<Model, String> {
    let dir = android
        .internal_data_path()
        .ok_or("no internal data path")?
        .join("wire");
    let material = crate::material::read_dir(&dir)?
        .ok_or_else(|| format!("nothing provisioned at {}", dir.display()))?;
    let seat = crate::transport::Seat::open(&material)?;
    Ok(Model::start(seat, CADENCE))
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
        self.screens(ui);

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
