//! **The editable fields the IME mirror addresses** — the three ids that are
//! an egui widget id AND the bridge's address for the same field, split out
//! of `app.rs` (bl-5a56) on the seam that file's own first sentence draws:
//! what the shell IS, and what the platform's keyboard can be pointed at.
//!
//! One definition used twice, per field, which is the whole reason they are
//! constants: a widget id spelled at the paint site and an address spelled at
//! the bridge would be two names for one field, and the IME would be told
//! about a field nothing painted.

use crate::shell::bridge::{Field, FieldKind};

/// The one editable field the shell carries. The id string is the egui
/// widget id AND the bridge's address for it — one definition, used twice.
pub(crate) const COMPOSER: Field = Field {
    id: "composer",
    kind: FieldKind::Composer,
};

/// The enrollment screen's envelope field. A separate id because the two are
/// never on screen together for the composer's reason inverted: a cold device
/// has no conversation to speak into, and a provisioned one has nothing left
/// to enroll — but they are different KINDS of editor (`bridge.rs`), and the
/// IME must be told which it is focused on.
pub(crate) const ENVELOPE: Field = Field {
    id: "envelope",
    kind: FieldKind::Envelope,
};

/// **The search field** (§13.6, bl-4c2b). Its own id and its own kind: a
/// needle is one line and must not be autocorrected — a corrected needle
/// searches for a word the operator did not type — while the composer is
/// prose and wants both.
pub(crate) const NEEDLE: Field = Field {
    id: "needle",
    kind: FieldKind::Needle,
};
