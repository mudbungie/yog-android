//! **The work-review reads** (DESIGN §13.15): what a conversation's worktree
//! holds, and what a workspace's attempts changed.
//!
//! **Each is one question asked at two depths**, and the deeper one answers
//! the shallower one again: an ask naming a path answers the listing AND the
//! file's bytes, so a preview never has to be merged into a listing it was not
//! read beside. What is held is replaced whole, which is `paned`'s rule with
//! nothing extra to remember.
//!
//! **The answer does not echo what was asked for.** A `files` reply carries a
//! preview and no path; a `work-diff` reply carries a patch and no address.
//! So the ask's own parameter is carried into the value here — the one place
//! it is known — and the paint puts the bytes under exactly that row. That is
//! the `step` drill-in's guarantee (§13.11) bought at the fold rather than by
//! the wire.

use crate::codec::reply::Reply;
use crate::codec::{Ask, Files, Work, WorkFile};
use crate::seat::Focus;
use crate::seat::pass::{answer, kind_err};
use crate::transport::Seat;

/// **The agent worktree**, listed — and one file's bytes when `path` names
/// one. It is about a conversation, so there is nothing to ask without one:
/// the same refusal `asks::records` makes of its own aimed reads.
pub(in crate::seat) fn files(
    seat: &Seat,
    focus: &Focus,
    path: Option<String>,
) -> Result<Files, String> {
    let Focus {
        workspace: Some(workspace),
        agent: Some(agent),
    } = focus.clone()
    else {
        return Err("files: no conversation is focused".to_owned());
    };
    let ask = Ask::Files {
        workspace: workspace.clone(),
        agent: agent.clone(),
        path: path.clone(),
    };
    match answer(seat, &ask)? {
        (Reply::Files(listing), _) => Ok(Files {
            workspace,
            agent,
            listing,
            opened: path.unwrap_or_default(),
        }),
        (other, _) => Err(kind_err("files", &other)),
    }
}

/// **What this workspace's attempts changed** — and one file's patch when
/// `file` names one. Aimed like the candidates listing beside it, and for its
/// reason: a workspace's churn under another workspace's name would be the
/// wrong claim, so there is nothing to ask with no workspace focused.
pub(in crate::seat) fn work(
    seat: &Seat,
    focus: &Focus,
    file: Option<WorkFile>,
) -> Result<Work, String> {
    let workspace = super::super::acts::focused(focus)?;
    let ask = Ask::WorkDiff {
        workspace: workspace.clone(),
        file: file.clone(),
    };
    match answer(seat, &ask)? {
        (Reply::WorkDiff(churned), _) => Ok(Work {
            workspace,
            rows: churned.rows,
            patch: churned.patch,
            opened: file,
        }),
        (other, _) => Err(kind_err("work-diff", &other)),
    }
}
