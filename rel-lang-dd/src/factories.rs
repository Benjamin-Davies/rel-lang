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
        debug_assert!(i < n);

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

    /// Returns a node for the function: `f(b) = all(b[i] == v[i] for i < n)`
    pub fn minterm_vec<I>(&self, v: I) -> Node
    where
        I: IntoIterator<Item = bool>,
        I::IntoIter: DoubleEndedIterator + ExactSizeIterator,
    {
        let mut node = self.cache.true_node();
        let false_node = self.cache.false_node();
        for (i, b) in v.into_iter().enumerate().rev() {
            if b {
                node = self.cache.get_or_insert(i as u64, &node, &false_node);
            } else {
                node = self.cache.get_or_insert(i as u64, &false_node, &node);
            }
        }
        Node {
            inner: node,
            cache: Rc::clone(&self.cache),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Manager;

    #[test]
    fn test_minterm() {
        let dd = Manager::new();

        let node = dd.minterm(1, 3);

        assert_eq!(node.eval([false, false, false]), Some(false));
        assert_eq!(node.eval([true, false, false]), Some(false));
        assert_eq!(node.eval([false, true, false]), Some(true));
        assert_eq!(node.eval([true, true, false]), Some(false));
        assert_eq!(node.eval([false, false, true]), Some(false));
        assert_eq!(node.eval([true, false, true]), Some(false));
        assert_eq!(node.eval([false, true, true]), Some(false));
        assert_eq!(node.eval([true, true, true]), Some(false));
    }

    #[test]
    fn test_minterm_vec() {
        let dd = Manager::new();

        let node = dd.minterm_vec([true, false, true]);

        assert_eq!(node.eval([false, false, false]), Some(false));
        assert_eq!(node.eval([true, false, false]), Some(false));
        assert_eq!(node.eval([false, true, false]), Some(false));
        assert_eq!(node.eval([true, true, false]), Some(false));
        assert_eq!(node.eval([false, false, true]), Some(false));
        assert_eq!(node.eval([true, false, true]), Some(true));
        assert_eq!(node.eval([false, true, true]), Some(false));
        assert_eq!(node.eval([true, true, true]), Some(false));
    }
}
