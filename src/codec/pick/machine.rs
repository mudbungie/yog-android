//! **The machine level** (DESIGN §25, bl-5e0b; REMOTE §5.7): a workspace's
//! providers listing read as *machine → provider*, where the wire states it
//! as one flat list of rows.
//!
//! Since §5.7 the listing carries, after the server's own rows, one row per
//! provider per registered machine offering its model access, named
//! `<row>@<client>`. Each level is its own menu on the glass.
//!
//! **The server says which machine, and this seat never reads a name**
//! (bl-7df4; REMOTE §5.7 at edition 30). Which client a name selects is the
//! server's reading, against its own registry, and a name may carry the
//! joiner on either side of it — so the listing states the answer: each such
//! row carries its `client` and its own `row` there ([`Place`]), and so does
//! the role assignment. A row carrying neither is the server's own. This
//! module groups by the one and shows the other; the name a pick writes is
//! the listing's own, whole. **Against a server older than the keys every
//! row reads as the server's own under its whole name**: nothing is hidden
//! and nothing is attributed to a machine on a guess.
//!
//! **The machine is derived, never stored.** What is in force is the
//! provider's row; its machine is that row's own statement. The one piece of
//! state is the machine a menu is browsing between the machine pick and the
//! provider pick, and it is the caller's to hold and to drop.

use super::{Place, ProviderRow, RoleRow};

/// **Where a provider row is served**: the server's own brazen, or a
/// registered machine's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Machine {
    /// The server itself — a row no client is stated for.
    Server,
    /// A registered machine, by the name the server knows it by.
    Client(String),
}

impl Machine {
    /// The machine a row's own statement names.
    #[must_use]
    pub fn of(on: Option<&Place>) -> Self {
        on.map_or(Self::Server, |at| Self::Client(at.client.clone()))
    }

    /// What the selector and its list call this machine.
    pub fn label(&self) -> String {
        match self {
            Self::Server => "server".to_owned(),
            Self::Client(name) => name.clone(),
        }
    }
}

/// What a row called `name` is shown as under its machine: its own spelling
/// there, or the name whole for the server's own.
fn shown(on: Option<&Place>, name: &str) -> String {
    on.map_or(name, |at| at.row.as_str()).to_owned()
}

/// **The machine menu**: the server itself, then each machine a row of the
/// listing is on, in the listing's order.
pub fn machines(rows: &[ProviderRow]) -> Vec<Machine> {
    let mut found = vec![Machine::Server];
    for row in rows {
        let machine = Machine::of(row.on.as_ref());
        if !found.contains(&machine) {
            found.push(machine);
        }
    }
    found
}

/// **The provider menu under one machine**: that machine's rows, each beside
/// the name it is SHOWN by — the row as its machine spells it. The row
/// itself rides whole, so what a pick writes is the listing's own name.
pub fn offered(rows: &[ProviderRow], machine: &Machine) -> Vec<(String, ProviderRow)> {
    rows.iter()
        .filter(|row| Machine::of(row.on.as_ref()) == *machine)
        .map(|row| (shown(row.on.as_ref(), &row.name), row.clone()))
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
    /// The same provider as its machine spells it — what the selector says.
    shown: Option<String>,
}

/// **The aim, from its three sources.** A machine being browsed stands alone
/// — its provider has not been picked yet; otherwise the provider is the
/// optimistic pick or, failing one, what the workspace is set to, and the
/// machine is that row's own statement: the listing's for a pick, the
/// assignment's for what is set.
pub fn aim(
    browsing: Option<&Machine>,
    picked: Option<&str>,
    set: Option<&RoleRow>,
    rows: &[ProviderRow],
) -> Aim {
    if let Some(machine) = browsing {
        return Aim {
            machine: Some(machine.clone()),
            provider: None,
            shown: None,
        };
    }
    let listed = |name: &str| rows.iter().find(|row| row.name == name);
    let held = picked
        .map(|name| (name, listed(name).and_then(|row| row.on.as_ref())))
        .or_else(|| set.map(|row| (row.provider.as_str(), row.on.as_ref())));
    Aim {
        machine: held.map(|(_, on)| Machine::of(on)),
        provider: held.map(|(name, _)| name.to_owned()),
        shown: held.map(|(name, on)| shown(on, name)),
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

    /// The provider selector's words: the row as its machine spells it.
    pub fn provider_face(&self) -> String {
        self.shown.clone().unwrap_or_else(|| "provider".to_owned())
    }
}

#[cfg(test)]
mod tests;
