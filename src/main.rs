use std::{
    io::{self, Write},
    ops,
};

use rel_lang::{repl::Repl, repl_helper::ReplHelper};
use rustyline::{Editor, Result, error::ReadlineError};

fn main() -> Result<()> {
    let mut stdout = io::stdout();

    let mut rl = Editor::new()?;
    rl.set_helper(Some(ReplHelper::new()));

    let mut repl = Repl::new();
    repl.welcome(&mut stdout)?;

    loop {
        stdout.write_all(b"> ")?;
        stdout.flush()?;

        let line = match rl.readline("> ") {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                line
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => return Err(err),
        };

        let res = repl.process_input(&line, &mut stdout)?;

        match res {
            ops::ControlFlow::Continue(()) => continue,
            ops::ControlFlow::Break(()) => break,
        }
    }

    Ok(())
}
