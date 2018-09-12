use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate byteorder;
extern crate uuid;
extern crate chrono;

extern crate iridium;
use iridium::assembler::Assembler;
use iridium::vm::VM;
use iridium::repl::REPL;
use clap::App;

fn main() {
    env_logger::init();
    info!("Starting logging!");
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let target_file = matches.value_of("INPUT_FILE");
    match target_file {
        Some(filename) => {
            let program = read_file(filename);
            let mut asm = Assembler::new();
            let mut vm = VM::new();
            let program = asm.assemble(&program);
            match program {
                Ok(p) => {
                    vm.add_bytes(p);
                    let events = vm.run();
                    println!("VM Events");
                    println!("--------------------------");
                    for event in &events {
                        println!("{:#?}", event);
                    };
                    std::process::exit(0);
                },
                Err(_e) => {

                }
            }
        },
        None => {
            start_repl();
        }
    }
}

fn start_repl() {
    let mut repl = REPL::new();
    repl.run();
}

fn read_file(tmp: &str) -> String {
    let filename = Path::new(tmp);
    match File::open(Path::new(&filename)) {
      Ok(mut fh) => {
        let mut contents = String::new();
        match fh.read_to_string(&mut contents) {
          Ok(_) => {
            contents
          },
          Err(e) => {
            println!("There was an error reading file: {:?}", e);
            std::process::exit(1);
          }
        }
      },
      Err(e) => {
        println!("File not found: {:?}", e);
        std::process::exit(1)
      }
    }
}
