//! **The connection banner** (DESIGN §13.2): the one sentence that is about
//! the WIRE rather than about the screen it interrupts.
//!
//! Its own file since bl-7781 took [`super`] to the cap — and on a real seam
//! rather than a line count: every other thing in that module is a screen or
//! a row of one, and this is neither. It is painted by four surfaces, under
//! the bar, wherever the operator happens to be.

use eframe::egui;

use crate::seat::Snapshot;
use crate::shell::theme::ink;
use crate::theme::State;

/// **What a wire being re-dialled says** (bl-eec1). The lower-case is the
/// banner's own register, and the ellipsis is the whole of the claim: it is
/// happening, it has not failed, and nothing is being asked of the operator.
const RECONNECTING: &str = "reconnecting…";

/// The connection banner: what the worker is standing on, under the bar and
/// above whatever screen it interrupted (§13.2).
///
/// **Two sentences and two states, never one** (bl-eec1). A wire being
/// re-dialled is the WORKING accent, because re-dialling is work; a failure
/// that outlived its grace, and a gesture the engine refused, are the ERROR
/// accent, because they will not mend themselves. The model publishes them as
/// two fields that cannot both be about the wire, so this paints whichever it
/// is handed and decides nothing.
pub(crate) fn banner(ui: &mut egui::Ui, snap: &Snapshot) {
    if snap.reconnecting {
        crate::shell::clip::tinted(ui, ink(State::Working), RECONNECTING);
    }
    if let Some(error) = &snap.error {
        // Copyable like every other prose surface (bl-7781): an engine's own
        // sentence is exactly the text an operator wants out of the phone.
        crate::shell::clip::tinted(ui, ink(State::Error), error);
    }
}
