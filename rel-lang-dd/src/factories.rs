use alloc::rc::Rc;

use crate::{Manager, Node};

impl Manager {
    /// Returns a node for the function: `f(b) = b[i]`
    pub fn bit(&self, i: u64) -> Node {
        let then_child = self.true_node();
        let else_child = self.false_node();
        self.get_or_insert(i, &then_child, &else_child)
    }

    /// Returns a node for the function: `f(b) = all(b[j] if j = i else !b[j] for j < n)`
    pub fn minterm(&self, i: u64, n: u64) -> Node {
        let mut node = self.cache.true_node();
        let false_node = self.cache.false_node();
        for j in (0..n).rev() {
            if j == i {
                node = self.cache.get_or_insert(j, &node, &false_node);
            } else {
                node = self.cache.get_or_insert(j, &false_node, &node);
            }
        }
        Node {
            inner: node,
            cache: Rc::clone(&self.cache),
        }
    }
}
