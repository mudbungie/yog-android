//! **The overlay** (DESIGN §22 item 6, bl-3f9a): every surface that used to
//! cover the chat with its own bar, as one value — which of them stands, what
//! its name line says, and what closing it yields. Pure and host-tested; the
//! shell's `overlay.rs` paints the frame these decisions describe.
//!
//! **One stands at a time.** The shell held three navigation facts for this —
//! which world surface was open, whether the records screen was, whether the
//! files screen was — and nothing stopped two of them being true at once. They
//! fold into one `Option<Overlay>`, the desktop's rule (lernie DESIGN §4.42)
//! and a subtraction here.
//!
//! **The search hits are an overlay whose standing is the server's answer**,
//! not a held value: the hits stand while an answer does, at the depth the
//! search was asked from (§13.6). So `Search` is never held; [`standing`] is
//! where that is decided, once.

use crate::codec::View;

/// Which surface floats over the chat. Navigation and nothing else — no more
/// durable than a scroll position: what this device IS is derived from the
/// leaf on disk, and what is on the glass is stored nowhere else.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    /// Everything waiting on the operator, across every workspace (§13.8).
    Queue,
    /// What the server last did (§13.8).
    Trail,
    /// What each attempt on this workspace's obligations cost (§13.12).
    Candidates,
    /// The drone loop and the alignment monitor (§13.13).
    Fleet,
    /// Which machines may execute for this workspace (§13.14).
    Clients,
    /// What this workspace's attempts changed (§13.15).
    Work,
    /// The config files, the task branch, the inbox flush, the unmaking
    /// (§13.17).
    Admin,
    /// This workspace's provider rows and the act that signs one in (§13.19).
    SignIn,
    /// The op table (§13.14).
    Help,
    /// The ball pane at one of its three views (§13.9).
    Balls(View),
    /// The conversation's machinery (§13.11).
    Records,
    /// The conversation's worktree (§13.15).
    Files,
    /// The hits a standing search answer carries (§13.6).
    Search,
    /// **The switcher** (§22 item 7): the workspaces and the focused one's
    /// conversations, fused — what back from the chat opens, and what stands
    /// whenever nothing is focused.
    Switcher,
    /// **This device's own knobs** (§23 item 11), one row each.
    Settings,
}

/// How deep the focus is — what the overlay floats over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Depth {
    /// No workspace focused: the switcher stands over it.
    Roster,
    /// A workspace and no conversation: the chat in start mode.
    Workspace,
    /// A conversation: the chat.
    Conversation,
}

/// The name minted material stands under (§13.18): it covers the admin
/// surface, and closing it returns there.
pub const MINTED: &str = "minted";

/// **The word at the name line's far end** (bl-dfad): what a tap on the line
/// does. Not `close`, because `close` is a ball act's wire word and its
/// label (PARITY §4's one name) — the ball pane stood with two of them.
pub const DONE: &str = "done";

/// What a close does. Three answers because three things can be standing
/// under one name line, and only one of them is navigation alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Close {
    /// Put the overlay down; the picks it held go with it.
    Leave,
    /// Drop the minted material and stay on the admin surface (§13.18): the
    /// act that surface exists to end with, and the screen it covered.
    Forget,
    /// Drop the search answer and its question: leaving a search leaves it.
    Unsearch,
    /// Leave the app: the switcher over nothing has nothing under it, so
    /// closing it is what back does there (§22 item 7).
    Exit,
}

/// The first line of the frame, in words: the overlay's name, what it is open
/// on, the read closing it reaches (the parity tag the line carries —
/// PARITY §2), and what closing does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    /// The screen's name — the probe's and the harness's (§15.2).
    pub name: &'static str,
    /// The words the line says: the phone's own (§13.3), not an op token.
    pub title: &'static str,
    pub subject: String,
    pub reaches: &'static str,
    pub close: Close,
    /// Whether the platform's back closes it. Every overlay but the switcher:
    /// back there leaves the app, because it is where every back lands.
    pub back: bool,
}

/// What a frame's subject names, read off the focus.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Under {
    pub workspace: String,
    pub conversation: String,
    pub needle: String,
}

impl Overlay {
    /// The name the line says, which is also the screen's name and the
    /// harness's tap target where it has one (§15.2) — one spelling.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Queue => "waiting",
            Self::Trail => "trail",
            Self::Candidates => "science",
            Self::Fleet => "fleet",
            Self::Clients => "clients",
            Self::Work => "work-diff",
            Self::Admin => "config",
            Self::SignIn => "login",
            Self::Help => "help",
            Self::Balls(view) => view.screen(),
            Self::Records => "records",
            Self::Files => "files",
            Self::Search => "search",
            Self::Switcher => "conversations",
            Self::Settings => "settings",
        }
    }

    /// **The words the name line says** (bl-dfad): the verb sheet's word for
    /// it (§22 item 5) where the screen's name is a wire op's token.
    pub(crate) const fn title(self) -> &'static str {
        match self {
            Self::Work => "diff",
            Self::Balls(View::Everywhere | View::Here) => "balls",
            _ => self.name(),
        }
    }

    /// Which read the screen under it is populated by — the op the close line
    /// is the owed interactable for (PARITY §2), as the bar's back control
    /// was.
    pub(crate) const fn reaches(self) -> &'static str {
        match self {
            Self::Queue
            | Self::Trail
            | Self::Help
            | Self::Balls(_)
            | Self::Search
            | Self::Switcher
            | Self::Settings => "workspaces",
            Self::Records | Self::Files => "transcript",
            Self::Candidates
            | Self::Fleet
            | Self::Clients
            | Self::Work
            | Self::Admin
            | Self::SignIn => "conversations",
        }
    }

    /// Whether it is a depth of one conversation, and so stands only while
    /// that conversation is the focus.
    const fn of_conversation(self) -> bool {
        matches!(self, Self::Records | Self::Files)
    }

    /// The name line. `minted` is whether minted material stands (§13.18): it
    /// covers the admin surface under its own name, and closing it forgets
    /// the material rather than leaving the surface.
    pub fn line(self, minted: bool, under: &Under) -> Line {
        let subject = match self {
            Self::Queue
            | Self::Trail
            | Self::Help
            | Self::Switcher
            | Self::Settings
            | Self::Balls(View::Board) => String::new(),
            Self::Balls(View::Everywhere) => "all workspaces".to_owned(),
            Self::Records | Self::Files => under.conversation.clone(),
            Self::Search => under.needle.clone(),
            _ => under.workspace.clone(),
        };
        let (name, reaches, close) = match self {
            Self::Admin if minted => (MINTED, Self::Admin.name(), Close::Forget),
            Self::Search => (self.name(), self.reaches(), Close::Unsearch),
            Self::Switcher if under.workspace.is_empty() => {
                (self.name(), self.reaches(), Close::Exit)
            }
            _ => (self.name(), self.reaches(), Close::Leave),
        };
        Line {
            name,
            title: if name == MINTED { MINTED } else { self.title() },
            subject,
            reaches,
            close,
            back: self != Self::Switcher,
        }
    }
}

/// **Which overlay stands**, given the one held, the focus depth and whether
/// a search answer is standing. A conversation's depth stands only over its
/// conversation; anything else held stands wherever the focus is (it names no
/// focus, so it leaves the focus alone); and with nothing held the search
/// hits stand over the roster while an answer does, and otherwise the
/// switcher does — nothing focused is the switcher's screen (§22 item 7).
/// `Search` held is not a state anything sets, and reads as nothing held.
pub fn standing(held: Option<Overlay>, depth: Depth, searched: bool) -> Option<Overlay> {
    held.filter(|overlay| *overlay != Overlay::Search)
        .filter(|overlay| !overlay.of_conversation() || depth == Depth::Conversation)
        .or_else(|| {
            (depth == Depth::Roster).then_some(if searched {
                Overlay::Search
            } else {
                Overlay::Switcher
            })
        })
}

#[cfg(test)]
mod tests;
