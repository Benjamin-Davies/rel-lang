use std::env;

use rel_lang::parser::parse;

fn main() {
    let filename = env::args()
        .nth(1)
        .expect("Please provide a filename as the first argument");
    let src = std::fs::read_to_string(&filename).unwrap();

    let ast = parse(&filename, &src);

    dbg!(ast);
}
