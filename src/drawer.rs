//! **The drawer** (DESIGN §24 item 2; lernie §4.44's bar under a thumb): the
//! one surface every place that is not about the focused conversation is
//! opened from — the workspace, the find field, the conversation tree, the
//! places and the seat's foot — pulled out from the left edge over the chat.
//! Pure and host-tested: how wide it is, when it stands, what opens and
//! closes it by gesture, what its head and its rows say, and how many are
//! waiting. `shell/drawer.rs` paints it and performs the taps.
//!
//! **It stands open while nothing is focused, and cannot close** (§24 item
//! 1): with no workspace there is no chat to paint under it, so the drawer is
//! the general path with empty inputs rather than a roster screen of its own.
//! [`stands`] is that rule, and the head's list obeys it too — with nothing
//! focused the list is open under `workspace ⏶`, the one thing there is to
//! do.

use crate::codec::{AgentState, ConvRow, WsRow};
use crate::theme::{self, Rgb, State, glyph};

/// **How far in from the left edge a drag may begin**, in points.
pub const EDGE: f32 = 24.0;
/// **How far it must travel** before it is a drawer and not a tap — right to
/// open, left from inside the drawer to close.
pub const REACH: f32 = 64.0;
/// **The most of the glass the drawer takes**, and the most it is ever wide:
/// five sixths, never more than 360 points — a phone's whole width less a
/// strip of the chat to tap back onto, and a tablet's list rather than its
/// half.
pub const SHARE: f32 = 5.0 / 6.0;
pub const CAP: f32 = 360.0;
/// **The scrim's alpha** over what is left of the chat: dark enough that the
/// drawer is what the eye is on, light enough that the chat is still there.
pub const SCRIM: u8 = 160;
/// **How long the edge takes to travel**, in seconds — the desktop's number
/// for the bar's edge (lernie §4.44 item 1): fast, and not a snap.
pub const EASE: f32 = 0.12;

/// The head's words with nothing focused, and the find field's hint.
pub const WORKSPACE: &str = "workspace";
pub const FIND: &str = "find";
/// What an empty list under a focused workspace says: it names the next act.
pub const EMPTY: &str = "no conversations here yet — + begins one";
/// The foot's row, which is also the overlay it opens.
pub const SETTINGS: &str = "settings";

/// **How wide the drawer is** on a glass `glass` points across.
#[must_use]
pub fn width(glass: f32) -> f32 {
    (glass * SHARE).clamp(0.0, CAP)
}

/// Whether a press that began at `origin` and now stands at `now` (both
/// `x` in points from the glass's left edge) is the drawer opening.
#[must_use]
pub fn opens(origin: Option<f32>, now: Option<f32>) -> bool {
    match (origin, now) {
        (Some(from), Some(to)) => from <= EDGE && to - from >= REACH,
        _ => false,
    }
}

/// Whether a press that began inside a drawer `width` wide and travelled
/// left past [`REACH`] is the drawer closing.
#[must_use]
pub fn closes(origin: Option<f32>, now: Option<f32>, width: f32) -> bool {
    match (origin, now) {
        (Some(from), Some(to)) => from <= width && from - to >= REACH,
        _ => false,
    }
}

/// **Whether the drawer stands** — or, the same rule one level in, whether
/// the head's list is open: because it was opened, or because nothing is
/// focused and there is nothing else to paint.
#[must_use]
pub fn stands(open: bool, focused: bool) -> bool {
    open || !focused
}

/// **What the head's door says**: the focused workspace's name, or the word.
#[must_use]
pub fn head(focused: Option<&str>) -> String {
    focused.map_or_else(|| WORKSPACE.to_owned(), str::to_owned)
}

/// **The seat-wide waiting count** (§24 item 6): the `attention` chip's
/// number, summed over every workspace the roster lists.
#[must_use]
pub fn waiting(workspaces: &[WsRow]) -> usize {
    workspaces.iter().map(|ws| ws.attention).sum()
}

/// **One conversation row as the desktop bar reads it** (lernie §4.44 item
/// 1): the state glyph in the row's ink, the asking `?` in the attention
/// accent before the name, the name in `INK` (the focused one in the brand),
/// one line beneath in `INK_WEAK`, and the age at the right end.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Words {
    pub mark: (String, Rgb),
    pub asking: bool,
    pub name: (String, Rgb),
    /// The failure where the server reddened the row, else the preview, each
    /// folded to one line; nothing where the row says nothing more.
    pub beneath: Option<String>,
    pub age: String,
}

/// **The state glyph**: the desktop's table (lernie `convs::glyph`).
#[must_use]
pub fn mark(state: &AgentState) -> String {
    match state {
        AgentState::Live | AgentState::InFlight => glyph::WORKING,
        AgentState::Quiescent => glyph::AT_REST,
        AgentState::Stopped => glyph::STOPPED,
        AgentState::Unknown(_) => glyph::NOTE,
    }
    .to_owned()
}

/// What a row says, read off the wire's row and this device's clock.
#[must_use]
pub fn words(row: &ConvRow, focused: bool, now_unix: i64) -> Words {
    let beneath = row
        .failure
        .as_ref()
        .map(|why| format!("failed · {}", crate::roster::one_line(why)))
        .or_else(|| Some(crate::roster::one_line(&row.preview)).filter(|line| !line.is_empty()));
    Words {
        mark: (mark(&row.state), theme::tone(&row.tone)),
        asking: row.attention > 0,
        name: (
            row.display.clone(),
            if focused { theme::BRAND } else { theme::INK },
        ),
        beneath,
        age: crate::roster::stamp(row.last_active_unix, now_unix),
    }
}

/// The accent the asking mark wears.
#[must_use]
pub fn asking_ink() -> Rgb {
    theme::accent(State::Attention)
}

#[cfg(test)]
mod tests;
