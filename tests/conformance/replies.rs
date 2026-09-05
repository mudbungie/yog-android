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
use super::expect::{NOT_THE_MINTER, UNSENT};

pub const REPLIES: &[(&str, Expect)] = &[
    ("acked", Reads),
    ("acknowledged", Reads),
    ("advertised", Reads),
    ("agent", Refuses(UNSENT)),
    ("answered", Reads),
    ("applied", Reads),
    ("armed", Refuses(UNSENT)),
    ("attention", Reads),
    ("balls", Refuses(UNSENT)),
    ("board", Refuses(UNSENT)),
    ("clients", Refuses(UNSENT)),
    ("config", Refuses(UNSENT)),
    ("conversations", Reads),
    ("deleted", Refuses(UNSENT)),
    ("delivered", Refuses(UNSENT)),
    ("enrolled", Refuses(NOT_THE_MINTER)),
    ("fanned", Refuses(UNSENT)),
    ("files", Refuses(UNSENT)),
    ("flagged", Reads),
    ("floored", Reads),
    ("follow", Reads),
    ("governing", Refuses(UNSENT)),
    ("help", Refuses(UNSENT)),
    ("inbox", Refuses(UNSENT)),
    ("invocations", Reads),
    ("lineages", Refuses(UNSENT)),
    ("login", Refuses(UNSENT)),
    ("marks", Refuses(UNSENT)),
    ("models", Reads),
    ("nudged", Reads),
    ("ops", Reads),
    ("outcome", Reads),
    ("prepared", Reads),
    ("providers", Reads),
    ("rail", Refuses(UNSENT)),
    ("refusal", Reads),
    ("retired", Refuses(UNSENT)),
    ("roles", Reads),
    ("routed", Reads),
    ("science", Refuses(UNSENT)),
    ("search", Reads),
    ("started", Reads),
    ("step", Refuses(UNSENT)),
    ("steps", Refuses(UNSENT)),
    ("trail-cleared", Reads),
    ("transcript", Reads),
    ("work-diff", Refuses(UNSENT)),
    ("workspace-balls", Refuses(UNSENT)),
    ("workspaces", Reads),
];
