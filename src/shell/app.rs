//! The frame loop: eframe boot, the inset pads, the IME mirror, and the
//! seat model the screens paint from. Every frame renders owned state and
//! blocks on nothing — the wire runs only on the model's worker thread.

use std::time::Duration;

use eframe::egui;
use winit::platform::android::activity::AndroidApp;

use super::boot::{Running, boot};
use super::bridge::{Bridge, Field, FieldKind};
use super::inset::InsetPx;
use crate::host::Host;
use crate::rows::AutoExpand;
use crate::seat::Model;

/// The one editable field the shell carries. The id string is the egui
/// widget id AND the bridge's address for it — one definition, used twice.
pub(crate) const COMPOSER: Field = Field {
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

pub(crate) struct Shell {
    android: AndroidApp,
    bridge: Bridge,
    pub(crate) running: Running,
    pub(crate) composer: String,
    /// Which KINDS of row open by default (the desktop's two knobs).
    pub(crate) auto: AutoExpand,
    /// The rows the operator has flipped by hand — overrides, never states:
    /// membership FLIPS a row's auto-state, so an empty set is "everything as
    /// configured" and the knobs above keep meaning what they say.
    pub(crate) folds: std::collections::BTreeSet<String>,
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
            running: boot(&android),
            android,
            bridge: Bridge::default(),
            composer: String::new(),
            auto: AutoExpand::default(),
            folds: std::collections::BTreeSet::new(),
            t0: std::time::Instant::now(),
            inset: InsetPx::default(),
            inset_at: 0,
        }
    }

    /// The seat model, when this launch is running one.
    pub(crate) fn model(&self) -> Option<&Model> {
        match &self.running {
            Running::Seat { model, .. } => Some(model),
            _ => None,
        }
    }

    pub(crate) fn model_mut(&mut self) -> Option<&mut Model> {
        match &mut self.running {
            Running::Seat { model, .. } => Some(model),
            _ => None,
        }
    }

    /// Who this device is on the wire, and as what: the leaf's own common
    /// name and the component its grade enrolled it as (REMOTE §2, §4.2).
    /// Painted rather than logged, because a seat showing an empty roster and
    /// a seat registered in no workspace look identical until this line says
    /// which client the engine was answering.
    pub(crate) fn identity(&self) -> String {
        match &self.running {
            Running::Seat { client, .. } => format!("{client} · seat"),
            Running::Foot { client, .. } => format!("{client} · foot grade"),
            Running::Cold { .. } => String::new(),
        }
    }

    /// The tool host, whichever component holds one.
    pub(crate) fn host_mut(&mut self) -> Option<&mut Host> {
        match &mut self.running {
            Running::Seat { host, .. } => host.as_mut(),
            Running::Foot { host, .. } => Some(host),
            Running::Cold { .. } => None,
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
