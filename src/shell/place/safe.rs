//! **The safe area**: the one rectangle everything on the glass is laid out
//! in (DESIGN §13.2, bl-f557) — the window less whatever the platform has
//! standing over it, on **all four sides**.
//!
//! The platform draws this app edge to edge and then puts its own things on
//! top: the status bar, the navigation bar, the display cutout, the
//! keyboard. Which EDGE each one takes is the orientation's to say, not this
//! interface's — a three-button navigation bar is a strip along the bottom
//! of a phone held upright and a strip down one SIDE of the same phone held
//! flat, with the cutout on the other. So the shell never reasons about
//! *top* and *bottom*: it asks for four numbers (`shell/inset.rs`), turns
//! them into points here, and hands them to egui as its safe-area insets
//! (`app::pass`). From there `Context::content_rect` is the safe rectangle,
//! and the rect every screen is painted into, every `Area` and every popup
//! derive from it without a screen having to remember anything.
//!
//! Pure and host-tested, for `place`'s own reason: the probe and the paint
//! are `cfg(target_os = "android")` and no test in this suite reaches them.

/// What the platform covers on each side of the window, in physical pixels,
/// exactly as it answered.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Px {
    /// The left edge: a cutout or a navigation strip, in landscape.
    pub left: i32,
    /// The top edge: the status bar.
    pub top: i32,
    /// The right edge: a navigation strip or a cutout, in landscape.
    pub right: i32,
    /// The bottom edge: the taller of the keyboard and the navigation bar.
    pub bottom: i32,
}

/// The same four edges in egui's points.
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct Edges {
    /// Points covered at the left.
    pub left: f32,
    /// Points covered at the top.
    pub top: f32,
    /// Points covered at the right.
    pub right: f32,
    /// Points covered at the bottom.
    pub bottom: f32,
}

impl Px {
    /// These edges at `ppp` device pixels to the point. A side the probe
    /// answered nonsense for — a negative inset, or any inset at a scale
    /// that is not a positive number — covers nothing: that is not a reason
    /// to lay the interface out past the edge of the glass.
    pub fn points(self, ppp: f32) -> Edges {
        let side = |px: i32| {
            if ppp > 0.0 {
                px.max(0) as f32 / ppp
            } else {
                0.0
            }
        };
        Edges {
            left: side(self.left),
            top: side(self.top),
            right: side(self.right),
            bottom: side(self.bottom),
        }
    }
}

impl Edges {
    /// The safe rectangle of a window `wide` by `tall` points, as
    /// `[left, top, right, bottom]` — the arithmetic egui performs with
    /// these insets (`content_rect`), stated here so the rule can be
    /// asserted instead of eyeballed on a phone. Opposite edges that
    /// between them cover the whole window meet rather than cross.
    pub fn inside(self, wide: f32, tall: f32) -> [f32; 4] {
        let left = self.left.min(wide);
        let top = self.top.min(tall);
        [
            left,
            top,
            (wide - self.right).max(left),
            (tall - self.bottom).max(top),
        ]
    }
}

#[cfg(test)]
mod tests;
