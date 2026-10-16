//! **The open set** (DESIGN §23 item 2; lernie §4.44 item 2): which
//! conversations the operator focused this run, per workspace, in the order
//! first focused — what the tab strip over the transcript paints.
//!
//! **Open is a navigation the operator performed**, which no read can answer,
//! so the set is per run and dies with the process: a seat that reopened
//! yesterday's tabs would be claiming a navigation nobody performed today.
//! **It is filled by the frame that paints the focused conversation and by
//! nothing else** ([`Tabs::focused`]): every door that focuses one — the
//! switcher, the dock, the ask, a start resolving — ends in that frame, so
//! there is no second act that opens a tab and nothing to keep in step.
//!
//! Pure and host-tested; `shell::tabs` paints the strip.

/// **How many tabs stand before the strip does** (§23 item 2): with one
/// conversation open the strip is the name the rule already carries, 48
/// points spent on nothing, which a phone's glass refuses.
pub const FROM: usize = 2;

/// The open conversations, each under the workspace it was focused in.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tabs {
    open: Vec<(String, String)>,
}

impl Tabs {
    /// **A conversation is painted focused**: it is open from now on, where
    /// it was not already.
    pub fn focused(&mut self, workspace: &str, agent: &str) {
        if !self.is_open(workspace, agent) {
            self.open.push((workspace.to_owned(), agent.to_owned()));
        }
    }

    /// Whether this conversation is open.
    #[must_use]
    pub fn is_open(&self, workspace: &str, agent: &str) -> bool {
        self.open.iter().any(|(w, a)| w == workspace && a == agent)
    }

    /// **The strip's tabs** under one workspace, in the order first focused.
    #[must_use]
    pub fn strip(&self, workspace: &str) -> Vec<String> {
        self.open
            .iter()
            .filter(|(w, _)| w == workspace)
            .map(|(_, a)| a.clone())
            .collect()
    }

    /// **Close one tab**: it leaves, and the answer is the conversation the
    /// focus moves to where it was the focused one — the neighbour to its
    /// left, or to its right at the left end — and `None` where it was the
    /// last, which puts the focus down to the workspace.
    pub fn close(&mut self, workspace: &str, agent: &str) -> Option<String> {
        let strip = self.strip(workspace);
        let at = strip.iter().position(|a| a == agent)?;
        self.open.retain(|(w, a)| !(w == workspace && a == agent));
        let neighbour = at
            .checked_sub(1)
            .or(if strip.len() > 1 { Some(1) } else { None });
        neighbour.and_then(|i| strip.get(i).cloned())
    }

    /// **A tab the listing no longer knows leaves with it** (lernie bl-6dfd's
    /// rule), except the focused one — a start's claim stands under a name
    /// the listing has not resolved yet, and the frame that resolves it
    /// focuses the root, after which the claim's tab is no longer the focus
    /// and goes on the next prune.
    pub fn prune(&mut self, workspace: &str, known: &[String], focus: Option<&str>) {
        self.open.retain(|(w, a)| {
            w != workspace || known.iter().any(|k| k == a) || Some(a.as_str()) == focus
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_focus_opens_once_and_the_strip_is_one_workspaces_in_order() {
        let mut tabs = Tabs::default();
        tabs.focused("w", "b");
        tabs.focused("w", "a");
        tabs.focused("w", "b");
        tabs.focused("v", "c");
        assert_eq!(tabs.strip("w"), ["b", "a"]);
        assert_eq!(tabs.strip("v"), ["c"]);
        assert!(tabs.is_open("w", "a"));
        assert!(!tabs.is_open("v", "a"));
    }

    #[test]
    fn a_close_moves_the_focus_to_the_left_neighbour_or_the_right_at_the_left_end() {
        let mut tabs = Tabs::default();
        for agent in ["a", "b", "c"] {
            tabs.focused("w", agent);
        }
        assert_eq!(tabs.close("w", "b").as_deref(), Some("a"));
        assert_eq!(tabs.close("w", "a").as_deref(), Some("c"));
        assert_eq!(tabs.close("w", "c"), None);
        assert_eq!(tabs.close("w", "c"), None);
        assert!(tabs.strip("w").is_empty());
    }

    #[test]
    fn a_tab_the_listing_forgot_leaves_unless_it_is_the_focus() {
        let mut tabs = Tabs::default();
        tabs.focused("w", "a");
        tabs.focused("w", "claim");
        tabs.focused("v", "gone");
        tabs.prune("w", &["a".to_owned()], Some("claim"));
        assert_eq!(tabs.strip("w"), ["a", "claim"]);
        tabs.prune("w", &["a".to_owned()], Some("a"));
        assert_eq!(tabs.strip("w"), ["a"]);
        assert_eq!(tabs.strip("v"), ["gone"]);
    }
}
