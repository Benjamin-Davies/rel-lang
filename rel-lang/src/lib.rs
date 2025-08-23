use std::{fs, ops, sync::OnceLock};

use itertools::Itertools;
use rel_lang_dd as dd;
use snafu::Snafu;

use crate::{
    eval::{Globals, Locals},
    parser::{parse_program, relation::parse_relation},
};

pub mod ast;
pub mod display;
pub mod eval;
pub mod parser;
pub mod relation;
pub mod repl;
pub mod repl_commands;
pub mod repl_helper;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(transparent)]
    IO { source: std::io::Error },
    #[snafu(transparent)]
    Parse { source: parser::Error },
    #[snafu(transparent)]
    Eval { source: eval::Error },
}

pub type Element = u32;
pub type Domain = ops::RangeTo<Element>;

fn iter_domain(domain: Domain) -> impl Iterator<Item = Element> + Clone {
    0..domain.end
}

fn iter_domain_product(domain: (Domain, Domain)) -> impl Iterator<Item = (Element, Element)> {
    let x_iter = iter_domain(domain.0);
    let y_iter = iter_domain(domain.1);
    x_iter.cartesian_product(y_iter)
}

fn num_vars(domain: Domain) -> u32 {
    if domain.end == 0 {
        0
    } else {
        let max = domain.end - 1;
        Element::BITS - max.leading_zeros()
    }
}

/// Big-endian bit representation of an element in the given domain.
fn bits(
    domain: Domain,
    n: Element,
) -> impl Iterator<Item = bool> + DoubleEndedIterator + ExactSizeIterator {
    (0..num_vars(domain))
        .rev()
        .map(move |i| (n & (1u32 << i)) != 0)
}

fn bits2(
    domain: (Domain, Domain),
    pair: (Element, Element),
) -> impl Iterator<Item = bool> + DoubleEndedIterator + ExactSizeIterator {
    chain_exact(bits(domain.0, pair.0), bits(domain.1, pair.1))
}

fn chain_exact<T, I, J>(
    i: I,
    j: J,
) -> impl Iterator<Item = T> + DoubleEndedIterator + ExactSizeIterator
where
    I: Iterator<Item = T> + DoubleEndedIterator + ExactSizeIterator,
    J: Iterator<Item = T> + DoubleEndedIterator + ExactSizeIterator,
{
    struct ChainExact<I, J> {
        i: I,
        j: J,
    }

    impl<T, I, J> Iterator for ChainExact<I, J>
    where
        I: Iterator<Item = T>,
        J: Iterator<Item = T>,
    {
        type Item = T;

        fn next(&mut self) -> Option<Self::Item> {
            self.i.next().or_else(|| self.j.next())
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            let (i_lower, i_upper) = self.i.size_hint();
            let (j_lower, j_upper) = self.j.size_hint();
            (
                i_lower + j_lower,
                i_upper.and_then(|i| j_upper.map(|j| i + j)),
            )
        }
    }

    impl<T, I, J> DoubleEndedIterator for ChainExact<I, J>
    where
        I: DoubleEndedIterator<Item = T>,
        J: DoubleEndedIterator<Item = T>,
    {
        fn next_back(&mut self) -> Option<Self::Item> {
            self.j.next_back().or_else(|| self.i.next_back())
        }
    }

    impl<T, I, J> ExactSizeIterator for ChainExact<I, J>
    where
        I: ExactSizeIterator<Item = T>,
        J: ExactSizeIterator<Item = T>,
    {
        fn len(&self) -> usize {
            self.i.len() + self.j.len()
        }
    }

    ChainExact {
        i: i.into_iter(),
        j: j.into_iter(),
    }
}

fn dd_manager() -> &'static dd::Manager {
    static MANAGER: OnceLock<dd::Manager> = OnceLock::new();

    MANAGER.get_or_init(dd::Manager::new)
}

pub fn load_file(filename: &str, globals: &mut Globals) -> Result<(), Error> {
    let src = fs::read_to_string(filename)?;
    let program = parse_program(filename, &src)?;
    globals.extend(program.items);
    Ok(())
}

pub fn load_relation(variable: &str, filename: &str, locals: &mut Locals) -> Result<(), Error> {
    let src = fs::read_to_string(filename)?;
    let (_name, relation) = parse_relation(filename, &src)?;
    locals.assign(variable, relation);
    Ok(())
}

pub fn load_matrix(variable: &str, filename: &str, locals: &mut Locals) -> Result<(), Error> {
    let src = fs::read_to_string(filename)?;
    let matrix = parser::matrix::parse_matrix(filename, &src)?;
    locals.assign(variable, matrix);
    Ok(())
}

pub fn save_relation(locals: &Locals, name: &str, filename: &str) -> Result<(), Error> {
    let relation = locals.get(name)?;
    fs::write(filename, relation.display(name).to_string())?;
    Ok(())
}

pub fn save_matrix(locals: &Locals, name: &str, filename: &str) -> Result<(), Error> {
    let relation = locals.get(name)?;
    fs::write(filename, relation.display_matrix().to_string())?;
    Ok(())
}
