use crate::{Rc, manager::Cache};

#[derive(Debug, Clone)]
pub struct Node {
    pub(crate) cache: Rc<Cache>,
    pub(crate) inner: Rc<Inner>,
}

#[derive(Debug)]
pub(crate) struct Inner {
    pub(crate) kind: Kind,
}

#[derive(Debug)]
pub(crate) enum Kind {
    True,
    False,
    NonTerminal {
        // level == variable index (i.e. variables are not permuted)
        level: u64,
        then_child: Rc<Inner>,
        else_child: Rc<Inner>,
    },
}

impl Node {
    pub fn is_true(&self) -> bool {
        matches!(self.inner.kind, Kind::True)
    }

    pub fn is_false(&self) -> bool {
        matches!(self.inner.kind, Kind::False)
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        if Rc::strong_count(&self.inner) == 1 {
            // We are the last owner of this node, so we should remove it from the cache.
            match &self.inner.kind {
                Kind::True | Kind::False => {
                    // No cleanup needed.
                }
                Kind::NonTerminal {
                    level,
                    then_child,
                    else_child,
                } => {
                    self.cache.remove(*level, then_child, else_child);
                }
            }
        }
    }
}
