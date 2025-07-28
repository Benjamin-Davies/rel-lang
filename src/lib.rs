use chumsky::prelude::*;

pub mod ast;
mod lexer;
pub mod parser;

pub type Span = SimpleSpan;
pub type Spanned<T> = (T, Span);
