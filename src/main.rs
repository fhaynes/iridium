#[macro_use]
extern crate nom;

pub mod vm;
pub mod instruction;
pub mod repl;
pub mod assembler;

fn main() {
    let mut repl = repl::REPL::new();
    repl.run();
}
