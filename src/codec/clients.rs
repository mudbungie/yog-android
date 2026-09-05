//! **Which machines may execute for this workspace** (REMOTE §5, §5.1; DESIGN
//! §13.14): the tool hosts registered against it, and what each one says it
//! offers.
//!
//! **Two lifetimes on one row, and the pane says both** (lernie DESIGN §4.28,
//! whose ruling transfers whole). `present` is an OBSERVATION — true at the
//! instant the engine answered, recorded nowhere on either end — and the
//! advertised set is a STATEMENT the machine last made, which stands whether
//! or not it is connected. A row therefore reads *not connected* beside a full
//! set as the ordinary thing: a tool host holds its connection only while it
//! is waiting for work, so a busy machine and an absent one are
//! indistinguishable from here.
//!
//! **An advertised element has one spelling wherever it is said.** A row's
//! tools are the same three-plus-one facts this device presents in its own
//! `advertise` (REMOTE §5.1), so they are read by the same reader
//! (`codec::tools::tool_of`) rather than by a second one — including the
//! `input_schema`, which rides through as the `Value` it always has. **What is
//! not decoded is not painted**: the schema is a machine's statement to a
//! model, and an operator reading a roster of machines is asking what a box
//! can do, not what shape its arguments take.

use serde_json::{Map, Value};

use super::fields::{arr_of, bool_of, str_of};
use super::tools::{Tool, tool_of};

/// **What this workspace's machines offer**, and the workspace it was read
/// for. `clients` names one, so a roster under another is the wrong claim —
/// the same §14 pairing law `Spread::about` keeps, one noun along.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Machines {
    pub workspace: String,
    pub rows: Vec<ClientRow>,
}

impl Machines {
    /// Whether this roster is about the workspace now focused.
    #[must_use]
    pub fn about(&self, workspace: &str) -> bool {
        self.workspace == workspace
    }
}

/// One machine registered against the workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientRow {
    /// The identity the engine knows it by — its certificate's common name.
    pub client: String,
    /// **An observation and not a state**: true at the instant the engine
    /// answered, and false for a machine that is merely busy.
    pub present: bool,
    /// What it last said it offers. It stands whether or not it is connected.
    pub tools: Vec<Tool>,
}

/// Read the `clients` answer's rows.
pub(super) fn rows(o: &Map<String, Value>) -> Result<Vec<ClientRow>, String> {
    arr_of(o, "rows")?.iter().map(row).collect()
}

/// One machine.
fn row(v: &Value) -> Result<ClientRow, String> {
    let o = v
        .as_object()
        .ok_or_else(|| "clients: row is not an object".to_owned())?;
    Ok(ClientRow {
        client: str_of(o, "client")?,
        present: bool_of(o, "present")?,
        tools: arr_of(o, "tools")?
            .iter()
            .map(tool_of)
            .collect::<Result<Vec<Tool>, String>>()?,
    })
}

#[cfg(test)]
mod tests;
