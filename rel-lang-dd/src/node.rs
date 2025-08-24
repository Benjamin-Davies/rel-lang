use crate::{Rc, Weak, manager::Cache};

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
        // Only used in the `Drop` implementation to remove the node from the cache.
        // Does not need to be weak, but we're less likely to leak memory later if we use a weak reference.
        // This is only present for this variant, otherwise the true and false singletons would need to be lazily initialized.
        cache: Weak<Cache>,
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

impl Drop for Inner {
    fn drop(&mut self) {
        match &self.kind {
            Kind::True | Kind::False => {
                // No cleanup needed.
            }
            Kind::NonTerminal {
                level,
                then_child,
                else_child,
                cache,
            } => {
                if let Some(cache) = cache.upgrade() {
                    cache.remove(*level, then_child, else_child);
                }
            }
        }
    }
}
