use std::{cell::RefCell, io, ops, rc::Rc};

use rel_lang::{repl::Repl, repl_helper::ReplHelper};
use rustyline::{Editor, Result, error::ReadlineError};

fn main() -> Result<()> {
    let mut rl = Editor::new()?;
    let mut stdout = io::stdout();

    let repl = Repl::new();
    repl.welcome(&mut stdout)?;

    let repl = Rc::new(RefCell::new(repl));
    rl.set_helper(Some(ReplHelper::new(repl.clone())));

    loop {
        let line = match rl.readline("> ") {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                line
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(err) => return Err(err),
        };

        let res = repl.borrow_mut().process_input(&line, &mut stdout)?;

        match res {
            ops::ControlFlow::Continue(()) => continue,
            ops::ControlFlow::Break(()) => break,
        }
    }

    Ok(())
}
