//! **The machine level** (DESIGN §25, bl-5e0b; REMOTE §5.7): a workspace's
//! providers listing read as *machine → provider*, where the wire states it
//! as one flat list of names.
//!
//! Since §5.7 the listing carries, after the server's own rows, one row per
//! provider per registered machine offering its model access, named
//! `<row>@<client>`. Each level is its own menu on the glass, so this module
//! is the one place the name is taken apart — and nothing is ever put back
//! together, because the name a pick writes is the listing's own, whole.
//!
//! **The split is the server's rule, against the server's roster.** The
//! listing does not carry the client as a field, and a row's own name may
//! carry the joiner, so the name alone cannot say where the row ends. The
//! server reads it against the clients registered in the workspace — the
//! longest registered name that fits wins, and a row must stand in front of
//! the joiner — and so does this seat, against the `clients` read of the same
//! workspace. **With no roster in hand every name reads as the server's own,
//! whole**: nothing is hidden and nothing is attributed to a machine on a
//! guess.
//!
//! **The machine is derived, never stored.** What is in force is the
//! provider's name; its machine is this module's reading of it. The one piece
//! of state is the machine a menu is browsing between the machine pick and
//! the provider pick, and it is the caller's to hold and to drop.

use super::{ProviderRow, RoleRow};
use crate::codec::Machines;

/// What joins a row to the machine that serves it (REMOTE §5.7).
const JOINER: char = '@';

/// **Where a provider row is served**: the server's own brazen, or a
/// registered machine's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Machine {
    /// The server itself — a bare row.
    Server,
    /// A registered machine, by the name the server knows it by.
    Client(String),
}

impl Machine {
    /// What the selector and its list call this machine.
    pub fn label(&self) -> String {
        match self {
            Self::Server => "server".to_owned(),
            Self::Client(name) => name.clone(),
        }
    }
}

/// **The roster a name is read against**: the clients registered in
/// `workspace`, or nothing where the roster in hand is another workspace's or
/// has not been read.
pub fn roster(machines: Option<&Machines>, workspace: Option<&str>) -> Vec<String> {
    machines
        .filter(|machines| workspace.is_some_and(|workspace| machines.about(workspace)))
        .map(|machines| machines.rows.iter().map(|row| row.client.clone()).collect())
        .unwrap_or_default()
}

/// **One provider name, read as its row and its machine** — the server's own
/// rule (yog `bz_host::far::split`): the longest registered client that
/// fits, with a row in front of the joiner; anything else is a bare row.
pub fn split(name: &str, clients: &[String]) -> (String, Machine) {
    clients
        .iter()
        .filter_map(|client| {
            let row = name.strip_suffix(client.as_str())?.strip_suffix(JOINER)?;
            (!row.is_empty()).then(|| (row.to_owned(), Machine::Client(client.clone())))
        })
        // The longest client leaves the shortest row.
        .min_by_key(|(row, _)| row.len())
        .unwrap_or_else(|| (name.to_owned(), Machine::Server))
}

/// **The machine menu**: the server itself, then each machine that
/// qualifies a row of the listing, in the listing's order.
pub fn machines(rows: &[ProviderRow], clients: &[String]) -> Vec<Machine> {
    let mut found = vec![Machine::Server];
    for row in rows {
        let (_, machine) = split(&row.name, clients);
        if !found.contains(&machine) {
            found.push(machine);
        }
    }
    found
}

/// **The provider menu under one machine**: that machine's rows, each beside
/// the name it is SHOWN by — the row without its qualification. The row
/// itself rides whole, so what a pick writes is the listing's own name.
pub fn offered(
    rows: &[ProviderRow],
    clients: &[String],
    machine: &Machine,
) -> Vec<(String, ProviderRow)> {
    rows.iter()
        .filter_map(|row| {
            let (shown, on) = split(&row.name, clients);
            (on == *machine).then(|| (shown, row.clone()))
        })
        .collect()
}

/// **What the selectors are pointed at**: a machine, and under it a
/// provider by its whole name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Aim {
    /// The machine, or none where nothing is set and nothing is browsed.
    pub machine: Option<Machine>,
    /// The provider's name as the wire spells it, qualification included.
    pub provider: Option<String>,
}

/// **The aim, from its three sources.** A machine being browsed stands alone
/// — its provider has not been picked yet; otherwise the provider is the
/// optimistic pick or, failing one, what the workspace is set to, and the
/// machine is read off that name.
pub fn aim(
    browsing: Option<&Machine>,
    picked: Option<&str>,
    set: Option<&RoleRow>,
    clients: &[String],
) -> Aim {
    if let Some(machine) = browsing {
        return Aim {
            machine: Some(machine.clone()),
            provider: None,
        };
    }
    let provider = picked
        .map(str::to_owned)
        .or_else(|| set.map(|row| row.provider.clone()));
    Aim {
        machine: provider.as_deref().map(|name| split(name, clients).1),
        provider,
    }
}

impl Aim {
    /// The machine selector's words: the machine, or the control's own name
    /// where there is none to state (§8 — never a value nobody set).
    pub fn machine_face(&self) -> String {
        self.machine
            .as_ref()
            .map_or_else(|| "machine".to_owned(), Machine::label)
    }

    /// The provider selector's words: the row without its qualification.
    pub fn provider_face(&self, clients: &[String]) -> String {
        self.provider
            .as_deref()
            .map_or_else(|| "provider".to_owned(), |name| split(name, clients).0)
    }
}

#[cfg(test)]
mod tests;
