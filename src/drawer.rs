//! **The drawer gesture** (DESIGN §23 item 1): a drag that begins within the
//! glass's left edge and travels right opens the switcher — the one thing a
//! phone list has that the desktop's bar lacked, and the whole of what the
//! bar becomes under a thumb. Pure; `shell::transcript` reads the pointer
//! and asks here.

/// **How far in from the left edge a drag may begin**, in points.
pub const EDGE: f32 = 24.0;
/// **How far right it must travel** before it is a drawer and not a tap.
pub const REACH: f32 = 64.0;

/// Whether a press that began at `origin` and now stands at `now` (both
/// `x` in points from the glass's left edge) is the drawer opening.
#[must_use]
pub fn opens(origin: Option<f32>, now: Option<f32>) -> bool {
    match (origin, now) {
        (Some(from), Some(to)) => from <= EDGE && to - from >= REACH,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_drag_from_the_edge_that_reaches_opens_and_nothing_else_does() {
        assert!(opens(Some(10.0), Some(80.0)));
        assert!(opens(Some(EDGE), Some(EDGE + REACH)));
        assert!(!opens(Some(30.0), Some(200.0)), "not from the edge");
        assert!(!opens(Some(10.0), Some(60.0)), "not far enough");
        assert!(!opens(Some(10.0), None));
        assert!(!opens(None, Some(80.0)));
    }
}
