//! **Every reply shape in the corpus, and this client's decision about it** —
//! the same recorded-decision table as `requests.rs`, over `corpus/reply/`.
//!
//! **There is no round trip here, and that is not an omission.** REMOTE §3
//! asks a client to *"round-trip what it emits"*, and this client emits no
//! reply — it is always the asker (§3's routing ruling), so a reply encoder
//! would be a second implementation of the engine's own spelling with nothing
//! to check it against. What a reply fixture proves is that the decoder reads
//! the engine's real bytes rather than the bytes this repo's own tests write.
//!
//! `refusal` is the one shape with no `kind`, and it **reads**: the kind-less
//! envelope is the refusal, and decoding it hands back the engine's sentence.

use super::expect::Expect::{self, Reads, Refuses};
use super::expect::UNSENT;

pub const REPLIES: &[(&str, Expect)] = &[
    ("acked", Reads),
    ("acknowledged", Reads),
    ("advertised", Reads),
    ("agent", Reads),
    ("answered", Reads),
    ("applied", Reads),
    ("armed", Reads),
    ("attention", Reads),
    ("balls", Reads),
    ("board", Reads),
    ("clients", Reads),
    ("config", Reads),
    ("conversations", Reads),
    ("deleted", Reads),
    ("delivered", Reads),
    ("enrolled", Reads),
    ("fanned", Reads),
    ("files", Reads),
    ("flagged", Reads),
    ("floored", Reads),
    ("follow", Reads),
    ("governing", Reads),
    ("help", Refuses(UNSENT)),
    ("inbox", Reads),
    ("invocations", Reads),
    ("lineages", Reads),
    ("login", Reads),
    ("marks", Reads),
    ("models", Reads),
    ("nudged", Reads),
    ("ops", Reads),
    ("outcome", Reads),
    ("prepared", Reads),
    ("providers", Reads),
    ("rail", Reads),
    ("refusal", Reads),
    ("retired", Reads),
    ("roles", Reads),
    ("routed", Reads),
    ("science", Reads),
    ("search", Reads),
    ("started", Reads),
    ("step", Reads),
    ("steps", Reads),
    ("trail-cleared", Reads),
    ("transcript", Reads),
    ("work-diff", Reads),
    ("workspace-balls", Reads),
    ("workspaces", Reads),
];
