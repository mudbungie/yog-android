//! **What one gesture does** — the command table the loop next door spends,
//! split from it (bl-2f17) when the candidates screen's two commands took
//! `worker.rs` past clippy's line bound. The seam is the one that file already
//! reads as: `run` is the loop and `wait` is its clock, and this is *a command
//! arrived, now what does the seat do about it*.
//!
//! **No act is in this table** (bl-2cad). An act is posted by the poster, on a
//! thread and a connection of its own (`seat::poster`), so that one never
//! waits behind whatever read this loop is inside; what comes back here is
//! `Cmd::Posted`, the fold of what the act earned.
//!
//! Every arm answers the same thing — the sentence the gesture earned, or none
//! — and the two that END the loop stay in `run`, because ending it is the
//! loop's own business and a table cannot say it.

use super::super::cmd::Cmd;
use super::super::pass::Standing;
use super::super::{Focus, asks};
use super::after::{self, preload};
use super::fold::{self, learned, searched};
use crate::transport::Seat;

/// Spend one command. `focus` is `&mut` because two of these MOVE it, which is
/// the one piece of loop state a gesture is allowed to write.
pub(super) fn spend(
    seat: &Seat,
    cmd: Cmd,
    focus: &mut Focus,
    standing: &mut Standing,
) -> Option<String> {
    match cmd {
        Cmd::Aim(moved) => aimed(seat, focus, standing, moved),
        // The three selector gestures. A read's answer is learned as the
        // server's own envelope (bl-0267); a failure is a sentence for the
        // banner exactly as an act's is.
        Cmd::Providers => learned(asks::providers(seat, focus), None, standing),
        Cmd::Catalog => fold::catalogued(asks::catalog(seat, focus), standing),
        Cmd::Models(provider) => {
            let listed = asks::models(seat, focus, &provider);
            learned(listed, Some(provider), standing)
        }
        Cmd::Options => options(seat, focus, standing),
        // **The watch crosses no wire at all** (§13.19) — it is what makes the
        // sign-in's lane wanted, and the next pass is what dials it.
        Cmd::Watch(provider) => {
            standing.signing.watch(provider);
            None
        }
        // **A search needs no read after it either**, and no read before it:
        // it names no place, so nothing about the focus decides what it means.
        // The answer is held by `Standing` and painted onto every snapshot
        // after it, exactly as the deposit counters are.
        Cmd::Search(text) => searched(asks::search(seat, &text), standing),
        // **The world reads a gesture makes** (§13.8, §13.9, §13.11, §13.12;
        // the queue is the lane's, §14.1). Each replaces what it answers and a
        // failure keeps what was there, which is `searched`'s rule and is here
        // for its reason: losing an answer the server gave over one it did not
        // is the defect §13.2's grace exists to prevent.
        Cmd::Ops => fold::opened(asks::ops(seat), standing),
        Cmd::Balls(view) => fold::paned(asks::balls(seat, focus, view), standing),
        Cmd::Records => fold::recorded(asks::opened(seat, focus), standing),
        Cmd::Step(seq) => fold::drilled(asks::drill(seat, focus, seq), standing),
        // **The anchored read** (§13.16) folds into the records like the
        // drill-in beside it.
        Cmd::Anchor(at) => {
            let read = asks::anchored(seat, focus, at.clone());
            fold::anchored(read, at, standing)
        }
        Cmd::Science => fold::spread(asks::science(seat, focus), standing),
        Cmd::Clients => fold::machined(asks::clients(seat, focus), standing),
        Cmd::Offers => after::pending(seat, focus, standing, None),
        // Closing is the whole act (§13.18): the surface lives in the standing.
        Cmd::Forget => {
            standing.offers = None;
            None
        }
        Cmd::Config(at) => fold::configured(asks::config(seat, at), standing),
        Cmd::Marks => fold::marked(asks::marks(seat, focus), standing),
        Cmd::Proposals(id) => fold::staged(asks::proposals(seat, focus, id), standing),
        Cmd::Files(path) => fold::filed(asks::files(seat, focus, path), standing),
        Cmd::Work(file) => fold::worked(asks::work(seat, focus, file), standing),
        // **What an act earned** (`worker::post`, bl-2cad): the poster made
        // the crossing, and the fold — the standing it writes and the read that
        // makes the act visible — is run here, where every read is.
        Cmd::Posted(earned) => earned(seat, standing),
        // The two the loop keeps: `Stop` ends it and a lane's frame is adopted
        // inside `wait`, so neither reaches here — and neither has a sentence.
        Cmd::Stop | Cmd::Lane(_) | Cmd::Refresh => None,
    }
}

/// **Move the focus, and read what the new workspace is set to when it
/// changed** (bl-e9f9). The assignments are a fact about the WORKSPACE, so
/// they are read when the workspace moves and not when the focus merely
/// deepens into a conversation inside it. One body for both moves, because
/// that is the only thing the two arms ever differed in.
fn aimed(seat: &Seat, focus: &mut Focus, standing: &mut Standing, moved: Focus) -> Option<String> {
    let was = focus.workspace.take();
    *focus = moved;
    // The operator has gone somewhere: a start's claim on the focus is over.
    standing.claimed = None;
    // And the poster was told of this move by the same handle (`seat::poster`):
    // counting them is how a start's follow says which move it was made after.
    standing.moves += 1;
    if was != focus.workspace {
        // A sign-in is one workspace's run: the watch outlives the screen
        // (§13.19) and never a move to another workspace, where the same
        // row name is a different pair.
        standing.signing.watch(None);
        preload(seat, focus, standing);
    }
    None
}

/// **The controls row's own pair** (bl-0691), both swallowed like the preload
/// they are: neither is a gesture the operator made, so a server that cannot
/// answer either leaves the row exactly where it stood — dark, and saying so
/// when it is tapped.
fn options(seat: &Seat, focus: &Focus, standing: &mut Standing) -> Option<String> {
    preload(seat, focus, standing);
    if let Ok((workspace, envelope)) = asks::providers(seat, focus) {
        standing.options.learned(&workspace, None, envelope);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{Cmd, Focus, Standing, spend};
    use crate::test_support::{material, mint_ca, mint_leaf, scratch};
    use crate::transport::Seat;

    /// **The two the loop keeps have nothing to say here.** `run` ends on
    /// `Stop` and adopts a lane's frame inside `wait`, so neither ever
    /// reaches this table — but a `match` must be total, and the honest arm
    /// is the one that says there is no sentence rather than a panic this
    /// crate may not write.
    #[test]
    fn the_two_the_loop_keeps_earn_no_sentence() {
        let dir = scratch();
        mint_ca(&dir, "ca");
        mint_leaf(&dir, "ca", "client", false);
        // `Seat::open` dials nothing, so a leaf is the whole of what this
        // needs: the arm under test never touches the wire.
        let seat = Seat::open(&material(&dir, "ca", "client", "127.0.0.1:9")).unwrap();
        let said = spend(
            &seat,
            Cmd::Stop,
            &mut Focus::default(),
            &mut Standing::default(),
        );
        assert!(said.is_none());
    }
}
