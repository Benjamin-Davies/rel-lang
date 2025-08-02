use std::{io, ops};

use crate::{
    eval::{Globals, Locals, eval},
    parser::parse_expr,
    relation::Relation,
    repl_commands::Node,
};

const WELCOME_MESSAGE: &str = "Welcome to the rel-lang REPL! Type '.help' for assistance.";

pub struct Repl {
    pub commands: Node,
    pub state: State,
}

pub struct State {
    pub globals: Globals,
    pub locals: Locals,
    pub last_result: Option<Relation>,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            commands: Node::root(),
            state: State {
                globals: Globals::default(),
                locals: Locals::default(),
                last_result: None,
            },
        }
    }

    pub fn welcome(&self, mut out: impl io::Write) -> io::Result<()> {
        writeln!(out, "{WELCOME_MESSAGE}")?;
        Ok(())
    }

    pub fn process_input(
        &mut self,
        line: &str,
        mut out: impl io::Write,
    ) -> io::Result<ops::ControlFlow<()>> {
        if line.is_empty() {
            return Ok(ops::ControlFlow::Continue(()));
        }
        if line.starts_with('.') {
            return self.process_command(line, &mut out);
        }

        let Ok(expr) = parse_expr(line) else {
            writeln!(out, "Error parsing expression")?;
            return Ok(ops::ControlFlow::Continue(()));
        };
        let result = eval(&self.state.globals, &self.state.locals, &expr);
        match result {
            Ok(value) => {
                // Relation::display already adds a newline.
                write!(out, "{}", value.display("<expr>"))?;
                self.state.last_result = Some(value);
            }
            Err(e) => writeln!(out, "Error: {e}")?,
        }

        Ok(ops::ControlFlow::Continue(()))
    }

    fn process_command(
        &mut self,
        command: &str,
        mut out: impl io::Write,
    ) -> io::Result<ops::ControlFlow<()>> {
        let args = command.split_whitespace().collect::<Vec<_>>();

        let Some(node) = self.commands.traverse(&args) else {
            writeln!(out, "Unknown command or invalid syntax")?;
            return Ok(ops::ControlFlow::Continue(()));
        };
        let Some(func) = &node.func else {
            writeln!(out, "Incomplete command")?;
            return Ok(ops::ControlFlow::Continue(()));
        };

        func(&mut self.state, &mut out, &args)
    }
}
