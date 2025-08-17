use core::cell::RefCell;

use alloc::{
    collections::btree_map::BTreeMap,
    rc::{Rc, Weak},
};

use crate::node::{Node, NodeKind};

#[derive(Debug, Clone)]
pub struct Manager {
    cache: Rc<Cache>,
    true_node: Rc<Node>,
    false_node: Rc<Node>,
}

#[derive(Debug)]
pub(crate) struct Cache {
    unique_cache: RefCell<BTreeMap<(CacheKey, CacheKey), Weak<Node>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CacheKey {
    ptr: *const Node,
}

impl Manager {
    pub fn new() -> Self {
        let cache = Rc::new(Cache::new());

        let true_node = Rc::new(Node {
            cache: Rc::clone(&cache),
            kind: NodeKind::True,
        });
        let false_node = Rc::new(Node {
            cache: Rc::clone(&cache),
            kind: NodeKind::False,
        });

        Self {
            cache,
            true_node,
            false_node,
        }
    }

    pub fn true_node(&self) -> Rc<Node> {
        Rc::clone(&self.true_node)
    }

    pub fn false_node(&self) -> Rc<Node> {
        Rc::clone(&self.false_node)
    }

    pub fn get_or_insert(&self, then_child: &Rc<Node>, else_child: &Rc<Node>) -> Rc<Node> {
        self.cache.get_or_insert(then_child, else_child)
    }
}

impl Cache {
    fn new() -> Self {
        Self {
            unique_cache: RefCell::new(BTreeMap::new()),
        }
    }

    pub(crate) fn get_or_insert(
        self: &Rc<Self>,
        then_child: &Rc<Node>,
        else_child: &Rc<Node>,
    ) -> Rc<Node> {
        if let Some(node) = self.get(then_child, else_child) {
            return node;
        }

        let new_node = Rc::new(Node {
            cache: Rc::clone(self),
            kind: NodeKind::NonTerminal {
                then_child: Rc::clone(then_child),
                else_child: Rc::clone(else_child),
            },
        });
        self.insert(then_child, else_child, &new_node);

        new_node
    }

    fn get(&self, then_child: &Rc<Node>, else_child: &Rc<Node>) -> Option<Rc<Node>> {
        let key = (CacheKey::from(then_child), CacheKey::from(else_child));
        let unique_cache = self.unique_cache.borrow();
        unique_cache.get(&key).and_then(Weak::upgrade)
    }

    fn insert(&self, then_child: &Rc<Node>, else_child: &Rc<Node>, node: &Rc<Node>) {
        let key = (CacheKey::from(then_child), CacheKey::from(else_child));
        let mut unique_cache = self.unique_cache.borrow_mut();
        unique_cache.insert(key, Rc::downgrade(node));
    }

    pub(crate) fn remove(&self, then_child: &Rc<Node>, else_child: &Rc<Node>) {
        let key = (CacheKey::from(then_child), CacheKey::from(else_child));
        let mut unique_cache = self.unique_cache.borrow_mut();
        unique_cache.remove(&key);
    }
}

impl From<&Rc<Node>> for CacheKey {
    fn from(node: &Rc<Node>) -> Self {
        Self { ptr: node.as_ref() }
    }
}
