//! **The reads a gesture asks for** — the selectors' three (bl-0267, bl-e9f9),
//! the search, the trail. A *pass* is the standing set on the model's own
//! clock (`seat::pass`); these are asked when the operator opens something.
//! The two follow-class reads are neither: they are held (`seat::lane`,
//! DESIGN §14.1), and a one-shot ask of a held lane would wait a hold.
//!
//! **Split from `seat::acts` on the contract's own line** (bl-07b1). yog
//! REMOTE §3: *"Asks are the opposite case and re-ask freely: a read is
//! answered in place, and asking twice is asking once (§9.7)."* Everything in
//! this file may be re-asked by the next tick with nothing remembered and
//! nothing at risk, which is exactly why none of it carries the in-doubt
//! machinery its neighbour does — and why a read's failure is one sentence for
//! the banner and no more.

use serde_json::Value;

use super::Focus;
use super::pass::{answer, kind_err};
use crate::codec::reply::Reply;
use crate::codec::{Ask, Found, OpRow};
use crate::transport::Seat;

/// **How much trail a phone asks for** (DESIGN §13.8). The engine's own tail
/// bound is larger; this is a screenful and a scroll, chosen here because the
/// ask carries the number and the asker is the one that knows what it can
/// paint. A larger window costs a radio a bigger answer for rows below the
/// fold, on the device the §14.1 lane exists to keep asleep.
const TAIL: usize = 64;

/// **What the focused workspace's roles are set to** (bl-e9f9), handed back
/// as the engine's own envelope like the two option reads beside it.
pub(super) fn roles(seat: &Seat, focus: &Focus) -> Result<(String, Value), String> {
    let workspace = super::acts::focused(focus)?;
    let ask = Ask::Roles {
        workspace: workspace.clone(),
    };
    match answer(seat, &ask)? {
        (Reply::Roles(_), envelope) => Ok((workspace, envelope)),
        (other, _) => Err(kind_err("roles", &other)),
    }
}

/// **The focused workspace's providers** (bl-0267), handed back as the
/// engine's own envelope: what the selectors hold is what the engine said,
/// and the cache stores exactly that (§14).
pub(super) fn providers(seat: &Seat, focus: &Focus) -> Result<(String, Value), String> {
    let workspace = super::acts::focused(focus)?;
    let ask = Ask::Providers {
        workspace: workspace.clone(),
    };
    match answer(seat, &ask)? {
        (Reply::Providers(_), envelope) => Ok((workspace, envelope)),
        (other, _) => Err(kind_err("providers", &other)),
    }
}

/// One provider's models, the same way.
pub(super) fn models(
    seat: &Seat,
    focus: &Focus,
    provider: &str,
) -> Result<(String, Value), String> {
    let workspace = super::acts::focused(focus)?;
    let ask = Ask::Models {
        workspace: workspace.clone(),
        provider: provider.to_owned(),
    };
    match answer(seat, &ask)? {
        (Reply::Models(_), envelope) => Ok((workspace, envelope)),
        (other, _) => Err(kind_err("models", &other)),
    }
}

/// **What the needle found** (yog DESIGN §8.5, bl-4c2b) — the one read this
/// seat makes that names no place, so it takes no focus.
///
/// **An empty needle is no search, on both sides of the wire.** Before it: a
/// cleared field is answered here, because the answer being dropped is this
/// seat's own copy and an operator must be able to leave a search with the
/// engine unreachable. After it: the engine's own spelling of *no search* is
/// an answer whose needle is empty, and it means the same thing. One rule,
/// stated once, and nothing downstream has to tell a cleared search from a
/// search that was never made.
pub(super) fn search(seat: &Seat, text: &str) -> Result<Option<Found>, String> {
    if text.trim().is_empty() {
        return Ok(None);
    }
    let ask = Ask::Search {
        text: text.to_owned(),
    };
    match answer(seat, &ask)? {
        (Reply::Search(found), _) => Ok((!found.needle.is_empty()).then_some(found)),
        (other, _) => Err(kind_err("search", &other)),
    }
}

/// **The ops trail's tail** (yog §4.2, DESIGN §13.8) — the read that names no
/// place, like the search beside it: the trail is the engine's, not a
/// workspace's, so nothing about the focus decides what it says.
pub(super) fn ops(seat: &Seat) -> Result<Vec<OpRow>, String> {
    match answer(seat, &Ask::Ops { max: TAIL })? {
        (Reply::Ops(rows), _) => Ok(rows),
        (other, _) => Err(kind_err("ops", &other)),
    }
}
