use crate::{Manager, Node, Rc, manager::Cache, node};

impl Manager {
    /// Returns a node for the function: `f(b) = b[i]`.
    pub fn bit(&self, i: u64) -> Node {
        Node {
            cache: Rc::clone(&self.cache),
            inner: bit(&self.cache, i),
        }
    }

    /// Returns a node for the function: `f(b) = all(if j = i then b[j] else !b[j] for j < n)`.
    pub fn minterm(&self, i: u64, n: u64) -> Node {
        assert!(i < n);

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

    /// Returns a node for the function that ensures its argument (when interpreted as a big-endian
    /// uint) is less than `v`.
    pub fn less_than_eq_vec<I>(&self, v: I) -> Node
    where
        I: IntoIterator<Item = bool>,
        I::IntoIter: DoubleEndedIterator + ExactSizeIterator,
    {
        let mut node = self.cache.true_node();
        let true_node = self.cache.true_node();
        let false_node = self.cache.false_node();
        for (i, b) in v.into_iter().enumerate().rev() {
            if b {
                node = self.cache.get_or_insert(i as u64, &node, &true_node);
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

pub(crate) fn bit(cache: &Rc<Cache>, i: u64) -> Rc<node::Inner> {
    let then_child = cache.true_node();
    let else_child = cache.false_node();
    cache.get_or_insert(i, &then_child, &else_child)
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
