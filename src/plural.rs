//! **A count, worded** (bl-d56b): `1 agent`, `2 agents`, `0 agents`. One
//! helper for every place the glass says how many of a thing there are, so
//! the drawer's foot cannot say `1 tools` while another surface says it
//! right. Every noun this seat counts is a regular one (`agent`, `tool`,
//! `step`, `lineage`, `candidate`, `char`); an irregular plural is a second
//! argument the day one arrives, not a table now.

/// `n` and the noun, the noun plural unless `n` is one.
#[must_use]
pub fn counted(n: u64, noun: &str) -> String {
    if n == 1 {
        format!("1 {noun}")
    } else {
        format!("{n} {noun}s")
    }
}

#[cfg(test)]
mod tests {
    use super::counted;

    #[test]
    fn one_is_singular_and_every_other_count_is_plural() {
        assert_eq!(counted(1, "agent"), "1 agent");
        assert_eq!(counted(0, "agent"), "0 agents");
        assert_eq!(counted(2, "tool"), "2 tools");
    }
}
