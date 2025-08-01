use std::{fs, ops};

use itertools::Itertools;
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

pub fn load_file(filename: &str, globals: &mut Globals) -> Result<(), Error> {
    let src = fs::read_to_string(filename)?;
    let program = parse_program(filename, &src)?;
    globals.extend(program.items);
    Ok(())
}

pub fn load_relation(filename: &str, locals: &mut Locals) -> Result<String, Error> {
    let src = fs::read_to_string(filename)?;
    let (name, relation) = parse_relation(filename, &src)?;
    locals.assign(&name, relation);
    Ok(name)
}

pub fn load_matrix(filename: &str, locals: &mut Locals) -> Result<String, Error> {
    let src = fs::read_to_string(filename)?;
    let (name, matrix) = parser::matrix::parse_matrix(filename, &src)?;
    locals.assign(&name, matrix);
    Ok(name)
}

pub fn save_relation(locals: &Locals, name: &str, filename: &str) -> Result<(), Error> {
    let relation = locals.get(name)?;
    fs::write(filename, relation.display(name).to_string())?;
    Ok(())
}

pub fn save_matrix(locals: &Locals, name: &str, filename: &str) -> Result<(), Error> {
    let relation = locals.get(name)?;
    fs::write(filename, relation.display_matrix(name).to_string())?;
    Ok(())
}
