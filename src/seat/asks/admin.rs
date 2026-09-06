//! **The admin surface's two reads** (DESIGN §13.17): what one config file
//! holds, and which task branch a workspace is marked with.
//!
//! **Each is its write's own op token with the written half left out**, which
//! is the engine's grammar rather than a convention here — so the two halves
//! of one op are an ask and an act on this side, and `codec::request` splits
//! its tables on exactly that field.
//!
//! **Neither answer echoes what it was asked about**, so the ask names it: a
//! config reply carries bytes and no destination, a marks reply a branch and
//! no workspace. That is `asks::review`'s rule at a third site, and it is what
//! lets a screen paint a file under the destination it asked for.

use crate::codec::reply::Reply;
use crate::codec::{Ask, Config, Destination, Marks};
use crate::seat::Focus;
use crate::seat::pass::{answer, kind_err};
use crate::transport::Seat;

/// **What one config file holds.** The destination is the gesture's own — two
/// of the three name no workspace at all — so nothing about the focus decides
/// what is read.
pub(in crate::seat) fn config(seat: &Seat, at: Destination) -> Result<Config, String> {
    let ask = Ask::Config { at: at.clone() };
    match answer(seat, &ask)? {
        (Reply::Config(text), _) => Ok(Config { at, text }),
        (other, _) => Err(kind_err("config", &other)),
    }
}

/// **Which task branch this workspace is marked with.** Aimed like the ball
/// pane's own aimed view, and for its reason: a mark is per workspace, so one
/// under another workspace's name would be the wrong claim.
pub(in crate::seat) fn marks(seat: &Seat, focus: &Focus) -> Result<Marks, String> {
    let workspace = super::super::acts::focused(focus)?;
    let ask = Ask::Marks {
        workspace: workspace.clone(),
    };
    match answer(seat, &ask)? {
        (Reply::Marks(branch), _) => Ok(Marks { workspace, branch }),
        (other, _) => Err(kind_err("marks", &other)),
    }
}
