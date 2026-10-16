//! **The lines as the glass bubbles them** (DESIGN §23 item 7): which lines
//! stand on one fill, and which role's fill it is. Pure, so the shape of a
//! turn's blocks answers to the suite and the shell only paints frames.
//!
//! A block is a run of lines under one bubble: an operator's turn alone; a
//! working's head with every step under it, together; the model's prose, on
//! `RAISED` where it is the turn's answer and on `SURFACE` where a further
//! working follows it in the same turn or it is still streaming — what
//! follows it is not known yet, so it is not yet the answer. What is said by
//! nobody in particular — the machinery's own word, a peer, a wound — stands
//! on `SURFACE` too: it is read through, never the answer.

use super::{Line, Part, Role, Tone};
use crate::theme::bubble::Bubble;

/// One bubbled block: its role, and how many lines it holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Block {
    pub bubble: Bubble,
    pub lines: usize,
}

/// **The lines grouped into blocks**, in order; the counts sum to the lines.
#[must_use]
pub fn blocks(lines: &[Line]) -> Vec<Block> {
    let mut out = Vec::new();
    let mut at = 0;
    while let Some(line) = lines.get(at) {
        // A head and every step under it are one block; a step is always
        // under a head (`lines` lays no other), so the two parts are one arm.
        let (bubble, held) = match line.part {
            Part::Said => (said(line, lines.get(at + 1..).unwrap_or_default()), 1),
            Part::Head | Part::Step { .. } => {
                let steps = lines
                    .get(at + 1..)
                    .unwrap_or_default()
                    .iter()
                    .take_while(|next| matches!(next.part, Part::Step { .. }))
                    .count();
                (Bubble::Trail, 1 + steps)
            }
        };
        out.push(Block {
            bubble,
            lines: held,
        });
        at += held;
    }
    out
}

/// The bubble under one thing said: the operator's own, else the model's
/// answer where nothing of the turn follows it, else the surface.
fn said(line: &Line, rest: &[Line]) -> Bubble {
    match line.row.role {
        Some(Role::User) => Bubble::Operator,
        Some(Role::Model) if line.row.tone != Tone::Live && !followed(rest) => Bubble::Final,
        _ => Bubble::Intermediate,
    }
}

/// Whether a working follows, among `rest`, before the next turn begins.
fn followed(rest: &[Line]) -> bool {
    rest.iter()
        .take_while(|next| next.row.role != Some(Role::User))
        .any(|next| next.part != Part::Said)
}
