// NEGATIVE fixture for locks-outside-state: this file IS named state.rs, so
// Mutex/RwLock here must NOT be flagged (the rule's `ignores: **/state.rs`).
// Structural-only; does not compile.
use std::sync::Mutex;
use std::sync::RwLock;

struct AppState {
    counter: Mutex<u32>,
    table: RwLock<u32>,
}

fn build() {
    let _m: Mutex<u32> = Mutex::new(0);
    let _r: RwLock<u32> = RwLock::new(0);
}
