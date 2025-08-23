use core::ops::{Deref, DerefMut};

use alloc::collections::btree_map::BTreeMap;

use crate::{
    Lock, Rc, Weak,
    node::{self, Kind, Node},
};

#[derive(Debug, Clone)]
pub struct Manager {
    pub(crate) cache: Rc<Cache>,
}

#[derive(Debug)]
pub(crate) struct Cache {
    true_node: Rc<node::Inner>,
    false_node: Rc<node::Inner>,
    unique_cache: Lock<UniqueCache>,
}

pub(crate) type UniqueCache = BTreeMap<(u64, CacheKey, CacheKey), Weak<node::Inner>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct CacheKey {
    ptr: *const node::Inner,
}

#[cfg(feature = "sync")]
unsafe impl Send for CacheKey {}

#[cfg(feature = "sync")]
unsafe impl Sync for CacheKey {}

impl Manager {
    pub fn new() -> Self {
        Self {
            cache: Rc::new(Cache::new()),
        }
    }

    pub fn true_node(&self) -> Node {
        Node {
            cache: Rc::clone(&self.cache),
            inner: Rc::clone(&self.cache.true_node),
        }
    }

    pub fn false_node(&self) -> Node {
        Node {
            cache: Rc::clone(&self.cache),
            inner: Rc::clone(&self.cache.false_node),
        }
    }

    pub fn get_or_insert(&self, level: u64, then_child: &Node, else_child: &Node) -> Node {
        Node {
            cache: Rc::clone(&self.cache),
            inner: self
                .cache
                .get_or_insert(level, &then_child.inner, &else_child.inner),
        }
    }
}

impl Default for Manager {
    fn default() -> Self {
        Self::new()
    }
}

impl Cache {
    fn new() -> Self {
        Self {
            unique_cache: Lock::new(BTreeMap::new()),
            true_node: Rc::new(node::Inner { kind: Kind::True }),
            false_node: Rc::new(node::Inner { kind: Kind::False }),
        }
    }

    pub(crate) fn true_node(&self) -> Rc<node::Inner> {
        Rc::clone(&self.true_node)
    }

    pub(crate) fn false_node(&self) -> Rc<node::Inner> {
        Rc::clone(&self.false_node)
    }

    pub(crate) fn get_or_insert(
        &self,
        level: u64,
        then_child: &Rc<node::Inner>,
        else_child: &Rc<node::Inner>,
    ) -> Rc<node::Inner> {
        if Rc::ptr_eq(then_child, else_child) {
            return Rc::clone(then_child);
        }

        if let Some(node) = self.get(level, then_child, else_child) {
            return node;
        }

        let new_node = Rc::new(node::Inner {
            kind: Kind::NonTerminal {
                level,
                then_child: Rc::clone(then_child),
                else_child: Rc::clone(else_child),
            },
        });
        self.insert(level, then_child, else_child, &new_node);

        new_node
    }

    fn get(
        &self,
        level: u64,
        then_child: &Rc<node::Inner>,
        else_child: &Rc<node::Inner>,
    ) -> Option<Rc<node::Inner>> {
        let key = (
            level,
            CacheKey::from(then_child),
            CacheKey::from(else_child),
        );
        let unique_cache = self.unique_cache();
        unique_cache.get(&key).and_then(Weak::upgrade)
    }

    fn insert(
        &self,
        level: u64,
        then_child: &Rc<node::Inner>,
        else_child: &Rc<node::Inner>,
        node: &Rc<node::Inner>,
    ) {
        let key = (
            level,
            CacheKey::from(then_child),
            CacheKey::from(else_child),
        );
        let mut unique_cache = self.unique_cache_mut();
        unique_cache.insert(key, Rc::downgrade(node));
    }

    pub(crate) fn remove(
        &self,
        level: u64,
        then_child: &Rc<node::Inner>,
        else_child: &Rc<node::Inner>,
    ) {
        let key = (
            level,
            CacheKey::from(then_child),
            CacheKey::from(else_child),
        );
        let mut unique_cache = self.unique_cache_mut();
        unique_cache.remove(&key);
    }

    fn unique_cache(&self) -> impl Deref<Target = UniqueCache> {
        #[cfg(not(feature = "sync"))]
        return self.unique_cache.borrow();
        #[cfg(feature = "sync")]
        return self.unique_cache.read().unwrap();
    }

    fn unique_cache_mut(&self) -> impl DerefMut<Target = UniqueCache> {
        #[cfg(not(feature = "sync"))]
        return self.unique_cache.borrow_mut();
        #[cfg(feature = "sync")]
        return self.unique_cache.write().unwrap();
    }
}

impl From<&Rc<node::Inner>> for CacheKey {
    fn from(node: &Rc<node::Inner>) -> Self {
        Self { ptr: node.as_ref() }
    }
}
