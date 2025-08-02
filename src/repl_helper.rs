use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use rustyline::{
    Helper, Result,
    completion::{Completer, FilenameCompleter, Pair},
    highlight::Highlighter,
    hint::Hinter,
    validate::Validator,
};

use crate::repl::Repl;

pub struct ReplHelper {
    filename_completer: FilenameCompleter,
    commands: Node,
    repl: Rc<RefCell<Repl>>,
}

/// The available commands form a rooted tree where the tokens are represented by edges.
#[derive(Debug, Default)]
struct Node {
    edges: BTreeMap<Edge, Node>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Edge {
    Keyword(&'static str),
    Filename,
    Variable,
}

impl Node {
    fn root() -> Self {
        let mut root = Node::default();

        root.insert(Edge::Keyword(".help"));
        root.insert(Edge::Keyword(".exit"));
        root.insert(Edge::Keyword(".store")).insert(Edge::Variable);

        let load = root.insert(Edge::Keyword(".load"));
        load.insert(Edge::Keyword("prog")).insert(Edge::Filename);
        load.insert(Edge::Keyword("rel")).insert(Edge::Filename);
        load.insert(Edge::Keyword("mat")).insert(Edge::Filename);

        let save = root.insert(Edge::Keyword(".save"));
        save.insert(Edge::Keyword("rel"))
            .insert(Edge::Variable)
            .insert(Edge::Filename);
        save.insert(Edge::Keyword("mat"))
            .insert(Edge::Variable)
            .insert(Edge::Filename);

        root
    }

    fn insert(&mut self, edge: Edge) -> &mut Self {
        self.edges.entry(edge).or_default()
    }
}

impl ReplHelper {
    pub fn new(repl: Rc<RefCell<Repl>>) -> Self {
        Self {
            filename_completer: FilenameCompleter::new(),
            commands: Node::root(),
            repl,
        }
    }
}

impl Completer for ReplHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &rustyline::Context,
    ) -> Result<(usize, Vec<Self::Candidate>)> {
        if line.starts_with('.') {
            let line = &line[..pos];
            let mut parts = line.split_whitespace().collect::<Vec<_>>();
            // There is at-least one char because the line starts with a dot.
            if char::is_whitespace(line.chars().last().unwrap()) {
                // If the last character is whitespace, we should complete a new empty part.
                parts.push("");
            }

            let mut node = &self.commands;
            // There is at-least one part because the line starts with a dot.
            for part in &parts[..parts.len() - 1] {
                if let Some(edge) = node.edges.keys().find(|e| {
                    matches!(e, Edge::Keyword(k) if k == part)
                        || matches!(e, Edge::Variable | Edge::Filename)
                }) {
                    node = &node.edges[edge];
                } else {
                    return Ok((pos, Vec::new()));
                }
            }

            let last_part = parts.last().unwrap();
            let last_part_start = line.rfind(last_part).unwrap_or(0);
            let mut suggestions = Vec::new();

            let matching_keywords = node
                .edges
                .keys()
                .filter_map(|edge| {
                    if let &Edge::Keyword(keyword) = edge {
                        if keyword.starts_with(last_part) {
                            Some(Pair {
                                display: keyword.to_owned(),
                                replacement: keyword.to_owned(),
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            if !matching_keywords.is_empty() {
                suggestions.extend(matching_keywords);
            }

            if node.edges.contains_key(&Edge::Filename) {
                let (start, filenames) = self.filename_completer.complete(line, pos, ctx)?;
                debug_assert_eq!(start, last_part_start);
                suggestions.extend(filenames);
            }

            if node.edges.contains_key(&Edge::Variable) {
                let repl = self.repl.borrow();
                let variables = repl.locals.get_by_prefix(last_part).map(|var| Pair {
                    display: var.to_owned(),
                    replacement: var.to_owned(),
                });
                suggestions.extend(variables);
            }

            Ok((last_part_start, suggestions))
        } else {
            // TODO: Handle variable/function name completion
            Ok((0, Vec::new()))
        }
    }
}

impl Hinter for ReplHelper {
    type Hint = String;
}

impl Highlighter for ReplHelper {}

impl Validator for ReplHelper {}

impl Helper for ReplHelper {}
