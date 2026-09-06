//! **The two world surfaces** (DESIGN §13.8, bl-35bd): the attention queue as
//! a queue, and the ops trail with the two acts over it.
//!
//! **They sit at the top depth for the search's reason** (§13.6). Neither read
//! names a workspace or a conversation — the queue is every workspace's and
//! the trail is the engine's — so the screen that reaches them is the one
//! where the whole world is already what is on the glass. An entry on a
//! workspace's conversation list would say the trail was that workspace's,
//! which is a scope the wire does not carry and §8 forbids this app to imply.
//!
//! **The trail is asked for when it is opened, and the queue is never asked
//! at all.** The trail is read by nothing standing, so a surface nobody has
//! opened costs this device no radio — the §14.1 lane's own argument applied
//! at the seat. The queue is that lane's (DESIGN §14.1): it stands for the
//! seat's whole life and its frames write the one holder every screen paints
//! from, so opening this surface is a look at what is already held.
//!
//! **The queue's rows navigate; the trail's do not.** A queue row is an
//! address this seat already focuses (the workspace and the agent, in the
//! words every gesture takes), so tapping one opens that conversation, exactly
//! as a search hit does. A trail row addresses nothing: it is a line of the
//! record, and nothing this device could open would be *that action* — so it
//! paints and does not tap, the same answer a ball hit gets on the search
//! screen.
//!
//! **`clear-trail` is the first armed control in this app** (§13.2's *tap is
//! the act* has an exception at last). Every gesture the seat had until now
//! kept what it acted on: an interrupt keeps what is committed, a stop leaves
//! the transcript, a refused call stays parked. This one DISCARDS a durable
//! record, and the record is the thing every other recovery sentence in this
//! client points at (REMOTE §9.8). The arm is two taps on one control, spelled
//! in the control's own label rather than in a dialog: a phone's back gesture
//! must dismiss anything modal, and a confirmation that a back press can
//! answer is a confirmation nobody read.

use eframe::egui;

use crate::seat::Snapshot;
use crate::shell::app::Shell;

/// Which world surface is open over the roster. Navigation and nothing else —
/// no more durable than a scroll position — for `Shell::settings`' reason
/// exactly: what this device IS is derived from the leaf on disk, and what is
/// on the glass is not stored anywhere.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum World {
    /// Everything waiting on the operator, across every workspace.
    Queue,
    /// What the engine last did.
    Trail,
    /// **What each attempt on this workspace's obligations cost** (§13.12) —
    /// the candidates listing, and the three acts over it.
    Candidates,
    /// **The two armings this workspace carries** (§13.13) — the drone loop,
    /// and the alignment monitor over what it commits.
    Fleet,
    /// **Which machines may execute for this workspace** (§13.14), and what
    /// each one says it offers.
    Clients,
    /// **What this workspace's attempts changed** (§13.15) — the churn, and
    /// the bytes of any one changed file.
    Work,
    /// **The admin surface** (§13.17) — the config files, the task branch,
    /// the inbox flush, and the unmaking of the workspace itself.
    Admin,
    /// **The sign-in surface** (§13.19) — this workspace's provider rows,
    /// the act that signs one in, and the held tail of what that run says.
    SignIn,
    /// **The op table** (§13.14) — every gesture the engine speaks, read out
    /// of the vendored table and costing no wire read at all.
    Help,
    /// **The ball pane at one of its three views** (§13.9). The view rides on
    /// the navigation rather than on the answer, so a screen names itself from
    /// what was opened and paints only the answer that belongs under it.
    Balls(crate::codec::View),
}

mod admin;
mod balls;
mod candidates;
mod clients;
mod fleet;
mod help;
mod signin;
mod work;

pub(in crate::shell) use candidates::FLOOR;
pub(in crate::shell) use fleet::FLOOR as CAP;
mod trail;
mod waiting;

impl Shell {
    /// Open one of them. For the trail, opening IS the ask — an operator
    /// opening the surface is the request for its rows; the queue is held
    /// standing and asks nothing.
    pub(super) fn open_world(&mut self, world: World) {
        self.opened = Some(world);
        self.armed = false;
        // A ball or an attempt picked on one visit is not still picked on the
        // next: the act controls address a row, and a row nobody can see is
        // not one.
        self.ball = None;
        self.candidate = None;
        // A destination picked on one visit is not still picked on the next,
        // and neither is the draft it seeded: the editor's text is the file
        // that was read, and a screen that reopened holding one would be
        // offering to write bytes nobody had just looked at.
        self.destination = None;
        self.seeded = None;
        let Some(model) = self.model() else { return };
        // **A tail followed on one visit is not still followed on the next**
        // (§13.19), and it is the WORKER that holds which one — so leaving
        // the screen is a command rather than a field cleared here. It is
        // sent on the way into every surface, this one included: the lane's
        // subject is a row somebody tapped, and nobody has yet.
        model.watch_login(None);
        // **Opening IS the ask**, for the trail and for the ball pane alike:
        // both are read by nothing standing, so a surface nobody has opened
        // costs this device no radio at all (§14.1's argument, at the seat).
        // The queue is the exception and asks nothing — it is the lane's.
        match world {
            World::Trail => model.list_trail(),
            World::Balls(view) => model.list_balls(view),
            World::Candidates => model.list_candidates(),
            World::Clients => model.list_clients(),
            World::Work => model.open_work(None),
            // The admin screen opens on the mark it can read with no pick at
            // all; a config file is read by tapping its destination, because
            // opening a screen is not a request for three files.
            World::Admin => model.read_marks(),
            // **Opening IS the `providers` ask** (§13.19). The tail is not
            // asked for here: which run is followed is a row's own tap, and
            // a lane on a provider nobody has picked would be a held socket
            // for a question nobody asked.
            World::SignIn => model.list_providers(),
            // **The three that ask nothing.** The queue is the held lane's,
            // so opening it is a look at what is already held (§14.1); the
            // fleet screen reads nothing at all, because what its acts DID is
            // on the board and one fact has one home (§13.13); and the op
            // table is compiled into this binary (§13.14).
            World::Fleet | World::Help | World::Queue => (),
        }
    }

    /// **The one control that reaches one of the pane's reads**, painted where
    /// that read's subject is: `balls` and `board` on the roster, because
    /// neither names a workspace, and `workspace-balls` on a workspace's own
    /// conversation list. The name the harness taps it by is the op's own
    /// (§15.2), which is also the screen it opens.
    pub(in crate::shell) fn balls_entry(&mut self, ui: &mut egui::Ui, view: crate::codec::View) {
        let control = super::tap(ui, view.screen().into(), view.screen());
        self.note_control(view.screen(), ui, control.rect);
        if control.clicked() {
            self.open_world(World::Balls(view));
        }
    }

    /// **The aimed entries, in bands of two** (§13.14). Five full-width rows
    /// over a conversation list is most of a phone screen, and the list is
    /// what has to give way for them; bands of two is half that. TWO to a
    /// band rather than more, because `workspace-balls` is the longest screen
    /// name this app paints and a row of four would put the last of them off
    /// the glass — bl-f36e's finding, which is that a control off the glass is
    /// one the parity inventory cannot record and a thumb cannot reach.
    ///
    /// **The list is chunked rather than written out in pairs** (bl-5a56): the
    /// fifth entry arrived and an odd count has to be a band of one, which a
    /// literal of pairs cannot spell. One roster, in the order a thumb meets
    /// it, and the banding is arithmetic.
    pub(in crate::shell) fn aimed_entries(&mut self, ui: &mut egui::Ui) {
        let aimed = [
            (
                crate::codec::View::Here.screen(),
                World::Balls(crate::codec::View::Here),
            ),
            (candidates::SCREEN, World::Candidates),
            (fleet::SCREEN, World::Fleet),
            (clients::SCREEN, World::Clients),
            (work::SCREEN, World::Work),
            (admin::SCREEN, World::Admin),
            (signin::SCREEN, World::SignIn),
        ];
        for band in aimed.chunks(2) {
            self.entry_band(ui, band);
        }
    }

    /// One band of entries, each stating its own name for the harness and the
    /// op it reaches for the parity gate.
    fn entry_band(&mut self, ui: &mut egui::Ui, pair: &[(&'static str, World)]) {
        let band = egui::vec2(ui.available_width(), crate::shell::mark::TOUCH);
        ui.allocate_ui_with_layout(
            band,
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                for (name, world) in pair.iter().copied() {
                    let control = ui.add(
                        egui::Button::new(name)
                            .min_size(egui::vec2(0.0, crate::shell::mark::TOUCH)),
                    );
                    crate::shell::act::act(ui, &control, name);
                    self.note_control(name, ui, control.rect);
                    if control.clicked() {
                        self.open_world(world);
                    }
                }
            },
        );
    }

    /// **The op table's entry** (§13.14), on the roster beside the queue and
    /// the trail: what it is about is not a workspace and not a conversation.
    pub(in crate::shell) fn help_entry(&mut self, ui: &mut egui::Ui) {
        let name = help::SCREEN;
        let control = super::tap(ui, name.into(), name);
        self.note_control(name, ui, control.rect);
        if control.clicked() {
            self.open_world(World::Help);
        }
    }

    /// Paint whichever is open. One arm each, and each names its own screen
    /// (`app/probe.rs` — the name lives at the branch, never derived a second
    /// time from the same state).
    pub(super) fn world(&mut self, ui: &mut egui::Ui, snap: &Snapshot, world: World) {
        match world {
            World::Queue => self.waiting(ui, snap),
            World::Trail => self.trail(ui, snap),
            World::Balls(view) => self.balls(ui, snap, view),
            World::Candidates => self.candidates(ui, snap),
            World::Fleet => self.fleet(ui, snap),
            World::Clients => self.clients(ui, snap),
            World::Help => self.help(ui, snap),
            World::Work => self.work(ui, snap),
            World::Admin => self.admin(ui, snap),
            World::SignIn => self.signin(ui, snap),
        }
    }

    /// Leave, back to the roster. The arm goes with it: an armed control the
    /// operator walked away from is not still armed when they come back.
    fn close_world(&mut self) {
        // The tail is the worker's, so leaving is a command (§13.19) — and it
        // is sent first, while `opened` still says which screen was left.
        if let Some(model) = self.model() {
            model.watch_login(None);
        }
        self.opened = None;
        self.armed = false;
        self.ball = None;
        self.candidate = None;
        self.destination = None;
        self.seeded = None;
    }
}
