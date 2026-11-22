//! **The poster** (bl-2cad): the thread that posts what the operator says, on
//! a connection of its own, the moment it is said.
//!
//! **Why a thread of its own.** Things happen on the server; this seat reads
//! its replica and may say something at any moment, and on the wire it is free
//! to do both at once — only a tool host's `invocations` read is exclusive
//! (yog REMOTE §5.1). Until this file every act was an arm of the worker's
//! command table, so a send waited for whatever read the worker was inside: a
//! pass of up to four reads, a lane's dial, the five reads the records screen
//! opens with, the read that follows another act. On a slow network that is
//! the connect bound, several times over, between the tap and the message
//! leaving. This is the desktop seat's own shape (lernie DESIGN, the poster
//! beside the asker), for its reason: *an act must not wait behind a read that
//! is mid-pass*.
//!
//! **Acts are serial among themselves, and that is deliberate.** The operator
//! said them in an order, and a start is two gestures that must stay adjacent
//! (§8.1). What they are not serial with is any read.
//!
//! **What an act earned goes back to the worker as a command**
//! ([`Cmd::Posted`]). The worker owns the standing and makes every read, so
//! the fold — and the read that makes an act visible — is run there
//! (`worker::post`). Nothing is shared and no lock is added: the two threads
//! meet at two channels.
//!
//! **Where an act is aimed is this thread's own copy of the focus, and the
//! handle is what keeps it right.** The frame's handle sends every focus
//! gesture to both threads — here first — so *tap a conversation, then send*
//! lands where the tap aimed even while the worker is still reading the
//! conversation before it. One move is not the operator's: a start is followed
//! into the conversation it founded (§25), and that is the worker's, made when
//! the list first shows the row. [`Post::Founded`] tells this thread, with the
//! count of the operator's moves it was made after; a count that is behind
//! this thread's own means the operator has moved since, the worker is about
//! to hear of it, and the follow is dropped here exactly as that move will
//! overwrite it there.

use std::sync::mpsc;

use super::Focus;
use super::cmd::Cmd;
use super::say::Say;
use crate::transport::Seat;

/// What the poster is told.
pub(super) enum Post {
    /// The operator aimed the focus — the handle's, sent beside `Cmd::Aim`.
    Aim(Focus),
    /// A start's follow moved it — the worker's, made after this many of the
    /// operator's moves.
    Founded(u64, Focus),
    /// Post this act, now, at the focus as it stands.
    Say(Say),
    /// The handle is going away.
    Stop,
}

/// Where the next act is addressed, and how many of the operator's moves got
/// it there.
#[derive(Default)]
struct Aim {
    focus: Focus,
    moves: u64,
}

impl Aim {
    /// The operator moved: that is where the next act goes.
    fn moved(&mut self, focus: Focus) {
        self.focus = focus;
        self.moves += 1;
    }

    /// A start's follow, made after `after` of the operator's moves. One the
    /// operator has moved past is not taken.
    fn founded(&mut self, after: u64, focus: Focus) {
        if after == self.moves {
            self.focus = focus;
        }
    }
}

/// Post every act as it arrives, and hand what each earned to the worker.
/// `focus` is where the seat resumed — the cache's, as the worker's own is.
pub(super) fn run(
    seat: &Seat,
    focus: Focus,
    posts: &mpsc::Receiver<Post>,
    back: &mpsc::Sender<Cmd>,
) {
    let mut aim = Aim { focus, moves: 0 };
    loop {
        match posts.recv() {
            // The handle sends `Stop` before it can go away, and the worker
            // holds a sender of its own, so a dead channel is not how this
            // ends — but it would be an ending, and it is the same one.
            Ok(Post::Stop) | Err(mpsc::RecvError) => return,
            Ok(Post::Aim(focus)) => aim.moved(focus),
            Ok(Post::Founded(after, focus)) => aim.founded(after, focus),
            // A receipt nobody is left to fold is a worker that has stopped,
            // which only the handle going away does: the act still crossed.
            Ok(Post::Say(say)) => {
                let _ = back.send(Cmd::Posted(super::worker::post(seat, &aim.focus, say)));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Aim, Focus};

    fn at(agent: &str) -> Focus {
        Focus {
            workspace: Some("home".to_owned()),
            agent: Some(agent.to_owned()),
        }
    }

    /// **A start's follow is taken only while the operator has not moved since
    /// it was made.** The worker makes it after the moves it has spent; a
    /// count behind this thread's own is a tap the worker has not reached yet,
    /// and that tap is where both threads end up — including a tap on the very
    /// place the operator already stood, which an equality on the focus could
    /// not tell from no tap at all.
    #[test]
    fn a_follow_the_operator_has_moved_past_is_not_taken() {
        let mut aim = Aim::default();
        aim.moved(at("a1"));
        aim.founded(1, at("founded"));
        assert_eq!(aim.focus, at("founded"), "nothing moved since: followed");
        aim.moved(at("a1"));
        aim.founded(1, at("late"));
        assert_eq!(
            aim.focus,
            at("a1"),
            "the operator moved since: theirs stands"
        );
        aim.founded(2, at("founded"));
        assert_eq!(aim.focus, at("founded"));
    }
}
