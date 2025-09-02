use std::ops;

use rel_lang_dd as dd;

use crate::{Domain, Element, bits, bits2, dd_manager, iter_domain_product, num_vars};

#[derive(Clone)]
pub struct Relation {
    domain: (Domain, Domain),
    node: dd::Node,
}

impl Relation {
    pub fn empty(domain: (Domain, Domain)) -> Self {
        Self {
            domain,
            node: dd_manager().false_node(),
        }
    }

    pub fn identity(domain: Domain) -> Self {
        Self::sparse((domain, domain), (0..domain.end).map(|x| (x, x)))
    }

    pub fn universal(domain: (Domain, Domain)) -> Self {
        if domain.0.end == 0 || domain.1.end == 0 {
            return Self::empty(domain);
        }

        let dd = dd_manager();
        Self {
            domain: domain.clone(),
            node: dd.less_than_eq_vec(bits(domain.0.clone(), domain.0.end - 1))
                & dd.less_than_eq_vec(bits(domain.1.clone(), domain.1.end - 1))
                    .shift(num_vars(domain.0.clone()).into()),
        }
    }

    pub fn sparse(
        domain: (Domain, Domain),
        pairs: impl IntoIterator<Item = (Element, Element)>,
    ) -> Self {
        let (x_domain, y_domain) = &domain;
        let dd = dd_manager();

        let mut node = dd.false_node();
        for (x, y) in pairs {
            assert!(
                x_domain.contains(&x),
                "lhs of element {:?} is not in domain",
                (x, y),
            );
            assert!(
                y_domain.contains(&y),
                "rhs of element {:?} is not in domain",
                (x, y),
            );

            node |= dd.minterm_vec(bits2(domain, (x, y)));
        }
        Self { domain, node }
    }

    pub fn true_relation() -> Self {
        Self::universal((..1, ..1))
    }

    pub fn false_relation() -> Self {
        Self::empty((..1, ..1))
    }

    pub fn domain(&self) -> (Domain, Domain) {
        self.domain
    }

    pub fn converse(self) -> Self {
        let (x_domain, y_domain) = self.domain;

        let num_vars_x = num_vars(x_domain.clone());
        let num_vars_y = num_vars(y_domain.clone());

        Self {
            domain: (y_domain.clone(), x_domain.clone()),
            node: self.node.split_shift(
                num_vars_x.into(),
                num_vars_y.into(),
                -i64::from(num_vars_x),
            ),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.node.is_false()
    }

    pub fn is_subset_of(&self, other: &Self) -> bool {
        debug_assert_eq!(
            self.domain, other.domain,
            "domains {:?} and {:?} do not match",
            self.domain, other.domain,
        );
        self.node.implies(&other.node).is_true()
    }

    pub fn contains(&self, pair: (Element, Element)) -> bool {
        debug_assert_eq!(
            self.domain.0.contains(&pair.0),
            true,
            "lhs of element {:?} is not in domain {:?}",
            pair,
            self.domain.0
        );
        debug_assert_eq!(
            self.domain.1.contains(&pair.1),
            true,
            "rhs of element {:?} is not in domain {:?}",
            pair,
            self.domain.1
        );

        self.node.eval(bits2(self.domain, pair)).unwrap_or(false)
    }

    pub fn iter(&self) -> impl Iterator<Item = (Element, Element)> {
        iter_domain_product(self.domain.clone()).filter(move |&pair| self.contains(pair))
    }

    pub fn collapse_left(&self) -> Relation {
        Relation::sparse((self.domain.0, ..1), self.iter().map(|(x, _)| (x, 0)))
    }

    pub fn choose_one(&self) -> Relation {
        Relation::sparse(self.domain, self.iter().take(1))
    }
}

impl PartialEq for Relation {
    fn eq(&self, other: &Self) -> bool {
        debug_assert_eq!(
            self.domain, other.domain,
            "domains {:?} and {:?} do not match",
            self.domain, other.domain,
        );
        self.node == other.node
    }
}

impl Eq for Relation {}

impl ops::Neg for Relation {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            node: self.node ^ Self::universal(self.domain).node,
            ..self
        }
    }
}

impl ops::BitOr for Relation {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        debug_assert_eq!(
            self.domain, rhs.domain,
            "domains {:?} and {:?} do not match",
            self.domain, rhs.domain,
        );
        Self {
            node: self.node | rhs.node,
            ..self
        }
    }
}

impl ops::BitAnd for Relation {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        debug_assert_eq!(
            self.domain, rhs.domain,
            "domains {:?} and {:?} do not match",
            self.domain, rhs.domain,
        );
        Self {
            node: self.node & rhs.node,
            ..self
        }
    }
}

impl ops::Mul for Relation {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        debug_assert_eq!(
            self.domain.1, rhs.domain.0,
            "domains {:?} and {:?} cannot be multiplied",
            self.domain, rhs.domain,
        );

        // TODO: faster algorithm
        let new_domain = (self.domain.0.clone(), rhs.domain.1.clone());
        let inner_dim = self.domain.1.end;
        Self::sparse(
            new_domain.clone(),
            iter_domain_product(new_domain).filter(|&(i, k)| {
                (0..inner_dim).any(|j| self.contains((i, j)) && rhs.contains((j, k)))
            }),
        )
    }
}

impl From<bool> for Relation {
    fn from(value: bool) -> Self {
        if value {
            Self::true_relation()
        } else {
            Self::false_relation()
        }
    }
}
