//! **The wire version this build speaks, and the changelog of every bump** —
//! its own file (bl-5070): the number is one line and the reasoning behind
//! each move of it is the rest, so [`super`] is the preface EXCHANGE and this
//! is what the preface STATES and why it ever changed. Together they would sit
//! in the ≥200 pre-split band, and they are edited for unrelated reasons.
//!
//! **The number is not written here either** (bl-6fec). It lives in the
//! repo-root `PROTOCOL` file — one line, one integer — which `build.rs`
//! compiles into the constant this module re-exports. That is where a bump is
//! made, and nothing under `src` says the integer: the gates that read this
//! number are other repositories fetching ONE PATH out of a tree they do not
//! build, and a Rust path is not a stable address for that (yog split its own
//! `src/wire/hello.rs` and left the old path re-exporting, which a build
//! cannot notice and a regex reads as no declaration). So this file is the
//! ledger and the re-export; the file at the root is the fact.

/// The protocol this build speaks. Mirrors the server's `wire::hello::PROTOCOL`
/// — one integer, and **a new verb is not a bump**: an unknown `op` or reply
/// `kind` already refuses in band naming it, which is the boundary correcting
/// itself rather than two protocols meeting. It moves when an *existing* shape
/// changes meaning: the framing, the envelope, or what a spelling already in
/// use is taken to say.
///
/// **19, and the ledger below is now HISTORY** (yog REMOTE §3.2, bl-e598;
/// landed here bl-93cd). The number went 13 → 18 in one week, and four of
/// those five bumps carried nothing but additions — each costing three
/// consumer repositories a re-vendor and a window in which no published suite
/// composed. So `PROTOCOL` became a **major**: it moves only on a breaking
/// change — a field removed or re-typed, a meaning changed under a spelling
/// still in use, a field the engine newly *requires* on a request — and 18 →
/// 19 is the last bump of the old kind and the first of the new. Every
/// ADDITION now ships with no bump at all and is stamped an **edition** per
/// field path in `corpus/shapes.json`; the reader is grows-only, the preface
/// states the edition beside the version, and `crate::ledger` is where all of
/// that lives (DESIGN §2.1). Every entry from here down describes a bump that
/// would not be one today, which is exactly why they are kept.
///
/// **18 since the re-vendor of bl-5070.** Seventeen moves stand behind it and
/// every one is recorded below — as a shape this seat reads, or as one whose
/// move cost it the integer and nothing else. 2 was the tool-host pair
/// (`subject_cwd` on an advertised element, `cwd` on an invocation — REMOTE
/// §5.1, §5.3); **3** put `failure` on the conversation row, the agent answer
/// and the queue row (§9.10); **4** put `flag` on the queue row (§9.11); **5**
/// rewrote `reply/governing` (§9.12); **6** minted the §9.4 tuning pair and
/// widened `reply/providers`; **7** put `surface` on every `reply/help` row —
/// the interface-parity classification `crate::parity` judges this seat by;
/// **8** put `wrote` on `reply/advertised`, the foot's own receipt
/// (`codec::reply`, DESIGN §6). `last_active_unix` rode in at 2 with §9.9.
///
/// **9** (§9.16) put the `wounded` entry on `reply/transcript` — the settled-
/// failure notice, read by `codec::transcript` and painted by `rows::wounded`
/// — and took `auth_failed` off `reply/steps`, a shape this seat refuses.
/// **10** (§14.1) moved **no field at all**: `attention` became follow-class,
/// the same ask and the same reply shape answered as a *sequence* by an intake
/// that can hold — precisely "what a spelling already in use is taken to
/// say", and the one class of move the corpus ledger cannot see, since frame
/// count is not a field signature. **11** (§9.17) put `failed`, `exit_label`
/// and `standing` on `reply/ops`, so the trail reads the engine's words rather
/// than classifying `exit` (`codec::trail`). **12** put `says` on every queue
/// row — the firing rules in words, one home on the engine (`codec::queue`).
/// **13** (§9.18) put a typed `settings` array on `reply/config`, a shape this
/// seat refuses; it cost the integer and nothing else.
///
/// **14** moved two shapes this seat reads and one it does not send whole:
/// `reply/clients` rows gained `last_seen` (§5), the third durable fact — a
/// stamp absent for a client that has never dialled, which is the reading that
/// tells a sleeping machine from a ghost, painted by `roster::spoke`; and
/// `enroll` gained an optional `address` (§8.4), the route the enrolled device
/// will dial, which is a fact about a box this seat cannot see and is
/// therefore a recorded refusal here (`codec::enroll`). **15** (§5.5) put the
/// **tool window** on the follow lane: `tools` beside the fold, required and
/// empty-included, two transitions per call merged by `tool_use`
/// (`codec::follow::window`) and painted as rows the record has not caught up
/// with (`crate::live`). The lane's SUBJECT moved with it — a follow read
/// follows a step rather than the model call inside it — which is a change of
/// meaning under an unchanged spelling, the class the ledger cannot see: this
/// seat consumed it by gating the prose half on `Flight::Inference` and the
/// window on any flight at all. **16** (§9.20) put `client` on every
/// `reply/ops` row, the identity that made the act — a leaf's common name, or
/// `local` for yog's own — which the trail paints beside the origin.
///
/// **17** (yog bl-ebef, bl-6661) is two shapes under one version, both the
/// same litany 0.0.11 pin landing. A delivered row — on `reply/transcript` and
/// on `reply/inbox`'s deposit envelope alike — gained an optional
/// `sender_name` / `from_name`: the sender's display name, present exactly
/// when the sender is an agent wearing one, absent (never null) otherwise. The
/// framing sender is the FILENAME's origin token, so every message a child
/// sent was headed by sixty characters of timestamped hex — which on a phone
/// is the whole width the header has, which is why the name takes the header
/// here and the id keeps riding beside it (`codec::transcript`,
/// `codec::records::inbox`, `rows::project`). And the §6 signal vocabulary
/// gained `truncated`, a turn cut off at the request's output cap: **a new
/// VALUE and not a field**, which the corpus ledger cannot see and a strict
/// decoder would refuse by name — this seat carries `signals` as its tokens by
/// a recorded narrowing (`codec::queue`), so the word arrives painted and
/// nothing here had to move. `doctor` arrived with them as a whole new shape,
/// which §3 exempts from the bump: it is a recorded refusal in both directions
/// and a `parity.toml` line until bl-a393 builds its surface.
///
/// **18** (yog bl-58bb, bl-ab53) is two shapes under one version, and the
/// engine's own entry says a later lane landing on 18 adds its shape there
/// rather than taking 19 — because since yog bl-bca2 a raise HOLDS the release
/// until three consumer mains vendor it, so each extra number is another window
/// in which no published suite composes.
///
/// The follow lane's tool-window entry gained **`held`**: the capability
/// control's own sentence about a call it parked, beside the `tool_use` id and
/// the tool name the entry already carried. A held invocation is parked
/// *before* the executor is entered, so it lands neither of the two files the
/// window's pair of transitions is made of — the lane reported the
/// conversation at rest and ended the stream at the exact moment the operator
/// was the thing it was waiting for, and on a foot lane every call to a
/// non-shell tool is held. Its **presence** is the status, the discipline
/// `exit_code` already carries on the same entry
/// (`codec::follow::window`), and the row it becomes wears the attention
/// accent — the one state on this glass that means *asking for you*
/// (`rows::windowed`, `docs/STYLE.md` §3).
///
/// And `reply/steps` rows gained a fourth `framing` word, **`in_flight`**: the
/// step being written right now, told apart from the one an interrupt cut.
/// **A new VALUE and not a field**, which the corpus ledger cannot see and
/// which a strict decoder built against 17 refuses by name — and this seat's
/// decoder is now one of them (`codec::records::steps`), because a screen
/// branches on the word: the census paints a live step in the working accent
/// and a cut one in the annotation accent, where every live step used to read
/// `killed`.
///
/// And `request/answer` and `reply/answered` gained **`scope`** (yog bl-94a5),
/// over `call | conversation | workspace`: how far one capability answer
/// stands — the held call, its class for this conversation and its descent, or
/// that class for every conversation in the workspace. **Required in both
/// directions** rather than optional-defaults-to-`call`, because an absent
/// field would let the two ends disagree about how wide an instruction was and
/// *wider* is the reading nobody may reach by accident. This seat reads it back
/// off the receipt rather than assuming what it sent, and offers it as a
/// chooser above the three verdicts, defaulted narrow and reset whenever the
/// parked call changes (`codec::hold`, DESIGN §13.7).
///
/// And every shape carrying a **`prepared` body gained `role`** (yog bl-9ced)
/// — four of them, `reply/prepared`, `request/prompt`, `reply/fanned` and
/// `request/fan` — the role a conversation is born on, beside the `lineage` it
/// already carried and read out of the same config commit. litany 0.0.12 made
/// `--role` resolve a root's soul, provider assignment and tool grant as any
/// role the governing commit declares, which turns plan mode from *a lineage a
/// workspace must author, forever* into one field of a start. `prepare`
/// answers **null** — litany's `worker`, spelled as an absence exactly as
/// `lineage`'s default is — and the SEAT is what writes it back on the
/// `/prompt` it deposits. This client carries the null faithfully and writes
/// none (`codec::start` says why, and bl-045b is the exit).
///
/// **`PROTOCOL_PUBLISHED` is the engine's and is deliberately not vendored.**
/// yog gained it in the same commit: the newest version it has PUBLISHED, read
/// by ONE thing — its corpus generator's rule that a shape may not change at a
/// version already in use, which used to be judged against the version the
/// record was last generated at and so refused the second lane of one
/// unreleased wave. This repository has no generator and no such rule: it
/// vendors a record yog wrote. A constant with no reader is a fact that goes
/// stale unwatched, so there is none here.
///
/// **The learning loop arrived with them and cost no integer at all.**
/// `proposals`, `proposal` and `reply/proposals` are whole new shapes (yog
/// bl-dd88, REMOTE §9.22), which §3 exempts from the bump: a peer that has not
/// heard of a verb refuses it in band by name. What a client owes them is a
/// re-vendor to GAIN them, which is what this one is — the listing and its two
/// verdicts are members of the §9 config family, so they are on the admin
/// screen where a config is read and written (DESIGN §13.17).
///
/// **10 is the bump that was a design decision here, not a re-vendor.** The
/// wire intake this seat dials HOLDS a follow-class read: the first frame at
/// connect, a frame per change, a terminator when the hold ends — thirty
/// seconds, the follow lane's own bound. A one-shot read of a held lane
/// blocks for the whole hold, and the standing pass used to make exactly that
/// read of `attention` on every cycle and of `follow` at a 500 ms rest. So
/// since bl-8e3c this seat holds both lanes beside the pass (DESIGN §14.1):
/// `seat::lane` parks a reader on the connection and hands each frame to the
/// worker, which folds it — the §5.5 append fold for `follow`, replacement
/// for `attention` — and the pass never waits on either.
///
/// **A move in a shape this codec does not spell still moves this number,
/// and that is the point.** `reply/config`'s typed `settings` array is the
/// standing example — a ride-through this seat reads past on purpose
/// (`codec::admin`) — so 13 cost this client nothing but the integer, and the
/// integer is the whole of what the handshake gates on. A seat that stayed at
/// 8 because "nothing it reads changed" would simply stop speaking to the
/// engine, which is what this ball was filed against. *(This paragraph named
/// `governing` and `steps` beside it until bl-5070; bl-146b's records screen
/// has read both since, and the conformance table is the authority for which
/// shapes are refused — not a list here that goes stale as surfaces land.)*
///
/// **The version is the only thing that breaks an old seat, and it breaks it
/// on purpose.** An unknown FIELD is tolerated — this codec reads the fields
/// it spells and ignores the rest, which `codec::conv`'s own test pins. What
/// ends an old build is this preface: fail-closed, both ways, by §3's design.
/// Since 19 an unknown WORD is tolerated too, and an absent post-floor field
/// with it, so what the preface now ends is only a build of the wrong MAJOR —
/// which is the whole of the change.
pub use protocol::PROTOCOL;

/// The generated constant: `build.rs` writes it from the repo-root `PROTOCOL`
/// file, and this is the only place under `src` that names it.
mod protocol {
    include!(concat!(env!("OUT_DIR"), "/protocol.rs"));
}
