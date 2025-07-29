use std::{fs, ops};

use chumsky::prelude::*;
use itertools::Itertools;
use snafu::Snafu;

use crate::{eval::Globals, parser::parse};

pub mod ast;
pub mod eval;
mod lexer;
pub mod parser;
pub mod relation;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(transparent)]
    IO { source: std::io::Error },
    #[snafu(transparent)]
    Parse { source: parser::Error },
}

type Span = SimpleSpan;
type Spanned<T> = (T, Span);

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

pub fn load_file(filename: &str, globals: &mut Globals) -> Result<(), Error> {
    let src = fs::read_to_string(filename)?;
    let program = parse(filename, &src)?;
    globals.extend(program.items);
    Ok(())
}
