use std::{io, ops};

use crate::{
    eval::{Globals, Locals, eval},
    load_file, load_matrix, load_relation,
    parser::parse_expr,
    relation::Relation,
    save_matrix, save_relation,
};

const WELCOME_MESSAGE: &str = "Welcome to the rel-lang REPL! Type '.help' for assistance.";

const HELP_MESSAGE: &str = "Available commands:\n\
    .help - Show this help message\n\
    .exit - Exit the REPL\n\
    .store <variable> - Store the last result in a variable\n\
    .load prog <filename> - Load a program from a file\n\
    .load rel <filename> - Load a relation from a file\n\
    .load mat <filename> - Load a matrix from a file\n\
    .save rel <variable> <filename> - Save a relation to a file\n\
    .save mat <variable> <filename> - Save a matrix to a file";

pub const COMMANDS: &[&str] = &["help", "exit", "clear", "load", "save"];

pub struct Repl {
    globals: Globals,
    locals: Locals,
    last_result: Option<Relation>,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            globals: Globals::default(),
            locals: Locals::default(),
            last_result: None,
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
        if let Some(command) = line.strip_prefix('.') {
            return self.process_command(command, &mut out);
        }

        let Ok(expr) = parse_expr(line) else {
            writeln!(out, "Error parsing expression")?;
            return Ok(ops::ControlFlow::Continue(()));
        };
        let result = eval(&self.globals, &self.locals, &expr);
        match result {
            Ok(value) => {
                self.last_result = Some(value.clone());
                writeln!(out, "{}", value.display("<expr>"))?;
            }
            Err(e) => writeln!(out, "Error evaluating expression: {e}")?,
        }

        Ok(ops::ControlFlow::Continue(()))
    }

    fn process_command(
        &mut self,
        command: &str,
        mut out: impl io::Write,
    ) -> io::Result<ops::ControlFlow<()>> {
        let args = command.split_whitespace().collect::<Vec<_>>();
        match args.get(0).copied() {
            Some("help") => writeln!(out, "{HELP_MESSAGE}")?,
            Some("exit") => {
                writeln!(out, "Exiting the REPL.")?;
                return Ok(ops::ControlFlow::Break(()));
            }
            Some("store") => {
                if args.len() < 2 {
                    writeln!(out, "Usage: .store <variable>")?;
                    return Ok(ops::ControlFlow::Continue(()));
                }
                let variable = args[1];

                if let Some(result) = &self.last_result {
                    self.locals.assign(variable, result.clone());
                    writeln!(out, "Stored last result in variable '{variable}'")?;
                } else {
                    writeln!(out, "No last result to store")?;
                }
            }
            Some("load") => {
                if args.len() < 3 {
                    writeln!(out, "Usage: .load <type> <filename>")?;
                    return Ok(ops::ControlFlow::Continue(()));
                }
                let load_type = args[1];
                let filename = args[2];

                match load_type {
                    "prog" => match load_file(filename, &mut self.globals) {
                        Ok(()) => writeln!(out, "Program loaded successfully from '{filename}'")?,
                        Err(e) => writeln!(out, "Error loading program: {e}")?,
                    },
                    "rel" => match load_relation(filename, &mut self.locals) {
                        Ok(name) => writeln!(
                            out,
                            "Relation '{name}' loaded successfully from '{filename}'"
                        )?,
                        Err(e) => writeln!(out, "Error loading relation: {e}")?,
                    },
                    "mat" => match load_matrix(filename, &mut self.locals) {
                        Err(e) => writeln!(out, "Error loading matrix: {e}")?,
                        Ok(name) => {
                            writeln!(out, "Matrix '{name}' loaded successfully from '{filename}'")?
                        }
                    },
                    _ => writeln!(out, "Unknown load type: '{load_type}'")?,
                }
            }
            Some("save") => {
                if args.len() < 4 {
                    writeln!(out, "Usage: .save <type> <variable> <filename>")?;
                    return Ok(ops::ControlFlow::Continue(()));
                }
                let save_type = args[1];
                let variable = args[2];
                let filename = args[3];

                match save_type {
                    "rel" => match save_relation(&self.locals, variable, filename) {
                        Ok(()) => writeln!(out, "Relation '{variable}' saved to '{filename}'")?,
                        Err(e) => writeln!(out, "Error saving relation: {e}")?,
                    },
                    "mat" => match save_matrix(&self.locals, variable, filename) {
                        Ok(()) => writeln!(out, "Matrix '{variable}' saved to '{filename}'")?,
                        Err(e) => writeln!(out, "Error saving matrix: {e}")?,
                    },
                    _ => writeln!(out, "Unknown save type: '{save_type}'")?,
                }
            }
            Some(command) => writeln!(out, "Unknown command '{command}'")?,
            None => writeln!(out, "No command provided")?,
        }

        Ok(ops::ControlFlow::Continue(()))
    }
}
