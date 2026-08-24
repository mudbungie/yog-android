// ast-grep corpus of deliberate rule violations. This file does NOT compile;
// it exists only so `make rules-audit` has positive and negative cases: the
// audit asserts `ast-grep scan rules/fixtures` FAILS (every rule still bites)
// while `ast-grep scan src` stays clean. Each POSITIVE section below should
// trip exactly the named rule; the NEGATIVES section at the bottom must trip
// nothing.

use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Mutex;

// ===========================================================================
// POSITIVE: no-named-lifetimes
// ===========================================================================
struct Holder<'a> {
    slice: &'a str,
}

fn first<'a>(xs: &'a [u32]) -> &'a u32 {
    &xs[0]
}

// ===========================================================================
// POSITIVE: no-pub-borrow-return
// ===========================================================================
pub fn borrow_string(s: &String) -> &String {
    s
}

pub fn borrow_impl() -> impl Iterator<Item = u32> {
    std::iter::empty()
}

pub fn borrow_elided(s: &str) -> &'_ str {
    s
}

// ===========================================================================
// POSITIVE: no-lint-suppression
// ===========================================================================
#[allow(clippy::pedantic)]
fn suppressed_prod() {}

// inner #![allow] in a non-test mod is also banned.
mod prod_mod {
    #![allow(clippy::all)]
}

// ===========================================================================
// POSITIVE: no-assert-outside-tests
// ===========================================================================
fn asserts_in_prod(n: u32) {
    assert!(n > 0);
    assert_eq!(n, 1);
    assert_ne!(n, 2);
}

// ===========================================================================
// POSITIVE: locks-outside-state (this file is NOT state.rs)
// ===========================================================================
fn uses_mutex() {
    let _m: Mutex<u32> = Mutex::new(0);
    let _r: RwLock<u32> = RwLock::new(0);
}

// ===========================================================================
// POSITIVE: no-assert-outside-tests (debug_ variants — banned too, bl-383b)
// ===========================================================================
fn debug_asserts_in_prod(n: u32) {
    debug_assert!(n > 0);
    debug_assert_eq!(n, 1);
    debug_assert_ne!(n, 2);
}

// ===========================================================================
// POSITIVE: no-rc-refcell
// ===========================================================================
fn uses_rc_refcell() {
    let _rc: Rc<u32> = Rc::new(0);
    let _cell: RefCell<u32> = RefCell::new(0);
}

// ===========================================================================
// POSITIVE: no-pub-generic-bounds
// ===========================================================================
pub fn bounded_fn<T: Clone>(t: T) -> T {
    t
}

// ===========================================================================
// POSITIVE: unsafe-outside-sys (this file is NOT src/shell/sys.rs)
// ===========================================================================
unsafe fn raw_call() {}

fn uses_unsafe_block() {
    unsafe { raw_call() }
}

#[unsafe(no_mangle)]
fn exported_unmangled() {}

pub fn where_fn<T>(t: T) -> T
where
    T: Clone,
{
    t
}

pub struct BoundedStruct<T: Clone> {
    value: T,
}

// ===========================================================================
// NEGATIVES: none of the following must match any rule.
// ===========================================================================

// '_ lifetime is allowed locally (no-named-lifetimes excepts '_ and 'static).
fn elided_ok(s: &str) -> std::slice::Iter<'_, u8> {
    s.as_bytes().iter()
}

// &'static return is allowed (no-pub-borrow-return excepts 'static... as a
// lifetime token; the return itself is caught, so it is spelled non-pub here).
pub(crate) fn static_str() -> &'static str {
    "ok"
}

// Non-pub fn returning a borrow is fine (rule targets pub only).
pub(crate) fn crate_borrow(s: &str) -> &str {
    s
}

// #[allow] and assert! inside a #[cfg(test)] mod are permitted.
#[cfg(test)]
mod tests {
    #[allow(clippy::pedantic)]
    fn suppressed_in_test() {}

    fn asserts_in_test(n: u32) {
        assert!(n > 0);
        assert_eq!(n, 1);
        assert_ne!(n, 2);
    }
}

// Non-pub generic with bounds is fine (rule targets pub only).
fn private_bounded<T: Clone>(t: T) -> T {
    t
}

// pub generic WITHOUT bounds is fine (the rule bans bounds, not generics).
pub fn unbounded_pub<T>(t: T) -> T {
    t
}

// pub fn returning an owned concrete type is fine.
pub fn owned_return() -> String {
    String::new()
}

// Arc is fine (only Rc/RefCell are banned; Cow is not a borrow return here).
fn uses_arc() -> Cow<'static, str> {
    let _a: std::sync::Arc<u32> = std::sync::Arc::new(0);
    Cow::Borrowed("x")
}
