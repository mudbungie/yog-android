//! **Where a drop-down's list goes** (DESIGN §13.2, bl-78c2): the band of
//! glass an opened list may occupy, decided from the TAPPABLE area rather
//! than from the display.
//!
//! `app::pass` spends the platform's bottom inset by shrinking the rect every
//! screen is painted into, and since bl-9cfd that is a fact for everything a
//! screen lays out — bl-192c then made the anchored bands claim it first, so
//! nothing painted past it. A popup is neither: egui puts it in an `Area` of
//! its own, positioned by `RectAlign::find_best_align` against
//! `Context::content_rect` and constrained to the same. That rect is the
//! viewport minus egui's **safe area**, a first-class notion this platform
//! never fills in — `egui-winit` reads it on iOS only — so on Android it is
//! the whole display, gesture-nav zone included, and a list opening downward
//! from a control that sits on the floor paints where taps never reach the
//! app (`shell/inset.rs`). Same class as the two fixes above it, one layer
//! further out, and this is its answer: **the list opens into the room the
//! tappable area actually has** — above the control when that is the roomier
//! side, capped to what it finds, scrolling inside the cap.
//!
//! Pure math and host-tested, which is the point rather than a convenience.
//! The paint stack is `cfg(target_os = "android")` and no test in this suite
//! can reach a line of it, so a rule that lived at the call site could not be
//! asserted at all — and this class has now been fixed three times without
//! one. `fit` decides, `list` states the band egui will then paint, and the
//! invariant *an opened list is inside the tappable area* is one composition
//! of the two. `place/tests.rs` holds it over a sweep of geometries; that
//! composition is the ratchet.

/// A stretch of glass: two edges, in egui's points, top first.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Band {
    /// The upper edge.
    pub top: f32,
    /// The lower edge.
    pub bottom: f32,
}

/// What `holds` forgives: a hundredth of a point, which is a few
/// ten-thousandths of a device pixel on the densest phone this seat runs on.
/// The placement arithmetic subtracts and re-adds the same edge, and f32
/// does not promise that round trip exactly — a rounding residue is not a
/// control crossing into the dead zone, and calling it one would make the
/// rule below unfalsifiable in the other direction.
const SLACK: f32 = 0.01;

impl Band {
    /// Does this band wholly contain `inner`? The invariant this module
    /// exists to keep, spelled once so the assertion and the prose are the
    /// same sentence.
    pub fn holds(&self, inner: Self) -> bool {
        inner.top >= self.top - SLACK && inner.bottom <= self.bottom + SLACK
    }
}

/// How a list is placed: which side of its control it opens on, and the
/// tallest its whole rect — frame included — may be.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Fit {
    /// The list opens upward, its bottom edge at the control's top.
    pub above: bool,
    /// The height of the list's whole rect.
    pub height: f32,
}

/// Where a control's list goes, or `None` when neither side of the control
/// has room for one.
///
/// `area` is the tappable band, `anchor` the control's own edges, `gap` the
/// space the popup leaves between the two, and `wanted` the height the list
/// would take if nothing stopped it — `f32::INFINITY` before it has ever
/// been laid out, which is honest rather than a sentinel: an unmeasured list
/// wants everything, and gets the room.
///
/// **Downward unless upward is roomier.** A list that fits below its control
/// opens below, because that is where a thumb looks for one; it flips only
/// when it does not fit there and the other side has more room, and it is
/// capped either way. Both rooms are measured from the anchor CLAMPED into
/// the band, so a control that is itself out of bounds still cannot hand
/// back a list that is.
pub fn fit(area: Band, anchor: Band, gap: f32, wanted: f32) -> Option<Fit> {
    let anchor = held(area, anchor);
    let below = (area.bottom - anchor.bottom - gap).max(0.0);
    let above = (anchor.top - area.top - gap).max(0.0);
    let up = wanted > below && above > below;
    let room = if up { above } else { below };
    (room > 0.0).then_some(Fit {
        above: up,
        height: wanted.min(room),
    })
}

/// The band an opened list occupies: the vertical half of the placement egui
/// performs for the two alignments `fit` chooses between (`RectAlign`'s
/// `TOP_START` and `BOTTOM_START`), stated here so the rule can be asserted
/// instead of eyeballed on a phone.
pub fn list(area: Band, anchor: Band, gap: f32, fit: Fit) -> Band {
    let anchor = held(area, anchor);
    if fit.above {
        Band {
            top: anchor.top - gap - fit.height,
            bottom: anchor.top - gap,
        }
    } else {
        Band {
            top: anchor.bottom + gap,
            bottom: anchor.bottom + gap + fit.height,
        }
    }
}

/// The control's own edges, held inside the band it is painted in. A control
/// outside the tappable area is not a case this module answers — it is a
/// layout defect one level up — but it must not become a list outside it,
/// and one clamp dissolves the whole class rather than answering it twice.
fn held(area: Band, anchor: Band) -> Band {
    Band {
        top: anchor.top.clamp(area.top, area.bottom),
        bottom: anchor.bottom.clamp(area.top, area.bottom),
    }
}

#[cfg(test)]
mod tests;
