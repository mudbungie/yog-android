//! **The scan screen** (bl-d815): point the camera at the envelope a seat
//! rendered, and hand what it reads to the paste field's own sink.
//!
//! **It is a producer and nothing more.** A decode does not enroll anything —
//! it fills the same string the operator would have typed and
//! `super::material::land` spends it, so `crate::envelope`'s version check,
//! its grade-versus-certificate law and every sentence it can refuse with are
//! reached by exactly one path however the text arrived. There is no second
//! validation here to keep in step with that one.
//!
//! **The degraded path is always underneath.** A refused permission, a device
//! with no back lens, a camera another app is holding: each closes this screen
//! and writes its sentence into the enrollment screen's own refusal line, so
//! what the operator lands on is the paste field with an explanation — never a
//! preview that never fills.
//!
//! Android-only glue, excluded from coverage: the decode ([`crate::scan`]) and
//! the bridge vocabulary are pure and are tested.

use std::time::Duration;

use eframe::egui;
use winit::platform::android::activity::AndroidApp;

use crate::scan::{Camera, refusal, split};
use crate::shell::camera;

/// How long the scanner rests between decodes, in seconds. Frames arrive at
/// the camera's own rate and each decode is real work on the frame thread, so
/// the preview keeps every frame while the decoder takes roughly eight a
/// second — far more than a hand holding a phone steady needs, and a fraction
/// of the cost of decoding all of them.
const REST: f64 = 0.12;

/// The preview's subsample stride. The decoder reads the full plane; the
/// screen is a few hundred points wide and a quarter of the pixels is already
/// more than it can show.
const STRIDE: usize = 2;

/// The scan screen's own state. It holds an `AndroidApp` of its own because
/// every call it makes is *for an activity* — a permission dialog goes in
/// front of a screen — and because the handle living here keeps the
/// enrollment screen's argument list honest.
pub(crate) struct Scanner {
    android: AndroidApp,
    live: bool,
    asked: bool,
    opened: bool,
    preview: Option<egui::TextureHandle>,
    tried: f64,
}

impl Scanner {
    pub(crate) fn new(android: AndroidApp) -> Self {
        Self {
            android,
            live: false,
            asked: false,
            opened: false,
            preview: None,
            tried: 0.0,
        }
    }

    pub(super) fn live(&self) -> bool {
        self.live
    }

    /// The **scan QR** control was tapped.
    ///
    /// The permission is asked **per tap**, not per process. A refusal that
    /// stuck for the life of the app would make the second tap answer with the
    /// first tap's sentence and no dialog — while the platform's own rule is
    /// that a single denial is not final. Asking again is the operator's own
    /// gesture: they pressed the control that means *ask me*.
    pub(super) fn open(&mut self) {
        self.live = true;
        self.asked = false;
        self.opened = false;
    }

    /// Close the camera. Idempotent, and called on every way out: a decode, a
    /// refusal, the cancel control, and the enrollment screen's own back.
    pub(crate) fn shut(&mut self) {
        if self.live {
            camera::stop();
        }
        self.live = false;
        self.opened = false;
        self.preview = None;
    }

    /// One frame of the scan screen. `Some(text)` is a symbol read; the caller
    /// puts it straight into the paste field and lands it.
    ///
    /// A refusal is written into `said` — the enrollment screen's own line —
    /// and closes the screen, which is what makes a denied permission land the
    /// operator back on the paste field with a sentence rather than nowhere.
    pub(super) fn run(&mut self, ui: &mut egui::Ui, said: &mut Option<String>) -> Option<String> {
        // A preview is a moving picture, and the shell's idle pace is 250 ms
        // (DESIGN §3's focus-gated repaint). It is asked for FIRST, before any
        // branch that can close this screen, so the frame that closes it is
        // followed straight away by the one that paints the paste field —
        // egui keeps the shortest pending request, so asking here is enough.
        ui.ctx().request_repaint_after(Duration::from_millis(16));
        // Before anything is read, because asking is what clears the previous
        // answer: a `state` poll taken first would report the LAST tap's
        // refusal and close this screen before the dialog could go up. When
        // the permission is already held the platform answers without a
        // dialog, so the cost of asking unconditionally is nothing.
        if !self.asked {
            self.asked = true;
            camera::ask(&self.android);
        }
        let looked = camera::look(&self.android);
        if let Some(why) = refusal(&looked) {
            self.shut();
            *said = Some(why);
            return None;
        }
        if matches!(looked, Camera::Granted) && !self.opened {
            self.opened = true;
            camera::start(&self.android);
        }
        self.paint(ui, &looked)
    }

    /// The screen: a heading, the live frame, the way out — and, when a frame
    /// is due a decode, the decode.
    fn paint(&mut self, ui: &mut egui::Ui, looked: &Camera) -> Option<String> {
        if ui.button("< cancel").clicked() {
            self.shut();
            return None;
        }
        ui.heading("scan the envelope");
        ui.weak("hold the whole symbol in the frame; it lands the moment it reads.");
        let taken = camera::frame();
        if let Some(frame) = taken.as_deref() {
            self.hold(ui.ctx(), frame);
        }
        // The LAST frame is painted, not the newest one only: frames arrive at
        // the camera's rate and this loop runs at its own, so painting only on
        // arrival makes the preview flicker — and makes a stalled camera look
        // like a screen that never started.
        if let Some(handle) = &self.preview {
            ui.add(egui::Image::new(handle).max_width(ui.available_width()));
        }
        if matches!(looked, Camera::Asking) {
            ui.weak("waiting for the camera permission…");
        } else if self.preview.is_none() {
            ui.weak("starting the camera…");
        } else {
            ui.weak("looking…");
        }
        let frame = taken.as_deref()?;
        let now = ui.input(|i| i.time);
        if now - self.tried < REST {
            return None;
        }
        self.tried = now;
        let found = crate::scan::read(frame)?;
        self.shut();
        Some(found)
    }

    /// Take the frame the decoder is about to read into the preview texture —
    /// the same buffer, so the preview cannot show one thing while the decode
    /// works on another.
    fn hold(&mut self, ctx: &egui::Context, frame: &[u8]) {
        let Some((width, height, luma)) = split(frame) else {
            return;
        };
        let (width, height) = (width as usize, height as usize);
        let small = [width / STRIDE, height / STRIDE];
        if small[0] == 0 || small[1] == 0 {
            return;
        }
        let mut gray: Vec<u8> = Vec::with_capacity(small[0] * small[1]);
        for y in 0..small[1] {
            for x in 0..small[0] {
                let Some(pixel) = luma.get(y * STRIDE * width + x * STRIDE) else {
                    return;
                };
                gray.push(*pixel);
            }
        }
        let image = egui::ColorImage::from_gray(small, &gray);
        match &mut self.preview {
            Some(handle) => handle.set(image, egui::TextureOptions::LINEAR),
            None => {
                self.preview =
                    Some(ctx.load_texture("scan-preview", image, egui::TextureOptions::LINEAR));
            }
        }
    }
}
