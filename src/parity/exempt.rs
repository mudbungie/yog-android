//! **The exemption file** (PARITY §7): one line per deliberately absent op,
//! each carrying a reason that cites a ball or a doc section.
//!
//! Two kinds occur — *unbuilt*, where the surface is intended and the citation
//! is the ball that will build it, and *never on this platform*, where the
//! citation is the ruling. Severability is the house test: deleting a line
//! re-reddens the gate and no code changes.
//!
//! **The format is a TOML subset, parsed here rather than by a dependency.**
//! `op = "reason"`, `#` comments, blank lines — nothing else, and anything
//! else is an error naming the line. A whole TOML parser would be a new
//! dependency (AGENTS.md rule 6) for a file whose entire grammar is one
//! shape, and a subset that refuses what it cannot read is safer than a
//! parser that accepts a nesting this gate would then ignore. What is written
//! is still valid TOML, so an editor reads it and a future dependency could
//! replace this without touching the file.
//!
//! **A reason with no citation is refused.** "Cited" is the whole difference
//! between a ledger and a list of excuses, so it is machine-checked: the
//! reason must name a ball id or a section sign.

/// One exempted op and the reason it is absent.
#[derive(Debug)]
pub(super) struct Row {
    pub(super) op: String,
    pub(super) reason: String,
}

/// Read `parity.toml`. Every non-comment line must be `op = "reason"`, every
/// op must appear once, and every reason must cite.
pub(super) fn read(text: &str) -> Result<Vec<Row>, String> {
    let mut rows: Vec<Row> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let row = parse(line)?;
        if rows.iter().any(|held| held.op == row.op) {
            return Err(format!(
                "parity.toml names `{}` twice — one op, one line",
                row.op
            ));
        }
        rows.push(row);
    }
    Ok(rows)
}

/// One line. Split at the first `=`, take the quoted remainder whole.
fn parse(line: &str) -> Result<Row, String> {
    let Some((op, reason)) = line.split_once('=') else {
        return Err(format!("parity.toml: `{line}` is not `op = \"reason\"`"));
    };
    let op = op.trim();
    let reason = reason.trim();
    let Some(reason) = reason.strip_prefix('"').and_then(|r| r.strip_suffix('"')) else {
        return Err(format!(
            "{op}: the reason must be one double-quoted string on the line"
        ));
    };
    if op.is_empty() || !op.chars().all(|c| c.is_ascii_lowercase() || c == '-') {
        return Err(format!(
            "parity.toml: `{op}` is not an op token (lowercase and hyphens)"
        ));
    }
    if !cites(reason) {
        return Err(format!(
            "{op}: the reason cites nothing — name the ball that will build it \
             (bl-xxxx) or the section that rules it out (§n)"
        ));
    }
    Ok(Row {
        op: op.to_owned(),
        reason: reason.to_owned(),
    })
}

/// What counts as a citation: a ball id, or a section sign introducing a
/// document's own numbering. Both are things a reader can go and read.
fn cites(reason: &str) -> bool {
    reason.contains('§') || ball(reason)
}

/// A ball id: `bl-` and four hex digits, the shape every id in this suite has.
fn ball(reason: &str) -> bool {
    reason.split("bl-").skip(1).any(|tail| {
        let id: Vec<char> = tail.chars().take(4).collect();
        id.len() == 4 && id.iter().all(char::is_ascii_hexdigit)
    })
}
