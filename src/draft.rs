//! **The draft, and which side of the mirror moved** (bl-8bbb).
//!
//! The composer is a native Android `EditText` overlaid at the rectangle egui
//! lays out for it (DESIGN §13.2), and **the field's own text and cursor are
//! the one home of the draft**. That buys cursor placement, selection
//! handles, the paste menu, autocorrect and the IME's own behaviour for free
//! — none of which an egui `TextEdit` fed by the `GameTextInput` mirror could
//! offer, because that mirror adopts a committed buffer wholesale and so
//! appends every keystroke at the end.
//!
//! Two decisions come out of that arrangement and both are pure, so both are
//! asserted here rather than on a device:
//!
//! * **What the field said.** One JNI crossing per frame places the view and
//!   reads it back, and its answer wears the two-line protocol every bridged
//!   call in this crate wears ([`read`]). A refusal is a sentence; an answer
//!   is a focus flag, the height the field's own text wants, and the text.
//!   **A malformed answer is never echoed back into a message**: the draft is
//!   the operator's prose, and a refusal that quoted it would put it in
//!   logcat, which is device-wide (DESIGN §15.2).
//! * **Which side moved** ([`Mirror`]). The field is authoritative, so a
//!   change there is ADOPTED; but the app writes the draft too — a send takes
//!   it, a refused deposit gives it back (DESIGN §13.2's outbox), a row menu
//!   spends it — and a change here is PUSHED. Comparing both against what was
//!   last agreed is what tells them apart, and it is the whole of the rule: a
//!   frame where neither moved touches nothing, which is what keeps a push
//!   from restarting the IME's composing session every frame.

use crate::theme::TOUCH;

/// How tall the field may grow before it scrolls inside itself, in points.
/// The floor is the touch target (`theme::TOUCH`) and this is the ceiling: a
/// field that grew unbounded would push the transcript off the glass.
pub const CAP: f32 = 132.0;

/// What the native field said about itself this frame.
pub struct Said {
    /// Whether the field holds the caret. The shell repaints fast while it
    /// does, because a native view's edits wake no egui frame.
    pub focused: bool,
    /// The height the field's own text wants, in device pixels — its
    /// measurement, not a guess taken here.
    pub high: i32,
    /// The draft, verbatim.
    pub text: String,
}

/// Read the field bridge's answer: `ok`, a focus flag, a height and the text,
/// or the `err` sentence a bridged call refuses with.
///
/// The text is everything after the third line, newlines included — a draft
/// is prose and prose has lines in it.
pub fn read(answer: &str) -> Result<Said, String> {
    let unreadable = || "the composer field answered a shape this build cannot read".to_owned();
    let Some((head, rest)) = answer.split_once('\n') else {
        return Err(unreadable());
    };
    if head == "err" {
        return Err(rest.to_owned());
    }
    if head != "ok" {
        return Err(unreadable());
    }
    let mut parts = rest.splitn(3, '\n');
    let focused = match parts.next() {
        Some("1") => true,
        Some("0") => false,
        _ => return Err(unreadable()),
    };
    let Some(Ok(high)) = parts.next().map(str::parse) else {
        return Err(unreadable());
    };
    Ok(Said {
        focused,
        high,
        text: parts.next().unwrap_or_default().to_owned(),
    })
}

/// The band the composer row stands at, in points: the field's own measured
/// content, floored at the touch target and capped at [`CAP`].
///
/// **It grows only while the field holds the caret** (bl-0691). The band is
/// taken off the top of the transcript, on the screen the transcript is the
/// whole point of, and a draft nobody is typing does not need to be read four
/// lines at a time — an unfocused composer rests at the touch floor and
/// scrolls inside itself, with every word of the draft still there the moment
/// it is tapped. The cap is untouched: this decides which of the two numbers
/// the measurement is clamped to, not what the ceiling is.
///
/// A device pixel scale of zero or less is not a scale — the platform has not
/// laid out yet — and the honest answer there is the resting height.
pub fn band(high: i32, ppp: f32, focused: bool) -> f32 {
    if ppp <= 0.0 || !focused {
        return TOUCH;
    }
    (high as f32 / ppp).clamp(TOUCH, CAP)
}

/// Which side of the mirror moved since the last frame.
#[derive(Debug, PartialEq, Eq)]
pub enum Move {
    /// The field moved: take its text as the draft.
    Adopt(String),
    /// This side moved: hand the text to the field.
    Push(String),
    /// Neither moved.
    Still,
}

/// What the two sides last agreed the draft was.
#[derive(Default)]
pub struct Mirror {
    seen: String,
}

impl Mirror {
    /// Which side moved, and what the other one owes. The FIELD is asked
    /// first because it is the one home of the draft: a frame in which both
    /// changed is a keystroke racing a programmatic write, and the operator's
    /// own typing is what must survive it.
    pub fn step(&mut self, native: &str, app: &str) -> Move {
        if native != self.seen {
            native.clone_into(&mut self.seen);
            return Move::Adopt(native.to_owned());
        }
        if app != self.seen {
            app.clone_into(&mut self.seen);
            return Move::Push(app.to_owned());
        }
        Move::Still
    }

    /// Record text this side handed the field outside a [`Mirror::step`] — a
    /// send clearing it. Without it the next step would read the cleared
    /// field as the FIELD having moved and adopt an emptiness back over a
    /// draft the app had already replaced.
    pub fn handed(&mut self, text: &str) {
        text.clone_into(&mut self.seen);
    }
}

#[cfg(test)]
mod tests;
