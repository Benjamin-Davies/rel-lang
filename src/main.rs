use std::{
    io::{self, Write},
    ops,
};

use rel_lang::repl::Repl;

fn main() -> Result<(), io::Error> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut repl = Repl::new();
    repl.welcome(&mut stdout)?;

    loop {
        stdout.write_all(b"> ")?;
        stdout.flush()?;

        let mut line = String::new();
        stdin.read_line(&mut line)?;
        if line.is_empty() {
            break; // Exit on EOF
        }

        let res = repl.process_input(&line, &mut stdout)?;

        match res {
            ops::ControlFlow::Continue(()) => continue,
            ops::ControlFlow::Break(()) => break,
        }
    }

    Ok(())
}
