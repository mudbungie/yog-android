//! The workspace listing row — the mirror of the server's `ws_row` spelling
//! (`boundary/reply/rows.rs`): name, §3.1 kind token, attention rollups, the
//! optional pin rank and the optional lineage tip. Absent optionals are facts
//! ("not pinned", "no lineage derived yet"), never nulls to guess at.

use serde_json::Value;

use super::fields::{bool_of, opt, opt_val, str_of, usize_of};

/// One enumerated workspace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WsRow {
    pub workspace: String,
    pub kind: WsKind,
    pub attention: usize,
    pub agents: usize,
    pub running: bool,
    pub pinned: Option<usize>,
    pub config_tip: Option<ConfigTip>,
}

/// The §3.1 classification token. The server's `Named` carries the name; here
/// the row's `workspace` **is** that name, so the kind carries no second copy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WsKind {
    Named,
    Foreign,
    Replay,
}

/// A workspace's config-lineage tip, both oids: short is a label, full is
/// what a `git show` outside yog takes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigTip {
    pub oid: String,
    pub short_oid: String,
}

/// Read one workspace row, strictly.
pub(crate) fn row(v: &Value) -> Result<WsRow, String> {
    let o = v.as_object().ok_or("workspace row: not an object")?;
    let kind = match str_of(o, "kind")?.as_str() {
        "named" => WsKind::Named,
        "foreign" => WsKind::Foreign,
        "replay" => WsKind::Replay,
        other => return Err(format!("workspace row: unknown kind {other:?}")),
    };
    Ok(WsRow {
        workspace: str_of(o, "workspace")?,
        kind,
        attention: usize_of(o, "attention")?,
        agents: usize_of(o, "agents")?,
        running: bool_of(o, "running")?,
        pinned: opt(o, "pinned", usize_of)?,
        config_tip: opt_val(o, "config_tip", tip)?,
    })
}

fn tip(v: &Value) -> Result<ConfigTip, String> {
    let o = v.as_object().ok_or("config_tip: not an object")?;
    Ok(ConfigTip {
        oid: str_of(o, "oid")?,
        short_oid: str_of(o, "short_oid")?,
    })
}

#[cfg(test)]
mod tests;
