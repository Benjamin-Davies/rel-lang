use crate::{Node, Rc, factories::bit, manager::Cache, node, ops::if_then_else};

impl Node {
    pub fn shift(&self, diff: i64) -> Self {
        Self {
            cache: Rc::clone(&self.cache),
            inner: shift(&self.cache, &self.inner, diff),
        }
    }

    /// Equivalent to CUDD's `shift_bdd` function.
    pub fn split_shift(&self, border: u64, diff_1: i64, diff_2: i64) -> Self {
        Self {
            cache: Rc::clone(&self.cache),
            inner: split_shift(&self.cache, &self.inner, border, diff_1, diff_2),
        }
    }
}

fn shift(cache: &Rc<Cache>, node: &Rc<node::Inner>, diff: i64) -> Rc<node::Inner> {
    if diff == 0 {
        return node.clone();
    }

    match &node.kind {
        node::Kind::True => cache.true_node(),
        node::Kind::False => cache.false_node(),
        node::Kind::NonTerminal {
            level,
            then_child,
            else_child,
            cache: _,
        } => {
            let new_then = shift(cache, then_child, diff);
            let new_else = shift(cache, else_child, diff);
            cache.get_or_insert((*level as i64 + diff) as u64, &new_then, &new_else)
        }
    }
}

fn split_shift(
    cache: &Rc<Cache>,
    node: &Rc<node::Inner>,
    border: u64,
    diff_1: i64,
    diff_2: i64,
) -> Rc<node::Inner> {
    if (diff_1 == 0 || border == 0) && diff_2 == 0 {
        return node.clone();
    }

    match &node.kind {
        node::Kind::True => cache.true_node(),
        node::Kind::False => cache.false_node(),
        node::Kind::NonTerminal {
            level,
            then_child,
            else_child,
            cache: _,
        } => {
            let new_then = split_shift(cache, then_child, border, diff_1, diff_2);
            let new_else = split_shift(cache, else_child, border, diff_1, diff_2);

            let condition = if *level < border {
                bit(cache, (*level as i64 + diff_1) as u64)
            } else {
                bit(cache, (*level as i64 + diff_2) as u64)
            };

            if_then_else(cache, &condition, &new_then, &new_else)
        }
    }
}
