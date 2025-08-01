use rustyline::{
    Helper, Result, completion::Completer, highlight::Highlighter, hint::Hinter,
    validate::Validator,
};

use crate::repl::COMMANDS;

pub struct ReplHelper;

impl Completer for ReplHelper {
    type Candidate = String;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context,
    ) -> Result<(usize, Vec<Self::Candidate>)> {
        if line.starts_with('.') {
            if !line[..pos].contains(' ') {
                let candidates = COMMANDS
                    .iter()
                    .filter(|cmd| cmd.starts_with(&line[1..pos]))
                    .map(|&cmd| cmd.to_owned())
                    .collect::<Vec<_>>();
                Ok((1, candidates))
            } else {
                // TODO: Handle command arguments
                Ok((0, Vec::new()))
            }
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
