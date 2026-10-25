//! yog-android: the phone seat.
//!
//! An Android client of yog's remote boundary (yog `docs/REMOTE.md`): a wire
//! client that dials the server over mTLS and speaks `Act | Ask` in, `Reply`
//! out. The server owns the world; this crate paints replies and posts
//! gestures, and holds no durable state of its own.
//!
//! `docs/DESIGN.md` is this repo's architecture authority; `AGENTS.md` is the
//! code-style authority. The wire contract is defined by the server
//! (yog REMOTE §3) and mirrored here — where the two implementations
//! disagree, one of them is a bug.

pub mod ask;
pub mod attention;
pub mod bootstrap;
pub mod cache;
pub mod codec;
pub mod dht;
pub mod dock;
pub mod draft;
pub mod drawer;
pub mod envelope;
pub mod foot;
pub mod frame;
pub mod hello;
pub mod help;
pub mod host;
pub mod icon;
pub mod ladder;
pub mod leaf;
pub mod ledger;
pub mod live;
pub mod markdown;
pub mod material;
pub mod outbox;
pub mod overlay;
pub mod panel;
pub mod parity;
pub mod plural;
pub mod pocket;
pub mod rendezvous;
pub mod roster;
pub mod rows;
pub mod scan;
pub mod seat;
pub mod settings;
pub mod shell;
pub mod signin;
pub mod state;
pub mod status;
pub mod switcher;
pub mod symbol;
pub mod tabs;
pub mod theme;
pub mod tools;
pub mod transport;
pub mod update;
pub mod verbs;

pub(crate) mod tls;

#[cfg(test)]
pub(crate) mod test_support;
