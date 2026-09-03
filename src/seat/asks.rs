//! **The reads a gesture asks for** — the selectors' three (bl-0267, bl-e9f9)
//! and the live tail (bl-4822). A *pass* is the standing set on the model's
//! own clock (`seat::pass`); these are asked when the operator opens something
//! or when a conversation is writing.
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
use crate::codec::Ask;
use crate::codec::reply::Reply;
use crate::transport::Seat;

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

/// **The answer in flight** (REMOTE §5.5, bl-4822), one shot: §5.5 says a
/// read starts holding nothing and *"the first frame of any read is the whole
/// tail so far"*, so what comes back is the answer as it stands and this seat
/// replaces rather than appends. The append fold belongs to a seat that HOLDS
/// the connection, and this one does not (DESIGN §7).
pub(super) fn follow(seat: &Seat, focus: &Focus) -> Result<crate::codec::Stream, String> {
    let Focus {
        workspace: Some(workspace),
        agent: Some(agent),
    } = focus.clone()
    else {
        return Err("follow: no conversation is focused".to_owned());
    };
    let ask = Ask::Follow { workspace, agent };
    match answer(seat, &ask)? {
        (Reply::Follow(stream), _) => Ok(stream),
        (other, _) => Err(kind_err("follow", &other)),
    }
}
