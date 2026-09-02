//! **The paint-first cache, from the model's side** (bl-de96): a second boot
//! paints what the first one was told, at the focus it was left at, before a
//! byte of wire — and a pass the engine did not answer never overwrites it.

use super::{Model, REST, cache_in, conv_reply, material, ops, pki, serve_many, settle, ws_reply};
use crate::transport::Seat;

/// The whole story in one walk: a model that answered wrote what it was
/// given; the next model over a DEAD address paints it on its very first
/// read — no settle, because there is nothing to wait for — and a failed pass
/// leaves the file exactly as it found it.
#[test]
fn a_second_boot_paints_the_last_answered_pass_before_the_wire_answers() {
    let dir = pki();
    let at = cache_in(&dir);
    let (address, served) = serve_many(
        &dir,
        "ca",
        "server",
        vec![vec![ws_reply()], vec![ws_reply()], vec![conv_reply()]],
    );
    let seat = Seat::open(&material(&dir, "ca", "client", &address)).unwrap();
    let mut model = Model::start(seat, REST, at.clone());
    settle(&mut model, &|s| !s.workspaces.is_empty());
    model.focus_workspace(Some("home".into()));
    settle(&mut model, &|s| !s.conversations.is_empty());
    drop(model);
    assert_eq!(
        ops(&served.join().unwrap()),
        ["workspaces", "workspaces", "conversations"]
    );

    // Nothing listens on port 1, so this model never gets an answer at all.
    let seat = Seat::open(&material(&dir, "ca", "client", "127.0.0.1:1")).unwrap();
    let mut model = Model::start(seat, REST, at.clone());
    let snap = model.snapshot();
    assert_eq!(snap.focus.workspace.as_deref(), Some("home"));
    assert_eq!(snap.workspaces[0].workspace, "home");
    assert_eq!(snap.conversations[0].root_id, "a1");
    // The cache is not an error state: what it paints is what the engine
    // last said, and the banner is for what the engine says now (bl-3202).
    assert_eq!(snap.error, None);

    // A pass that failed wrote nothing: the file still reads as it did.
    model.focus_workspace(Some("home".into()));
    settle(&mut model, &|s| s.focus.workspace.is_some());
    drop(model);
    let (focus, kept, _) = crate::cache::read(&at).unwrap();
    assert_eq!(focus.workspace.as_deref(), Some("home"));
    assert_eq!(kept.conversations[0].root_id, "a1");
}
