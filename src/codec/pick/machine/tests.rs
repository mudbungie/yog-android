//! The machine level: the server's statement of where a row is, the two
//! menus it makes, and the aim.

use super::{Aim, Machine, aim, machines, offered};
use crate::codec::pick::Place;
use crate::codec::{ProviderRow, RoleRow};

fn at(client: &str, row: &str) -> Place {
    let (client, row) = (client.to_owned(), row.to_owned());
    Place { client, row }
}

/// A listed row: the name whole, and the server's statement beside it.
fn listed(name: &str, on: Option<Place>) -> ProviderRow {
    ProviderRow {
        name: name.to_owned(),
        fact: "signed in".to_owned(),
        blocked: None,
        effort: true,
        priority: false,
        on,
    }
}

/// The server's own rows, two machines' rows, a server row whose own name
/// carries the joiner, and a machine whose name does.
fn listing() -> Vec<ProviderRow> {
    vec![
        listed("chat", None),
        listed("team@corp", None),
        listed("chat@foot", Some(at("foot", "chat"))),
        listed("spud@foot", Some(at("foot", "spud"))),
        listed("chat@cedar@desk", Some(at("cedar@desk", "chat"))),
    ]
}

fn set(provider: &str, on: Option<Place>) -> RoleRow {
    RoleRow {
        role: "worker".to_owned(),
        provider: provider.to_owned(),
        model: "m".to_owned(),
        effort: None,
        priority: false,
        on,
    }
}

fn far(name: &str) -> Machine {
    Machine::Client(name.to_owned())
}

/// **The machine menu is the server, then each machine a row states**, once
/// each and in the listing's order — and a name is never read for one: a
/// joiner in a server row's name makes no machine, and a machine whose own
/// name carries one is one machine.
#[test]
fn the_machine_menu_is_the_server_and_each_machine_a_row_states() {
    assert_eq!(
        machines(&listing()),
        [Machine::Server, far("foot"), far("cedar@desk")]
    );
    assert_eq!(machines(&[]), [Machine::Server]);
}

/// **A machine's providers are shown as it spells them and written whole**;
/// the server's own are shown whole, joiner and all.
#[test]
fn a_machines_rows_are_shown_as_it_spells_them_and_ride_whole() {
    let rows = listing();
    let shown = |machine: &Machine| -> Vec<(String, String)> {
        offered(&rows, machine)
            .into_iter()
            .map(|(shown, row)| (shown, row.name))
            .collect()
    };
    let pair = |shown: &str, name: &str| (shown.to_owned(), name.to_owned());
    assert_eq!(
        shown(&Machine::Server),
        [pair("chat", "chat"), pair("team@corp", "team@corp")]
    );
    assert_eq!(
        shown(&far("foot")),
        [pair("chat", "chat@foot"), pair("spud", "spud@foot")]
    );
    assert_eq!(shown(&far("cedar@desk")), [pair("chat", "chat@cedar@desk")]);
    assert!(shown(&far("desk")).is_empty());
}

/// **The machine is the statement of the provider in force**: the
/// assignment's own — which stands whether or not its machine is awake to
/// be listed — or, for an optimistic pick, the listing row it was picked off.
#[test]
fn the_aim_reads_the_machine_off_the_row_in_force() {
    let rows = listing();
    let row = set("local@nap", Some(at("nap", "local")));
    let aimed = aim(None, None, Some(&row), &rows);
    assert_eq!(aimed.machine, Some(far("nap")));
    assert_eq!(aimed.provider.as_deref(), Some("local@nap"));
    assert_eq!(aimed.machine_face(), "nap");
    assert_eq!(aimed.provider_face(), "local");
    let aimed = aim(None, Some("spud@foot"), Some(&row), &rows);
    assert_eq!(aimed.machine, Some(far("foot")));
    assert_eq!(aimed.provider.as_deref(), Some("spud@foot"));
    assert_eq!(aimed.provider_face(), "spud");
    let aimed = aim(None, Some("team@corp"), Some(&row), &rows);
    assert_eq!(aimed.machine, Some(Machine::Server));
    assert_eq!(aimed.machine_face(), "server");
    assert_eq!(aimed.provider_face(), "team@corp");
}

/// **A row that states no machine is the server's own, whole** — which is
/// every row of a server older than the statement, and a pick the listing
/// no longer holds.
#[test]
fn a_row_stating_no_machine_is_the_servers_own_under_its_whole_name() {
    let row = set("chat@foot", None);
    let aimed = aim(None, None, Some(&row), &[]);
    assert_eq!(aimed.machine, Some(Machine::Server));
    assert_eq!(aimed.provider_face(), "chat@foot");
    let aimed = aim(None, Some("gone@foot"), None, &listing());
    assert_eq!(aimed.machine, Some(Machine::Server));
    assert_eq!(aimed.provider_face(), "gone@foot");
}

/// **A machine being browsed stands alone**: its provider has not been
/// picked, so none is shown — not the assignment's, which is another
/// machine's row.
#[test]
fn a_browsed_machine_has_no_provider_until_one_is_picked() {
    let row = set("chat", None);
    let aimed = aim(Some(&far("foot")), Some("chat"), Some(&row), &listing());
    assert_eq!(
        aimed,
        Aim {
            machine: Some(far("foot")),
            provider: None,
            shown: None,
        }
    );
    assert_eq!(aimed.provider_face(), "provider");
}

/// Nothing set and nothing browsed is each control's own name, never a
/// value.
#[test]
fn nothing_set_is_each_controls_own_name() {
    let aimed = aim(None, None, None, &[]);
    assert_eq!(aimed.machine_face(), "machine");
    assert_eq!(aimed.provider_face(), "provider");
}
