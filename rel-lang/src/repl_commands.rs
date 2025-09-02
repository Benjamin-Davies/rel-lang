use std::{collections::BTreeMap, io, ops};

use crate::{load_file, load_matrix, load_relation, repl::State, save_matrix, save_relation};

const HELP_MESSAGE: &str = "Available commands:\n\
   .help - Show this help message\n\
   .exit - Exit the REPL\n\
   .store <variable> - Store the last result in a variable\n\
   .load prog <filename> - Load a program from a file\n\
   .load rel <variable> <filename> - Load a relation from a file\n\
   .load mat <variable> <filename> - Load a matrix from a file\n\
   .save rel <variable> <filename> - Save a relation to a file\n\
   .save mat <variable> <filename> - Save a matrix to a file";

/// The available commands form a rooted tree where the tokens are represented by edges.
#[derive(Default)]
pub struct Node {
    pub edges: BTreeMap<Edge, Node>,
    pub func: Option<CommandFn>,
}

type CommandFn =
    Box<dyn Fn(&mut State, &mut dyn io::Write, &[&str]) -> io::Result<ops::ControlFlow<()>>>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Edge {
    Keyword(&'static str),
    Filename,
    Variable,
}

impl Node {
    pub fn root() -> Self {
        let mut root = Node::default();

        root.insert(Edge::Keyword(".help"))
            .with_func(|_state, out, [_]| {
                writeln!(out, "{HELP_MESSAGE}")?;
                Ok(ops::ControlFlow::Continue(()))
            });
        root.insert(Edge::Keyword(".exit"))
            .with_func(|_state, _out, [_]| Ok(ops::ControlFlow::Break(())));
        root.insert(Edge::Keyword(".store"))
            .insert(Edge::Variable)
            .with_func(|state, out, [variable]| {
                if let Some(result) = &state.last_result {
                    state.locals.assign(variable, result.clone());
                    writeln!(out, "Stored in variable '{}'", variable)?;
                } else {
                    writeln!(out, "No last result to store")?;
                }
                Ok(ops::ControlFlow::Continue(()))
            });

        let load = root.insert(Edge::Keyword(".load"));
        load.insert(Edge::Keyword("prog"))
            .insert(Edge::Filename)
            .with_func(|state, out, [_, _, filename]| {
                match load_file(filename, &mut state.globals) {
                    Ok(()) => writeln!(out, "Program loaded successfully from '{filename}'")?,
                    Err(e) => writeln!(out, "Error loading program: {e}")?,
                }
                Ok(ops::ControlFlow::Continue(()))
            });
        load.insert(Edge::Keyword("rel"))
            .insert(Edge::Variable)
            .insert(Edge::Filename)
            .with_func(|state, out, [_, _, variable, filename]| {
                match load_relation(variable, filename, &mut state.locals) {
                    Ok(()) => writeln!(
                        out,
                        "Relation '{variable}' loaded successfully from '{filename}'"
                    )?,
                    Err(e) => writeln!(out, "Error loading relation: {e}")?,
                }
                Ok(ops::ControlFlow::Continue(()))
            });
        load.insert(Edge::Keyword("mat"))
            .insert(Edge::Variable)
            .insert(Edge::Filename)
            .with_func(|state, out, [_, _, variable, filename]| {
                match load_matrix(variable, filename, &mut state.locals) {
                    Ok(()) => writeln!(
                        out,
                        "Matrix '{variable}' loaded successfully from '{filename}'"
                    )?,
                    Err(e) => writeln!(out, "Error loading matrix: {e}")?,
                }
                Ok(ops::ControlFlow::Continue(()))
            });

        let save = root.insert(Edge::Keyword(".save"));
        save.insert(Edge::Keyword("rel"))
            .insert(Edge::Variable)
            .insert(Edge::Filename)
            .with_func(|state, out, [_, _, variable, filename]| {
                match save_relation(&state.locals, variable, filename) {
                    Ok(()) => writeln!(out, "Relation '{variable}' saved to '{filename}'")?,
                    Err(e) => writeln!(out, "Error saving relation: {e}")?,
                }
                Ok(ops::ControlFlow::Continue(()))
            });
        save.insert(Edge::Keyword("mat"))
            .insert(Edge::Variable)
            .insert(Edge::Filename)
            .with_func(|state, out, [_, _, variable, filename]| {
                match save_matrix(&state.locals, variable, filename) {
                    Ok(()) => writeln!(out, "Matrix '{variable}' saved to '{filename}'")?,
                    Err(e) => writeln!(out, "Error saving matrix: {e}")?,
                }
                Ok(ops::ControlFlow::Continue(()))
            });

        root
    }

    fn insert(&mut self, edge: Edge) -> &mut Self {
        self.edges.entry(edge).or_default()
    }

    fn with_func<const N: usize>(
        &mut self,
        func: impl Fn(&mut State, &mut dyn io::Write, [&str; N]) -> io::Result<ops::ControlFlow<()>>
        + 'static,
    ) {
        self.func = Some(Box::new(move |state, out, args| {
            func(state, out, args.try_into().unwrap())
        }));
    }

    pub fn traverse(&self, args: &[&str]) -> Option<&Self> {
        let mut node = self;
        for arg in args {
            if let Some(edge) = node.edges.keys().find(|e| {
                matches!(e, Edge::Keyword(k) if k.starts_with(arg))
                    || matches!(e, Edge::Variable | Edge::Filename)
            }) {
                node = &node.edges[edge];
            } else {
                return None;
            }
        }
        Some(node)
    }

    pub fn next_keywords_by_prefix(&self, prefix: &str) -> impl Iterator<Item = &str> {
        self.edges.keys().filter_map(move |edge| {
            if let &Edge::Keyword(keyword) = edge {
                if keyword.starts_with(prefix) {
                    Some(keyword)
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
}
