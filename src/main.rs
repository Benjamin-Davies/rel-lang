use std::env;

use rel_lang::{eval::Globals, load_file};

fn main() {
    let filename = env::args()
        .nth(1)
        .expect("Please provide a filename as the first argument");

    let mut globals = Globals::default();
    load_file(&filename, &mut globals);

    todo!();
}
