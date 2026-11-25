//! **What the frame can say to the worker** — the command vocabulary the
//! model's channel carries, split from the handle that sends it (bl-f36e)
//! when the ball pane's act took `model.rs` to the 300 wall. The seam is the
//! one `codec.rs` and `codec/encode.rs` already draw one layer down: what a
//! gesture IS, and the thing that spells it. `seat::worker` is what spends
//! every variant here; `seat::model` is what posts them.
//!
//! **The acts are not here** (bl-2cad). What the frame SAYS goes down a
//! channel of its own to the poster and is spelled in `seat::say`; this is the
//! reading side — what the frame asks for, where it aims, and the two things
//! other threads hand the worker: a lane's frame and an act's receipt.

pub(super) enum Cmd {
    /// **Aim the focus** — at a workspace, at one conversation inside it, or
    /// (both empty) back out to the roster. One command for the two handles
    /// that move it, because the focus is built once, by the handle, and
    /// sent to the poster too (`seat::poster`): the worker and the poster
    /// must be told the same place in the same order.
    Aim(super::Focus),
    /// **List the focused workspace's providers** (bl-0267) — a gesture of
    /// the selectors' own, asked when one is opened rather than on every
    /// pass: a pass is the standing set, and these are options.
    Providers,
    /// **List what the focused workspace can sign in to** (bl-892e) — the
    /// sign-in screen's opening ask. Its own command because it is its own
    /// question: the table is what a run routes to, the catalog is what a
    /// sign-in may enter into it.
    Catalog,
    /// List one provider's models.
    Models(String),
    /// **What the controls row is gated on** (bl-0691): the workspace's role
    /// assignments and its provider rows, read together because the row
    /// spends them together — which provider the worker is on, and what that
    /// provider's own row says it takes. Its own command rather than two,
    /// because they are one question asked once per workspace by the surface
    /// that needs the answer, and a surface asking twice for one thing is
    /// where a second reading comes from.
    Options,
    /// **Ask nothing but the pass that follows every command** — the
    /// workspace dropup's reload (§22 item 2): the workspaces are a standing
    /// read, so re-reading them is a pass, sooner.
    Refresh,
    /// **Follow one provider's sign-in, or stop** (§13.19). The lane's want,
    /// and the empty case is the second: it crosses no wire, so a tail can be
    /// closed with the server unreachable — `Cmd::Search`'s own shape at the
    /// other gesture that turns something off. Leaving the screen is not
    /// such a close (bl-95f2): a waiting run outlives its screen.
    Watch(Option<String>),
    /// **Search the world, or drop the answer being shown** (bl-4c2b). The
    /// empty needle is the second: it crosses no wire, so a search can be
    /// left with the server unreachable.
    Search(String),
    /// **Read the ops trail** (§13.8) — what the server last did. A gesture
    /// read and not a standing one: nothing paints it unless the surface is
    /// open, and a phone's radio is not free.
    Ops,
    /// **A held lane's frame, or its end** (§14.1) — the attention queue's
    /// or the live tail's. A command like any gesture, because the worker
    /// adopts it where it adopts everything else and no lock is needed for
    /// a thread to hand it over.
    Lane(super::lane::Framed),
    /// **Read what a reviewer has staged** (§13.17, REMOTE §9.22) — the
    /// listing, or, with an id, that proposal whole beside it. A gesture read
    /// on the trail's terms: nothing paints it unless the admin screen is
    /// open.
    Proposals(Option<String>),
    /// **Read the ball pane at one of its three views** (§13.9). One command
    /// because the pane holds one answer: which view is open is the shell's,
    /// and which read answered is the pane's own.
    Balls(crate::codec::View),
    /// **Read the conversation's machinery** (§13.11) — the five reads the
    /// records screen opens with. One command because they are one value: six
    /// questions about one conversation, retired together when it moves.
    Records,
    /// **Drill into one step** of it, by the sequence the census stated. Its
    /// own command because it is the one read of the six that is about a ROW
    /// rather than about the conversation.
    Step(String),
    /// **Which config governs a picked fork point** (§13.16) — the anchored
    /// form of a read the records opening already makes. Its own command for
    /// `Step`'s reason: it is about one POINT rather than about the
    /// conversation, so nothing standing carries it.
    Anchor(String),
    /// **Read the focused workspace's pending offers** (§13.18) — what opens
    /// the offers surface, and what it re-asks while it stands.
    Offers,
    /// **Close the offers surface** (§13.18): drop what it held. It crosses
    /// no wire.
    Forget,
    /// **Read one config file** (§13.17). The destination is the gesture's
    /// own — two of the three name no workspace — so nothing about the focus
    /// decides what is read.
    Config(crate::codec::Destination),
    /// **Read which task branch this workspace is marked with** (§13.17).
    Marks,
    /// **Read this workspace's attempts** (§13.12) — what the candidates
    /// screen opens with.
    Science,
    /// **Read the focused conversation's worktree** (§13.15) — the listing,
    /// or one file's bytes when a path is carried. One command for both
    /// because they are one question at two depths, and the deeper answer
    /// carries the shallower one whole.
    Files(Option<String>),
    /// **Read what this workspace's attempts changed** (§13.15) — the same
    /// pair one subject along: the listing, or one changed file's patch.
    Work(Option<crate::codec::WorkFile>),
    /// **Read which machines may execute for this workspace** (§13.14) —
    /// what the roster screen opens with, and what it re-asks while it is
    /// open, because presence is true only at the instant it was answered.
    Clients,
    /// **What an act earned, handed back by the poster** (`seat::poster`,
    /// bl-2cad): the fold that writes it onto the standing, and the read that
    /// makes it visible. A command like a lane's frame, and for the lane's
    /// reason — the worker adopts it where it adopts everything else.
    Posted(super::worker::Fold),
    Stop,
}
