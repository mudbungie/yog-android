//! The machine level: the split, the two menus it makes, and the aim.

use super::{Aim, Machine, aim, machines, offered, roster, split};
use crate::codec::{ClientRow, Machines, ProviderRow, RoleRow};

fn names(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}

fn listing(rows: &[&str]) -> Vec<ProviderRow> {
    rows.iter()
        .map(|name| ProviderRow {
            name: (*name).to_owned(),
            fact: "signed in".to_owned(),
            blocked: None,
            effort: true,
            priority: false,
        })
        .collect()
}

fn set(provider: &str) -> RoleRow {
    RoleRow {
        role: "worker".to_owned(),
        provider: provider.to_owned(),
        model: "m".to_owned(),
        effort: None,
        priority: false,
    }
}

fn far(name: &str) -> Machine {
    Machine::Client(name.to_owned())
}

/// **The server's rule, case for case**: a registered suffix behind the
/// joiner is a machine, the longest registered name wins, and a name with no
/// row in front of the joiner is not a qualification.
#[test]
fn a_name_is_split_against_the_roster_and_the_longest_client_wins() {
    let clients = names(&["foot", "cedar@foot", "desk"]);
    assert_eq!(
        split("local@foot", &clients),
        ("local".to_owned(), far("foot"))
    );
    assert_eq!(
        split("local@cedar@foot", &clients),
        ("local".to_owned(), far("cedar@foot")),
        "two registered names fit, and the longer is the machine"
    );
    assert_eq!(
        split("@foot", &clients),
        ("@foot".to_owned(), Machine::Server)
    );
    assert_eq!(
        split("localfoot", &clients),
        ("localfoot".to_owned(), Machine::Server)
    );
}

/// **A joiner alone proves nothing**: a row may carry one in its own name,
/// and a machine registered elsewhere is no machine here — so without a
/// roster that names the suffix, the name is the server's own, whole.
#[test]
fn a_suffix_no_roster_names_is_the_servers_own_row() {
    assert_eq!(
        split("local@foot", &[]),
        ("local@foot".to_owned(), Machine::Server)
    );
    assert_eq!(
        split("team@corp", &names(&["foot"])),
        ("team@corp".to_owned(), Machine::Server)
    );
}

/// The roster is the focused workspace's, or nothing.
#[test]
fn the_roster_is_read_only_under_its_own_workspace() {
    let held = Machines {
        workspace: "home".to_owned(),
        rows: vec![ClientRow {
            client: "foot".to_owned(),
            present: false,
            tools: Vec::new(),
            last_seen: None,
        }],
    };
    assert_eq!(roster(Some(&held), Some("home")), names(&["foot"]));
    assert!(roster(Some(&held), Some("away")).is_empty());
    assert!(roster(Some(&held), None).is_empty());
    assert!(roster(None, Some("home")).is_empty());
}

/// **The machine menu is the server, then each machine the listing names**,
/// once each and in the listing's order — never the whole roster.
#[test]
fn the_machine_menu_is_the_server_and_each_machine_the_listing_names() {
    let clients = names(&["foot", "desk", "idle"]);
    let rows = listing(&["chat", "local", "chat@foot", "local@foot", "chat@desk"]);
    assert_eq!(
        machines(&rows, &clients),
        [Machine::Server, far("foot"), far("desk")]
    );
    assert_eq!(machines(&[], &clients), [Machine::Server]);
}

/// **A machine's providers are shown bare and written whole.**
#[test]
fn a_machines_rows_are_shown_without_the_qualification_and_ride_whole() {
    let clients = names(&["foot"]);
    let rows = listing(&["chat", "local", "chat@foot", "spud@foot"]);
    let shown = |machine: &Machine| -> Vec<(String, String)> {
        offered(&rows, &clients, machine)
            .into_iter()
            .map(|(shown, row)| (shown, row.name))
            .collect()
    };
    assert_eq!(
        shown(&Machine::Server),
        [
            ("chat".to_owned(), "chat".to_owned()),
            ("local".to_owned(), "local".to_owned())
        ]
    );
    assert_eq!(
        shown(&far("foot")),
        [
            ("chat".to_owned(), "chat@foot".to_owned()),
            ("spud".to_owned(), "spud@foot".to_owned())
        ]
    );
    assert!(shown(&far("desk")).is_empty());
}

/// **The machine is derived from the provider in force**: the assignment's,
/// or the optimistic pick's over it.
#[test]
fn the_aim_reads_the_machine_off_the_provider_in_force() {
    let clients = names(&["foot"]);
    let row = set("chat@foot");
    let at = aim(None, None, Some(&row), &clients);
    assert_eq!(at.machine, Some(far("foot")));
    assert_eq!(at.provider.as_deref(), Some("chat@foot"));
    assert_eq!(at.machine_face(), "foot");
    assert_eq!(at.provider_face(&clients), "chat");
    let at = aim(None, Some("local"), Some(&row), &clients);
    assert_eq!(at.machine, Some(Machine::Server));
    assert_eq!(at.machine_face(), "server");
    assert_eq!(at.provider_face(&clients), "local");
}

/// **A machine being browsed stands alone**: its provider has not been
/// picked, so none is shown — not the assignment's, which is another
/// machine's row.
#[test]
fn a_browsed_machine_has_no_provider_until_one_is_picked() {
    let clients = names(&["foot"]);
    let row = set("chat");
    let at = aim(Some(&far("foot")), Some("chat"), Some(&row), &clients);
    assert_eq!(
        at,
        Aim {
            machine: Some(far("foot")),
            provider: None
        }
    );
    assert_eq!(at.provider_face(&clients), "provider");
}

/// Nothing set and nothing browsed is each control's own name, never a
/// value.
#[test]
fn nothing_set_is_each_controls_own_name() {
    let at = aim(None, None, None, &[]);
    assert_eq!(at.machine_face(), "machine");
    assert_eq!(at.provider_face(&[]), "provider");
}
