use core::ops;

use alloc::rc::Rc;

use crate::{
    Node,
    manager::{Cache, CacheKey},
    node,
};

impl ops::BitAnd for Node {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            cache: Rc::clone(&self.cache),
            inner: and(&self.cache, &self.inner, &rhs.inner),
        }
    }
}

impl ops::BitOr for Node {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            cache: Rc::clone(&self.cache),
            inner: or(&self.cache, &self.inner, &rhs.inner),
        }
    }
}

impl ops::Not for Node {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self {
            cache: Rc::clone(&self.cache),
            inner: not(&self.cache, &self.inner),
        }
    }
}

fn and(cache: &Cache, lhs: &Rc<node::Inner>, rhs: &Rc<node::Inner>) -> Rc<node::Inner> {
    if CacheKey::from(lhs) == CacheKey::from(rhs) {
        return Rc::clone(lhs);
    }

    match (&lhs.kind, &rhs.kind) {
        (node::Kind::False, _) | (_, node::Kind::False) => cache.false_node(),
        (_, node::Kind::True) => Rc::clone(lhs),
        (node::Kind::True, _) => Rc::clone(rhs),
        (
            node::Kind::NonTerminal {
                then_child: lhs_then,
                else_child: lhs_else,
            },
            node::Kind::NonTerminal {
                then_child: rhs_then,
                else_child: rhs_else,
            },
        ) => {
            let new_then = and(cache, lhs_then, rhs_then);
            let new_else = and(cache, lhs_else, rhs_else);
            cache.get_or_insert(&new_then, &new_else)
        }
    }
}

fn or(cache: &Cache, lhs: &Rc<node::Inner>, rhs: &Rc<node::Inner>) -> Rc<node::Inner> {
    if CacheKey::from(lhs) == CacheKey::from(rhs) {
        return Rc::clone(lhs);
    }

    match (&lhs.kind, &rhs.kind) {
        (node::Kind::True, _) | (_, node::Kind::True) => cache.true_node(),
        (_, node::Kind::False) => Rc::clone(lhs),
        (node::Kind::False, _) => Rc::clone(rhs),
        (
            node::Kind::NonTerminal {
                then_child: lhs_then,
                else_child: lhs_else,
            },
            node::Kind::NonTerminal {
                then_child: rhs_then,
                else_child: rhs_else,
            },
        ) => {
            let new_then = or(cache, lhs_then, rhs_then);
            let new_else = or(cache, lhs_else, rhs_else);
            cache.get_or_insert(&new_then, &new_else)
        }
    }
}

fn not(cache: &Cache, node: &Rc<node::Inner>) -> Rc<node::Inner> {
    match &node.kind {
        node::Kind::True => cache.false_node(),
        node::Kind::False => cache.true_node(),
        node::Kind::NonTerminal {
            then_child,
            else_child,
        } => {
            let new_then = not(cache, then_child);
            let new_else = not(cache, else_child);
            cache.get_or_insert(&new_then, &new_else)
        }
    }
}
