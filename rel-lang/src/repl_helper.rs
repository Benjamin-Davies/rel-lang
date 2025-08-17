use std::{cell::RefCell, rc::Rc};

use rustyline::{
    Helper, Result,
    completion::{Completer, FilenameCompleter, Pair},
    highlight::Highlighter,
    hint::Hinter,
    validate::Validator,
};

use crate::{repl::Repl, repl_commands::Edge};

pub struct ReplHelper {
    filename_completer: FilenameCompleter,
    repl: Rc<RefCell<Repl>>,
}

impl ReplHelper {
    pub fn new(repl: Rc<RefCell<Repl>>) -> Self {
        Self {
            filename_completer: FilenameCompleter::new(),
            repl,
        }
    }

    fn complete_command(&self, line: &str, pos: usize) -> Result<(usize, Vec<Pair>)> {
        let repl = self.repl.borrow();

        let line = &line[..pos];
        let mut args = line.split_whitespace().collect::<Vec<_>>();
        // There is at-least one char because the line starts with a dot.
        if char::is_whitespace(line.chars().last().unwrap()) {
            // If the last character is whitespace, we should complete a new empty part.
            args.push("");
        }

        // There is at-least one part because the line starts with a dot.
        let Some(node) = repl.commands.traverse(&args[..args.len() - 1]) else {
            return Ok((pos, Vec::new()));
        };

        let last_part = args.last().unwrap();
        let last_part_start = line.rfind(last_part).unwrap_or(0);
        let mut suggestions = Vec::new();

        let keywords = node.next_keywords_by_prefix(last_part).map(|k| Pair {
            display: k.to_owned(),
            replacement: k.to_owned(),
        });
        suggestions.extend(keywords);

        if node.edges.contains_key(&Edge::Filename) {
            let (start, filenames) = self.filename_completer.complete_path_unsorted(line, pos)?;
            debug_assert_eq!(start, last_part_start);
            suggestions.extend(filenames);
        }

        if node.edges.contains_key(&Edge::Variable) {
            let variables = repl
                .state
                .locals
                .variables_by_prefix(last_part)
                .map(|var| Pair {
                    display: var.to_owned(),
                    replacement: var.to_owned(),
                });
            suggestions.extend(variables);
        }

        suggestions.sort_by(|a, b| a.display.cmp(&b.display));
        Ok((last_part_start, suggestions))
    }
}

impl Completer for ReplHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context,
    ) -> Result<(usize, Vec<Self::Candidate>)> {
        if line.starts_with('.') {
            self.complete_command(line, pos)
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
