//! The tree read off the descent: every shape of sibling the wire's own
//! pre-order can put next to a row, and the cap the column shares with the
//! indent.

use super::{DEEPEST, threads};
use crate::roster::tests::under;

/// One row per depth, in the order the wire lays them out.
fn tree(depths: &[usize]) -> Vec<crate::codec::ConvRow> {
    depths
        .iter()
        .enumerate()
        .map(|(at, depth)| under(&format!("c-{at}"), 0, *depth))
        .collect()
}

#[test]
fn a_flat_list_of_roots_carries_no_rails_at_all() {
    let none: Vec<bool> = Vec::new();
    assert_eq!(
        threads(&tree(&[0, 0, 0])),
        [none.clone(), none.clone(), none]
    );
    assert!(threads(&[]).is_empty());
}

/// The last child's own rail is false — its elbow ends at the row — and every
/// child before it is true, which is the rule that carries on down to reach
/// the next one.
#[test]
fn a_childs_own_rail_says_whether_a_sibling_follows_it() {
    let listed = threads(&tree(&[0, 1, 1]));
    assert_eq!(listed, [Vec::new(), vec![true], vec![false]]);
}

/// Three depths, mixed siblings: a grandchild under a parent that has a later
/// sibling carries the outer rule as well as its own.
#[test]
fn a_grandchild_carries_its_ancestors_rule_beside_its_own() {
    let listed = threads(&tree(&[0, 1, 2, 2, 1]));
    assert_eq!(
        listed,
        [
            Vec::new(),
            vec![true],
            vec![true, true],
            vec![true, false],
            vec![false],
        ]
    );
}

/// The next SUBTREE closes every rule above it: a root row is depth 0, which
/// undercuts every level, so nothing from the tree before reaches into it.
#[test]
fn the_next_root_closes_every_rule_above_it() {
    let listed = threads(&tree(&[0, 1, 2, 0]));
    assert_eq!(
        listed,
        [Vec::new(), vec![false], vec![false, false], Vec::new()]
    );
}

/// A read whose root did not arrive is a subtree of one, and it is drawn like
/// any other row at its depth (bl-06d3's own answer, spent again here).
#[test]
fn a_row_whose_root_is_missing_is_drawn_at_its_own_depth() {
    assert_eq!(
        threads(&tree(&[2, 2])),
        [vec![false, true], vec![false, false]]
    );
}

/// Past the cap the OUTERMOST rules go, so the elbow stays in the column the
/// row's own box begins at — the indent's cap and this one are one column.
#[test]
fn the_column_is_capped_where_the_indent_is() {
    let deep = threads(&tree(&[DEEPEST + 3]));
    assert_eq!(deep.first().map(Vec::len), Some(DEEPEST));
}
