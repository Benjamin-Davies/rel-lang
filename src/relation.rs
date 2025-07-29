use std::{
    collections::{BTreeSet, btree_set},
    ops,
    rc::Rc,
};

use itertools::Itertools;

use crate::{Domain, Element, iter_domain, iter_domain_product};

#[derive(Debug, Clone)]
pub struct Relation {
    domain: (Domain, Domain),
    storage: Storage,
}

#[derive(Debug, Clone)]
enum Storage {
    Empty,
    Identity,
    Universal,
    Sparse(Rc<BTreeSet<(Element, Element)>>),
}

impl Relation {
    pub fn empty(domain: (Domain, Domain)) -> Self {
        Self {
            domain,
            storage: Storage::Empty,
        }
    }

    pub fn identity(domain: Domain) -> Self {
        if domain.end == 0 {
            return Self::empty((domain.clone(), domain));
        }
        Self {
            domain: (domain.clone(), domain),
            storage: Storage::Identity,
        }
    }

    pub fn universal(domain: (Domain, Domain)) -> Self {
        if domain.0.end == 0 || domain.1.end == 0 {
            return Self::empty(domain);
        }
        Self {
            domain,
            storage: Storage::Universal,
        }
    }

    pub fn sparse(
        domain: (Domain, Domain),
        pairs: impl IntoIterator<Item = (Element, Element)>,
    ) -> Self {
        let storage = Rc::new(
            pairs
                .into_iter()
                .inspect(|(x, y)| {
                    let (x_domain, y_domain) = &domain;
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
                })
                .collect(),
        );
        Self {
            domain,
            storage: Storage::Sparse(storage),
        }
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

    pub fn converse(mut self) -> Self {
        let (x_domain, y_domain) = self.domain;
        self.domain = (y_domain, x_domain);

        match self.storage {
            Storage::Empty | Storage::Identity | Storage::Universal => {}
            Storage::Sparse(ref storage) => {
                let new_storage = storage
                    .iter()
                    .map(|&(x, y)| (y, x))
                    .collect::<BTreeSet<_>>();
                self.storage = Storage::Sparse(Rc::new(new_storage));
            }
        }

        self
    }

    pub fn is_empty(&self) -> bool {
        match &self.storage {
            Storage::Empty => true,
            Storage::Identity => self.domain.0.end == 0,
            Storage::Universal => self.domain.0.end == 0 || self.domain.1.end == 0,
            Storage::Sparse(storage) => storage.is_empty(),
        }
    }

    pub fn is_subset_of(&self, other: &Self) -> bool {
        debug_assert_eq!(
            self.domain, other.domain,
            "domains {:?} and {:?} do not match",
            self.domain, other.domain,
        );
        match (&self.storage, &other.storage) {
            (Storage::Empty, _) => true,
            (Storage::Identity, Storage::Identity) => true,
            (_, Storage::Universal) => true,
            (_, _) => self.iter().all(|pair| other.contains(pair)),
        }
    }

    pub fn contains(&self, pair: (Element, Element)) -> bool {
        match &self.storage {
            Storage::Empty => false,
            Storage::Identity => pair.0 == pair.1,
            Storage::Universal => true,
            Storage::Sparse(storage) => storage.contains(&pair),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (Element, Element)> {
        enum Iter<'a> {
            Empty,
            Identity(ops::Range<u32>),
            Universal(itertools::Product<ops::Range<u32>, ops::Range<u32>>),
            Sparse(btree_set::Iter<'a, (Element, Element)>),
        }

        impl Iterator for Iter<'_> {
            type Item = (Element, Element);

            fn next(&mut self) -> Option<Self::Item> {
                match self {
                    Iter::Empty => None,
                    Iter::Identity(range) => range.next().map(|x| (x, x)),
                    Iter::Universal(product) => product.next(),
                    Iter::Sparse(iter) => iter.next().cloned(),
                }
            }
        }

        match &self.storage {
            Storage::Empty => Iter::Empty,
            Storage::Identity => Iter::Identity(0..self.domain.0.end),
            Storage::Universal => {
                Iter::Universal((0..self.domain.0.end).cartesian_product(0..self.domain.1.end))
            }
            Storage::Sparse(storage) => Iter::Sparse(storage.iter()),
        }
    }
}

impl PartialEq for Relation {
    fn eq(&self, other: &Self) -> bool {
        debug_assert_eq!(
            self.domain, other.domain,
            "domains {:?} and {:?} do not match",
            self.domain, other.domain,
        );
        match (&self.storage, &other.storage) {
            (Storage::Empty, Storage::Empty) => true,
            (Storage::Identity, Storage::Identity) => true,
            (Storage::Universal, Storage::Universal) => true,
            (Storage::Sparse(a), Storage::Sparse(b)) => a == b,
            (_, _) => self.is_subset_of(other) && other.is_subset_of(self),
        }
    }
}

impl Eq for Relation {}

impl ops::Neg for Relation {
    type Output = Self;

    fn neg(mut self) -> Self::Output {
        match self.storage {
            Storage::Empty => Self::universal(self.domain),
            Storage::Universal => Self::empty(self.domain),
            _ => {
                let new_storage = iter_domain_product(self.domain.clone())
                    .filter(|&(x, y)| !self.contains((x, y)))
                    .collect::<BTreeSet<_>>();
                self.storage = Storage::Sparse(Rc::new(new_storage));
                self
            }
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
        if self == rhs {
            return self;
        }
        match (&self.storage, &rhs.storage) {
            (Storage::Empty, _) => rhs,
            (_, Storage::Empty) => self,
            (Storage::Universal, _) | (_, Storage::Universal) => Self::universal(self.domain),
            _ => Self::sparse(self.domain, self.iter().chain(rhs.iter())),
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
        if self == rhs {
            return self;
        }
        match (&self.storage, &rhs.storage) {
            (Storage::Universal, _) => rhs,
            (_, Storage::Universal) => self,
            (Storage::Empty, _) | (_, Storage::Empty) => Self::empty(self.domain),
            _ => Self::sparse(self.domain, self.iter().filter(|pair| rhs.contains(*pair))),
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
        let new_domain = (self.domain.0, rhs.domain.1);
        match (&self.storage, &rhs.storage) {
            (Storage::Identity, _) => rhs,
            (_, Storage::Identity) => self,
            (Storage::Empty, _) | (_, Storage::Empty) => Self::empty(new_domain),
            (Storage::Universal, Storage::Universal) => Self::universal(new_domain),
            (Storage::Universal, Storage::Sparse(storage)) => Self::sparse(
                new_domain,
                storage
                    .iter()
                    .cartesian_product(iter_domain(self.domain.0))
                    .map(|(&(_x, y), i)| (i, y)),
            ),
            (Storage::Sparse(storage), Storage::Universal) => Self::sparse(
                new_domain,
                storage
                    .iter()
                    .cartesian_product(iter_domain(self.domain.1))
                    .map(|(&(x, _y), i)| (x, i)),
            ),
            (Storage::Sparse(a), Storage::Sparse(b)) => Self::sparse(
                new_domain,
                a.iter().flat_map(|&(x, y)| {
                    b.range((y, 0)..(y + 1, 0)).map(move |&(y2, z)| {
                        debug_assert_eq!(y, y2);
                        (x, z)
                    })
                }),
            ),
        }
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
