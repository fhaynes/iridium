#[macro_use]
extern crate nom;

pub mod assembler;
pub mod instruction;
pub mod repl;
pub mod vm;

fn main() {
    let mut repl = repl::REPL::new();
    repl.run();
}
