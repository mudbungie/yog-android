//! yog-android: the phone seat.
//!
//! An Android client of yog's remote boundary (yog `docs/REMOTE.md`): a wire
//! client that dials the engine over mTLS and speaks `Act | Ask` in, `Reply`
//! out. The engine owns the world; this crate paints replies and posts
//! gestures, and holds no durable state of its own.
//!
//! `docs/DESIGN.md` is this repo's architecture authority; `AGENTS.md` is the
//! code-style authority. The wire contract is defined by the server
//! (yog REMOTE §3) and mirrored here — where the two implementations
//! disagree, one of them is a bug.

pub mod codec;
pub mod frame;
pub mod material;
pub mod shell;
pub mod transport;

pub(crate) mod tls;

#[cfg(test)]
pub(crate) mod test_support;
