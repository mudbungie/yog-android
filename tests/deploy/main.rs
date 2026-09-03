//! **The deploy target's failure arms, proved without a phone** (bl-128f).
//!
//! `make deploy-phone ADDR=<ip:port>` ends in a device nobody's gate has, and
//! the address is per-run operator input by nature — so the real run is the
//! operator's and always will be. What IS testable is everything the target
//! decides before and around that device, and it is the half that goes wrong:
//! which gradle it resolved, whether a failed build still reached `adb`,
//! whether "already connected" was mistaken for an error, and whether an
//! install that printed no `Success` was reported as one.
//!
//! The tools are faked (`fakes.rs`) and every arm is an assertion about the
//! EXIT CODE and the ordered log of what the target actually spent. The exit
//! code carries the truth or nothing here does.
//!
//! Two halves, one seam: what the target RESOLVES before it builds anything,
//! and what it does with the device once it has.

mod fakes;

mod device;
mod resolve;
