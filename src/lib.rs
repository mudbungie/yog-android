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

pub mod attention;
pub mod bootstrap;
pub mod cache;
pub mod codec;
pub mod envelope;
pub mod foot;
pub mod frame;
pub mod hello;
pub mod help;
pub mod host;
pub mod icon;
pub mod leaf;
pub mod live;
pub mod material;
pub mod outbox;
pub mod parity;
pub mod pocket;
pub mod roster;
pub mod rows;
pub mod scan;
pub mod seat;
pub mod shell;
pub mod state;
pub mod symbol;
pub mod tools;
pub mod transport;

pub(crate) mod tls;

#[cfg(test)]
pub(crate) mod test_support;
