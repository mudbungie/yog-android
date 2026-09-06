//! **The handles that ASK** — every command the frame sends that populates a
//! surface, split from the handle itself (bl-5a56) when the work-review pair
//! took `model.rs` to the 300 wall.
//!
//! The seam is the one this crate draws at four other sites and calls by the
//! same name: the boundary's grammar is asks and acts (`codec::Gesture`), the
//! encoder splits `asked` from `acted`, the decoder reads with two tables, and
//! the worker's own `seat::asks` and `seat::acts` are the pair one layer down.
//! A table that reads a place, and a table that names a change.
//!
//! **Every one of these is safe to send twice** (REMOTE §3, §9.7): a read is
//! answered in place, so nothing here carries the in-doubt machinery its
//! neighbour does, and a failed one is a sentence for the banner and no more.

use super::Model;
use crate::seat::cmd::Cmd;

impl Model {
    /// Focus a workspace (its conversation list joins the standing set), or
    /// `None` to back out to the workspace roster.
    pub fn focus_workspace(&self, workspace: Option<String>) {
        let _ = self.cmds.send(Cmd::Workspace(workspace));
    }

    /// Focus one conversation: its transcript joins the standing set.
    pub fn focus_conversation(&self, workspace: String, agent: String) {
        let _ = self.cmds.send(Cmd::Conversation(workspace, agent));
    }

    /// **Ask for the focused workspace's providers** (bl-0267). The answer
    /// arrives in the next snapshot, and what was already known keeps
    /// painting meanwhile — the selectors open on the cache and correct
    /// themselves a round trip later.
    pub fn list_providers(&self) {
        let _ = self.cmds.send(Cmd::Providers);
    }

    /// Ask for one provider's models.
    pub fn list_models(&self, provider: String) {
        let _ = self.cmds.send(Cmd::Models(provider));
    }

    /// **Follow one provider's sign-in** (§13.19), or — with `None` — stop
    /// following one. The lane is opened by the next pass and its frames
    /// arrive in the snapshots after it; `None` crosses no wire at all, so a
    /// tail can be closed with the engine unreachable.
    pub fn watch_login(&self, provider: Option<String>) {
        let _ = self.cmds.send(Cmd::Watch(provider));
    }

    /// **Search everything this seat can see** (yog DESIGN §8.5) for `text`,
    /// or — with an empty needle — drop the answer that is standing. The hits
    /// arrive in the next snapshot like every other read's rows.
    pub fn search(&self, text: String) {
        let _ = self.cmds.send(Cmd::Search(text));
    }

    /// **Ask for the ops trail** (§13.8). The rows arrive in the next
    /// snapshot like every other read's, and what was already there keeps
    /// painting meanwhile.
    pub fn list_trail(&self) {
        let _ = self.cmds.send(Cmd::Ops);
    }

    /// **Read the ball pane** (§13.9) at `view`. The rows arrive in the next
    /// snapshot like every other read's, and what was already there keeps
    /// painting meanwhile — under its own view, never under this one.
    pub fn list_balls(&self, view: crate::codec::View) {
        let _ = self.cmds.send(Cmd::Balls(view));
    }

    /// **Read the conversation's machinery** (§13.11) — what the records
    /// screen opens with. The answers arrive in the next snapshot like every
    /// other read's, and what was already there keeps painting meanwhile:
    /// under its own conversation, never under this one.
    pub fn open_records(&self) {
        let _ = self.cmds.send(Cmd::Records);
    }

    /// **Read one step's records** (§13.11), by the sequence the census
    /// stated. The answer carries that sequence back, so nothing here has to
    /// remember which row was tapped.
    pub fn drill_step(&self, seq: String) {
        let _ = self.cmds.send(Cmd::Step(seq));
    }

    /// **Read the focused workspace's attempts** (§13.12). The rows arrive in
    /// the next snapshot like every other read's, and what was already there
    /// keeps painting meanwhile — under its own workspace, never under this
    /// one.
    pub fn list_candidates(&self) {
        let _ = self.cmds.send(Cmd::Science);
    }

    /// **Read this workspace's machines** (§13.14). The rows arrive in the
    /// next snapshot like every other read's, and what was already there keeps
    /// painting meanwhile — under its own workspace, never under this one.
    pub fn list_clients(&self) {
        let _ = self.cmds.send(Cmd::Clients);
    }
    /// **Read which config governs a picked fork point** (§13.16). The answer
    /// folds into the records it belongs to, carrying the commit it was asked
    /// at, so a policy cannot paint under a notch tapped since.
    pub fn anchor(&self, at: String) {
        let _ = self.cmds.send(Cmd::Anchor(at));
    }

    /// **Read one config file** (§13.17). The destination is the gesture's
    /// own; the answer comes back carrying it, so a file cannot paint under
    /// another destination's name.
    pub fn read_config(&self, at: crate::codec::Destination) {
        let _ = self.cmds.send(Cmd::Config(at));
    }

    /// **Read which task branch the focused workspace is marked with**
    /// (§13.17).
    pub fn read_marks(&self) {
        let _ = self.cmds.send(Cmd::Marks);
    }

    /// **Read the focused conversation's worktree** (§13.15) — the listing
    /// with no path, and one file's bytes with one. The answer replaces what
    /// was held whole, carrying the path it was asked at, so a preview cannot
    /// paint under a row the operator tapped since.
    pub fn open_files(&self, path: Option<String>) {
        let _ = self.cmds.send(Cmd::Files(path));
    }

    /// **Read what the focused workspace's attempts changed** (§13.15) — the
    /// listing with no file, and one changed file's bounded patch with one.
    /// `open_files`' shape, one subject along and for its reason.
    pub fn open_work(&self, file: Option<crate::codec::WorkFile>) {
        let _ = self.cmds.send(Cmd::Work(file));
    }
}
