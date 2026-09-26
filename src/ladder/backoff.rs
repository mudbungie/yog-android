//! The rest between climbs: the two rungs that cost seconds — a punch
//! window, a walk of the commons — are not climbed again until it expires,
//! doubling to [`LONGEST`], so a phone with no network settles instead of
//! walking a dark commons per dial. The sentence the last climb failed with
//! is kept so a dial inside the rest says the same thing without paying for
//! it again.

use std::time::{Duration, Instant};

/// The first rest after a failed climb, and the floor a success returns to.
const FIRST: Duration = Duration::from_secs(1);
/// The longest rest between climbs — `host::serve`'s own cap, for its reason.
const LONGEST: Duration = Duration::from_secs(64);

pub(crate) struct Backoff {
    wait: Duration,
    until: Option<Instant>,
    said: String,
}

impl Backoff {
    pub(crate) fn new() -> Backoff {
        Backoff {
            wait: FIRST,
            until: None,
            said: String::new(),
        }
    }

    /// The last failure's sentence while the rest has not expired at `now`.
    pub(crate) fn resting(&self, now: Instant) -> Option<String> {
        self.until
            .is_some_and(|until| now < until)
            .then(|| self.said.clone())
    }

    /// A climb failed at `now`: rest, and rest twice as long next time.
    pub(crate) fn failed(&mut self, now: Instant, said: String) {
        self.until = Some(now + self.wait);
        self.wait = (self.wait * 2).min(LONGEST);
        self.said = said;
    }

    /// Back to the floor: something connected, something served, or the
    /// network changed under the rest.
    pub(crate) fn settle(&mut self) {
        self.wait = FIRST;
        self.until = None;
    }
}
