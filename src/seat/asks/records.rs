//! **The records screen's reads** (DESIGN §13.11, §13.14): the six asked when
//! the screen opens, and the one posted off a row inside it.
//!
//! **Opening is the ask**, which is the trail's rule and the ball pane's
//! (§13.8, §13.9): a screen nobody has opened costs this device no radio,
//! which is §14.1's argument for the held lane applied at the seat. Six
//! questions per opening and none between.
//!
//! **They answer as one value or as one sentence.** The shapes are questions
//! about ONE conversation and the workspace holding it, and [`crate::codec::Records`] retires
//! them together the moment that conversation moves; a partial fold would put
//! one conversation's spine under another's steps the first time a read
//! failed on its own. So the first failure is the whole gesture's answer, and
//! what was on the glass keeps painting — `searched`'s rule, at a sixth site.

use crate::codec::reply::Reply;
use crate::codec::{Ask, Records, Step};
use crate::seat::Focus;
use crate::seat::pass::{answer, kind_err};
use crate::transport::Seat;

/// The six, in the order the screen paints them.
pub(in crate::seat) fn opened(seat: &Seat, focus: &Focus) -> Result<Records, String> {
    let (workspace, agent) = aimed(focus)?;
    let (ws, id) = (workspace.clone(), agent.clone());
    let head = match answer(
        seat,
        &Ask::Agent {
            workspace: ws,
            agent: id,
        },
    )? {
        (Reply::Agent(head), _) => head,
        (other, _) => return Err(kind_err("agent", &other)),
    };
    let (ws, id) = (workspace.clone(), agent.clone());
    let steps = match answer(
        seat,
        &Ask::Steps {
            workspace: ws,
            agent: id,
        },
    )? {
        (Reply::Steps(steps), _) => steps,
        (other, _) => return Err(kind_err("steps", &other)),
    };
    let (ws, id) = (workspace.clone(), agent.clone());
    let rail = match answer(
        seat,
        &Ask::Rail {
            workspace: ws,
            agent: id,
        },
    )? {
        (Reply::Rail(rail), _) => rail,
        (other, _) => return Err(kind_err("rail", &other)),
    };
    let (ws, id) = (workspace.clone(), agent.clone());
    let governing = match answer(
        seat,
        &Ask::Governing {
            workspace: ws,
            agent: id,
            at: None,
        },
    )? {
        (Reply::Governing(governing), _) => governing,
        (other, _) => return Err(kind_err("governing", &other)),
    };
    let (ws, id) = (workspace.clone(), agent.clone());
    let inbox = match answer(
        seat,
        &Ask::Inbox {
            workspace: ws,
            agent: id,
        },
    )? {
        (Reply::Inbox(rows), _) => rows,
        (other, _) => return Err(kind_err("inbox", &other)),
    };
    // **The sixth names the WORKSPACE rather than the conversation** (§13.14),
    // and it is asked here because what it lists is what the spine half's
    // `follows` is one of.
    let ask = Ask::Lineages {
        workspace: workspace.clone(),
    };
    let lineages = match answer(seat, &ask)? {
        (Reply::Lineages(rows), _) => rows,
        (other, _) => return Err(kind_err("lineages", &other)),
    };
    Ok(Records {
        workspace,
        agent,
        head,
        steps,
        rail,
        governing,
        inbox,
        lineages,
        drilled: None,
        anchored: None,
    })
}

/// **Which config governs a picked FORK POINT** (DESIGN §13.16) — the
/// anchored form of the read the opening already makes, posted off a picked
/// notch for `drill`'s reason exactly: it is about one point rather than about
/// the conversation, so nothing standing carries it.
///
/// The answer echoes no commit, so the caller pairs it with the one it asked
/// at; that pairing is the value's, not this function's.
pub(in crate::seat) fn anchored(
    seat: &Seat,
    focus: &Focus,
    at: String,
) -> Result<crate::codec::Governing, String> {
    let (workspace, agent) = aimed(focus)?;
    let ask = Ask::Governing {
        workspace,
        agent,
        at: Some(at),
    };
    match answer(seat, &ask)? {
        (Reply::Governing(governing), _) => Ok(governing),
        (other, _) => Err(kind_err("governing", &other)),
    }
}

/// **One step's records**, addressed by the sequence the census stated. It is
/// posted rather than standing for the reason the screen's own doc gives: a
/// standing read of one step would have to invent a selection and then hold
/// it, which is a second authority for a row somebody tapped.
pub(in crate::seat) fn drill(seat: &Seat, focus: &Focus, seq: String) -> Result<Step, String> {
    let (workspace, agent) = aimed(focus)?;
    let ask = Ask::Step {
        workspace,
        agent,
        seq,
    };
    match answer(seat, &ask)? {
        (Reply::Step(step), _) => Ok(step),
        (other, _) => Err(kind_err("step", &other)),
    }
}

/// The focused conversation, or the sentence saying there is none. These
/// reads are about a conversation, so there is nothing to ask without one —
/// the same refusal `asks::balls` makes of its aimed view.
fn aimed(focus: &Focus) -> Result<(String, String), String> {
    let Focus {
        workspace: Some(workspace),
        agent: Some(agent),
    } = focus.clone()
    else {
        return Err("records: no conversation is focused".to_owned());
    };
    Ok((workspace, agent))
}
