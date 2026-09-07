//! **The threading column** (DESIGN §13.20, bl-4d17): which indent columns a
//! conversation row's connector carries a rule through, and where its elbow
//! turns.
//!
//! The list was already a tree — bl-06d3 moved the recency order one rung up
//! so a child always sits directly under its parent — and it said so with
//! nothing but blank space. Blank space is a poor witness: two siblings a
//! screen apart look like two indents, an indent under a row whose own indent
//! is one unit shallower looks like the same rung, and the operator could not
//! tell a fan of three from three conversations somebody started. This module
//! is the reddit/HN idiom said as data — one rule per rung of depth, an elbow
//! into the row itself — so that the paint layer only spends what it is given.
//!
//! **The reading is taken off the descent, not off a parent field**, because
//! the wire carries no parent id: REMOTE §2.3's order is id-sorted siblings in
//! pre-order, so *"is there a later sibling at level L"* is answerable by
//! looking forward for the first row shallow enough to close the subtree. It
//! is exactly the same fact [`super::ordered`] relies on, read once more.

use crate::codec::ConvRow;

/// How deep a rail column may go, matching [`super::indent`]'s own cap for
/// one reason: the connector and the indent are two halves of one column, and
/// a row drawn at the capped offset with uncapped rails would hang its elbow
/// in a column the box does not start at. Past the cap the OUTERMOST rules
/// are the ones dropped, so the elbow is always the column the row's own box
/// begins at.
pub(super) const DEEPEST: usize = 8;

/// One entry per indent column, outermost first, saying whether a vertical
/// rule continues **past this row** in that column.
///
/// The last entry is the row's own: `true` means a sibling follows it (the
/// elbow's rule carries on down), `false` means it is the last child (the
/// elbow ends there). A root row's rails are empty, which is the general path
/// with no ancestor rather than an arm of its own.
pub fn threads(rows: &[ConvRow]) -> Vec<Vec<bool>> {
    (0..rows.len()).map(|at| rails(rows, at)).collect()
}

/// One row's rails. Walk forward from it, keeping the shallowest depth seen
/// so far: the first row that undercuts that floor is the first row to close
/// some subtree, and the level it lands ON is the level that has a later
/// sibling. Every deeper row between is inside a subtree already spoken for.
fn rails(rows: &[ConvRow], at: usize) -> Vec<bool> {
    let depth = rows.get(at).map_or(0, |row| row.depth);
    let mut rails = vec![false; depth];
    let mut floor = depth + 1;
    for row in rows.iter().skip(at + 1) {
        if row.depth >= floor {
            continue;
        }
        floor = row.depth;
        if let Some(rail) = floor.checked_sub(1).and_then(|level| rails.get_mut(level)) {
            *rail = true;
        }
        if floor == 0 {
            break;
        }
    }
    if let Some(over) = rails.len().checked_sub(DEEPEST).filter(|over| *over > 0) {
        rails.drain(..over);
    }
    rails
}

#[cfg(test)]
mod tests;
