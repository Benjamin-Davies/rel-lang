use core::{cmp::Ordering, ops};

use crate::Rc;

use crate::{
    Node,
    manager::{Cache, CacheKey},
    node,
};

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        eq(&self.inner, &other.inner)
    }
}

impl Eq for Node {}

impl ops::BitAnd for Node {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            cache: Rc::clone(&self.cache),
            inner: and(&self.cache, &self.inner, &rhs.inner),
        }
    }
}

impl ops::BitAndAssign for Node {
    fn bitand_assign(&mut self, rhs: Self) {
        self.inner = and(&self.cache, &self.inner, &rhs.inner);
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

impl ops::BitOrAssign for Node {
    fn bitor_assign(&mut self, rhs: Self) {
        self.inner = or(&self.cache, &self.inner, &rhs.inner);
    }
}

impl ops::BitXor for Node {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self {
            cache: Rc::clone(&self.cache),
            inner: xor(&self.cache, &self.inner, &rhs.inner),
        }
    }
}

impl ops::BitXorAssign for Node {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.inner = xor(&self.cache, &self.inner, &rhs.inner);
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

impl Node {
    /// Returns a new node representing the function `f -> g`.
    pub fn implies(&self, rhs: &Self) -> Self {
        Self {
            cache: Rc::clone(&self.cache),
            inner: implies(&self.cache, &self.inner, &rhs.inner),
        }
    }

    /// Returns a new node representing the function `if f then g else h`.
    pub fn if_then_else(&self, then_value: &Self, else_value: &Self) -> Self {
        Self {
            cache: Rc::clone(&self.cache),
            inner: if_then_else(
                &self.cache,
                &self.inner,
                &then_value.inner,
                &else_value.inner,
            ),
        }
    }
}

fn eq(lhs: &Rc<node::Inner>, rhs: &Rc<node::Inner>) -> bool {
    if CacheKey::from(lhs) == CacheKey::from(rhs) {
        return true;
    }

    match (&lhs.kind, &rhs.kind) {
        (node::Kind::True, node::Kind::True) => true,
        (node::Kind::False, node::Kind::False) => true,
        (
            node::Kind::NonTerminal {
                level: lhs_level,
                then_child: lhs_then,
                else_child: lhs_else,
                cache: _,
            },
            node::Kind::NonTerminal {
                level: rhs_level,
                then_child: rhs_then,
                else_child: rhs_else,
                cache: _,
            },
        ) => lhs_level == rhs_level && eq(lhs_then, rhs_then) && eq(lhs_else, rhs_else),
        _ => false,
    }
}

fn and(cache: &Rc<Cache>, lhs: &Rc<node::Inner>, rhs: &Rc<node::Inner>) -> Rc<node::Inner> {
    if CacheKey::from(lhs) == CacheKey::from(rhs) {
        return Rc::clone(lhs);
    }

    match (&lhs.kind, &rhs.kind) {
        (node::Kind::False, _) | (_, node::Kind::False) => cache.false_node(),
        (_, node::Kind::True) => Rc::clone(lhs),
        (node::Kind::True, _) => Rc::clone(rhs),
        (
            node::Kind::NonTerminal {
                level: lhs_level,
                then_child: lhs_then,
                else_child: lhs_else,
                cache: _,
            },
            node::Kind::NonTerminal {
                level: rhs_level,
                then_child: rhs_then,
                else_child: rhs_else,
                cache: _,
            },
        ) => match lhs_level.cmp(rhs_level) {
            Ordering::Less => {
                // Only split the lhs node (which is evaluated first)
                let new_then = and(cache, lhs_then, rhs);
                let new_else = and(cache, lhs_else, rhs);
                cache.get_or_insert(*lhs_level, &new_then, &new_else)
            }
            Ordering::Equal => {
                // Split both nodes (they are evaluated in parallel)
                let new_then = and(cache, lhs_then, rhs_then);
                let new_else = and(cache, lhs_else, rhs_else);
                cache.get_or_insert(*lhs_level, &new_then, &new_else)
            }
            Ordering::Greater => {
                // Only split the rhs node (which is evaluated first)
                let new_then = and(cache, lhs, rhs_then);
                let new_else = and(cache, lhs, rhs_else);
                cache.get_or_insert(*rhs_level, &new_then, &new_else)
            }
        },
    }
}

fn or(cache: &Rc<Cache>, lhs: &Rc<node::Inner>, rhs: &Rc<node::Inner>) -> Rc<node::Inner> {
    if CacheKey::from(lhs) == CacheKey::from(rhs) {
        return Rc::clone(lhs);
    }

    match (&lhs.kind, &rhs.kind) {
        (node::Kind::True, _) | (_, node::Kind::True) => cache.true_node(),
        (_, node::Kind::False) => Rc::clone(lhs),
        (node::Kind::False, _) => Rc::clone(rhs),
        (
            node::Kind::NonTerminal {
                level: lhs_level,
                then_child: lhs_then,
                else_child: lhs_else,
                cache: _,
            },
            node::Kind::NonTerminal {
                level: rhs_level,
                then_child: rhs_then,
                else_child: rhs_else,
                cache: _,
            },
        ) => match lhs_level.cmp(rhs_level) {
            Ordering::Less => {
                // Only split the lhs node (which is evaluated first)
                let new_then = or(cache, lhs_then, rhs);
                let new_else = or(cache, lhs_else, rhs);
                cache.get_or_insert(*lhs_level, &new_then, &new_else)
            }
            Ordering::Equal => {
                // Split both nodes (they are evaluated in parallel)
                let new_then = or(cache, lhs_then, rhs_then);
                let new_else = or(cache, lhs_else, rhs_else);
                cache.get_or_insert(*lhs_level, &new_then, &new_else)
            }
            Ordering::Greater => {
                // Only split the rhs node (which is evaluated first)
                let new_then = or(cache, lhs, rhs_then);
                let new_else = or(cache, lhs, rhs_else);
                cache.get_or_insert(*rhs_level, &new_then, &new_else)
            }
        },
    }
}

fn xor(cache: &Rc<Cache>, lhs: &Rc<node::Inner>, rhs: &Rc<node::Inner>) -> Rc<node::Inner> {
    if CacheKey::from(lhs) == CacheKey::from(rhs) {
        return cache.false_node();
    }

    match (&lhs.kind, &rhs.kind) {
        (node::Kind::True, _) => not(cache, rhs),
        (node::Kind::False, _) => Rc::clone(rhs),
        (_, node::Kind::True) => not(cache, lhs),
        (_, node::Kind::False) => Rc::clone(lhs),
        (
            node::Kind::NonTerminal {
                level: lhs_level,
                then_child: lhs_then,
                else_child: lhs_else,
                cache: _,
            },
            node::Kind::NonTerminal {
                level: rhs_level,
                then_child: rhs_then,
                else_child: rhs_else,
                cache: _,
            },
        ) => match lhs_level.cmp(rhs_level) {
            Ordering::Less => {
                // Only split the lhs node (which is evaluated first)
                let new_then = xor(cache, lhs_then, rhs);
                let new_else = xor(cache, lhs_else, rhs);
                cache.get_or_insert(*lhs_level, &new_then, &new_else)
            }
            Ordering::Equal => {
                // Split both nodes (they are evaluated in parallel)
                let new_then = xor(cache, lhs_then, rhs_then);
                let new_else = xor(cache, lhs_else, rhs_else);
                cache.get_or_insert(*lhs_level, &new_then, &new_else)
            }
            Ordering::Greater => {
                // Only split the rhs node (which is evaluated first)
                let new_then = xor(cache, lhs, rhs_then);
                let new_else = xor(cache, lhs, rhs_else);
                cache.get_or_insert(*rhs_level, &new_then, &new_else)
            }
        },
    }
}

fn not(cache: &Rc<Cache>, node: &Rc<node::Inner>) -> Rc<node::Inner> {
    match &node.kind {
        node::Kind::True => cache.false_node(),
        node::Kind::False => cache.true_node(),
        node::Kind::NonTerminal {
            level,
            then_child,
            else_child,
            cache: _,
        } => {
            let new_then = not(cache, then_child);
            let new_else = not(cache, else_child);
            cache.get_or_insert(*level, &new_then, &new_else)
        }
    }
}

fn implies(cache: &Rc<Cache>, lhs: &Rc<node::Inner>, rhs: &Rc<node::Inner>) -> Rc<node::Inner> {
    if CacheKey::from(lhs) == CacheKey::from(rhs) {
        return cache.true_node();
    }

    match (&lhs.kind, &rhs.kind) {
        (node::Kind::False, _) | (_, node::Kind::True) => cache.true_node(),
        (_, node::Kind::False) => not(cache, lhs),
        (node::Kind::True, _) => Rc::clone(rhs),
        (
            node::Kind::NonTerminal {
                level: lhs_level,
                then_child: lhs_then,
                else_child: lhs_else,
                cache: _,
            },
            node::Kind::NonTerminal {
                level: rhs_level,
                then_child: rhs_then,
                else_child: rhs_else,
                cache: _,
            },
        ) => match lhs_level.cmp(rhs_level) {
            Ordering::Less => {
                // Only split the lhs node (which is evaluated first)
                let new_then = implies(cache, lhs_then, rhs);
                let new_else = implies(cache, lhs_else, rhs);
                cache.get_or_insert(*lhs_level, &new_then, &new_else)
            }
            Ordering::Equal => {
                // Split both nodes (they are evaluated in parallel)
                let new_then = implies(cache, lhs_then, rhs_then);
                let new_else = implies(cache, lhs_else, rhs_else);
                cache.get_or_insert(*lhs_level, &new_then, &new_else)
            }
            Ordering::Greater => {
                // Only split the rhs node (which is evaluated first)
                let new_then = implies(cache, lhs, rhs_then);
                let new_else = implies(cache, lhs, rhs_else);
                cache.get_or_insert(*rhs_level, &new_then, &new_else)
            }
        },
    }
}

pub(crate) fn if_then_else(
    cache: &Rc<Cache>,
    f: &Rc<node::Inner>,
    g: &Rc<node::Inner>,
    h: &Rc<node::Inner>,
) -> Rc<node::Inner> {
    match (&f.kind, &g.kind, &h.kind) {
        (node::Kind::True, _, _) => Rc::clone(g),
        (node::Kind::False, _, _) => Rc::clone(h),
        (_, node::Kind::True, _) => or(cache, f, h),
        (_, node::Kind::False, _) => and(cache, &not(cache, f), h),
        (_, _, node::Kind::True) => or(cache, &not(cache, f), g),
        (_, _, node::Kind::False) => and(cache, f, g),
        (
            node::Kind::NonTerminal {
                level: f_level,
                then_child: f_then,
                else_child: f_else,
                cache: _,
            },
            node::Kind::NonTerminal {
                level: g_level,
                then_child: g_then,
                else_child: g_else,
                cache: _,
            },
            node::Kind::NonTerminal {
                level: h_level,
                then_child: h_then,
                else_child: h_else,
                cache: _,
            },
        ) => {
            let min_level = f_level.min(g_level).min(h_level);

            let (fv, fnv) = if f_level == min_level {
                (f_then, f_else)
            } else {
                (f, f)
            };
            let (gv, gnv) = if g_level == min_level {
                (g_then, g_else)
            } else {
                (g, g)
            };
            let (hv, hnv) = if h_level == min_level {
                (h_then, h_else)
            } else {
                (h, h)
            };

            let new_then = if_then_else(cache, fv, gv, hv);
            let new_else = if_then_else(cache, fnv, gnv, hnv);
            cache.get_or_insert(*min_level, &new_then, &new_else)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Manager, Rc};

    #[test]
    fn test_not() {
        let dd = Manager::new();
        let a = dd.get_or_insert(1, &dd.true_node(), &dd.false_node());
        let b = dd.get_or_insert(2, &dd.false_node(), &dd.true_node());
        let root = dd.get_or_insert(0, &a, &b);

        let root = !root;

        assert_eq!(root.eval([false, false, false]), Some(false));
        assert_eq!(root.eval([false, false, true]), Some(true));
        assert_eq!(root.eval([false, true, false]), Some(false));
        assert_eq!(root.eval([false, true, true]), Some(true));
        assert_eq!(root.eval([true, false, false]), Some(true));
        assert_eq!(root.eval([true, false, true]), Some(true));
        assert_eq!(root.eval([true, true, false]), Some(false));
        assert_eq!(root.eval([true, true, true]), Some(false));

        assert_eq!(root.eval([]), None);
        assert_eq!(root.eval([false; 2]), None);
        assert_eq!(root.eval([true; 2]), Some(false));
        assert_eq!(root.eval([false; 4]), Some(false));
    }

    #[test]
    fn test_and_or() {
        let dd = Manager::new();
        let root = (dd.bit(0) | dd.bit(2)) & (dd.bit(1) | dd.bit(3));

        assert_eq!(root.eval([false, false, false, false]), Some(false));
        assert_eq!(root.eval([true, false, false, false]), Some(false));
        assert_eq!(root.eval([false, true, false, false]), Some(false));
        assert_eq!(root.eval([true, true, false, false]), Some(true));
        assert_eq!(root.eval([false, false, true, false]), Some(false));
        assert_eq!(root.eval([true, false, true, false]), Some(false));
        assert_eq!(root.eval([false, true, true, false]), Some(true));
        assert_eq!(root.eval([true, true, true, false]), Some(true));
        assert_eq!(root.eval([false, false, false, true]), Some(false));
        assert_eq!(root.eval([true, false, false, true]), Some(true));
        assert_eq!(root.eval([false, true, false, true]), Some(false));
        assert_eq!(root.eval([true, true, false, true]), Some(true));
        assert_eq!(root.eval([false, false, true, true]), Some(true));
        assert_eq!(root.eval([true, false, true, true]), Some(true));
        assert_eq!(root.eval([false, true, true, true]), Some(true));
        assert_eq!(root.eval([true, true, true, true]), Some(true));
    }

    #[test]
    fn test_and_idempotent() {
        let dd = Manager::new();
        let a = dd.bit(0);

        let result = a.clone() & a.clone();

        assert!(Rc::ptr_eq(&result.inner, &a.inner));
    }

    #[test]
    fn test_or_idempotent() {
        let dd = Manager::new();
        let a = dd.bit(0);

        let result = a.clone() | a.clone();

        assert!(Rc::ptr_eq(&result.inner, &a.inner));
    }

    #[test]
    fn test_or_minterms() {
        let dd = Manager::new();

        let result = dd.minterm(0, 3) | dd.minterm(2, 3);

        assert_eq!(result.eval([false, false, false]), Some(false));
        assert_eq!(result.eval([true, false, false]), Some(true));
        assert_eq!(result.eval([false, true, false]), Some(false));
        assert_eq!(result.eval([true, true, false]), Some(false));
        assert_eq!(result.eval([false, false, true]), Some(true));
        assert_eq!(result.eval([true, false, true]), Some(false));
        assert_eq!(result.eval([false, true, true]), Some(false));
        assert_eq!(result.eval([true, true, true]), Some(false));
    }
}
