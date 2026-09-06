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

mod records;
mod review;

pub(super) use records::{drill, opened};
pub(super) use review::{files, work};

use super::Focus;
use super::pass::{answer, kind_err};
use crate::codec::reply::Reply;
use crate::codec::{Ask, Found, Machines, OpRow, Pane, Spread, View};
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

/// **The ball pane's three reads, as one ask** (DESIGN §13.9). One function
/// because the view is the only thing that differs and the pane holds exactly
/// one answer: which read was made is what [`Pane`] carries back, so nothing
/// downstream has to remember what was asked for.
///
/// Only the middle one names a place, and it takes the focus the same way the
/// selectors do — a workspace's balls under another workspace's name would be
/// the wrong claim, so there is nothing to ask with no workspace focused.
pub(super) fn balls(seat: &Seat, focus: &Focus, view: View) -> Result<Pane, String> {
    let ask = match view {
        View::Everywhere => Ask::Balls,
        View::Board => Ask::Board,
        View::Here => Ask::WorkspaceBalls {
            workspace: super::acts::focused(focus)?,
        },
    };
    match answer(seat, &ask)? {
        (Reply::Balls(rows), _) => Ok(Pane::Everywhere(rows)),
        (Reply::WorkspaceBalls(rows), _) => Ok(Pane::Here(rows)),
        (Reply::Board(board), _) => Ok(Pane::Board(board)),
        (other, _) => Err(kind_err(view.screen(), &other)),
    }
}

/// **What each attempt cost** (DESIGN §13.12) — the aimed read the candidates
/// screen opens with, focused the way the ball pane's own aimed view is: a
/// workspace's attempts under another workspace's name would be the wrong
/// claim, so there is nothing to ask with no workspace focused.
///
/// It is derived when asked — nothing behind it is stored — so the same row a
/// minute later is a statement about the world a minute later, and opening the
/// surface is what asks for it.
pub(super) fn science(seat: &Seat, focus: &Focus) -> Result<Spread, String> {
    let workspace = super::acts::focused(focus)?;
    let ask = Ask::Science {
        workspace: workspace.clone(),
    };
    match answer(seat, &ask)? {
        (Reply::Science(rows), _) => Ok(Spread { workspace, rows }),
        (other, _) => Err(kind_err("science", &other)),
    }
}

/// **Which machines may execute for this workspace** (REMOTE §5, §5.1; DESIGN
/// §13.14) — aimed like the ball pane's own aimed view, and for its reason: a
/// registration is per workspace, so a roster under another workspace's name
/// would be the wrong claim.
///
/// **It is re-asked while the screen is open** rather than posted once, which
/// is the one read here that MUST be: `present` is true only at the instant it
/// was answered, so a row saying a machine is connected is worth nothing
/// unless it is asked again (lernie DESIGN §4.28).
pub(super) fn clients(seat: &Seat, focus: &Focus) -> Result<Machines, String> {
    let workspace = super::acts::focused(focus)?;
    let ask = Ask::Clients {
        workspace: workspace.clone(),
    };
    match answer(seat, &ask)? {
        (Reply::Clients(rows), _) => Ok(Machines { workspace, rows }),
        (other, _) => Err(kind_err("clients", &other)),
    }
}
