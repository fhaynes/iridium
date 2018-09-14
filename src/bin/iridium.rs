use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate byteorder;
extern crate chrono;
extern crate env_logger;
extern crate uuid;
extern crate num_cpus;
extern crate iridium;

use clap::App;
use iridium::assembler::Assembler;
use iridium::repl::REPL;
use iridium::vm::VM;

fn main() {
    env_logger::init();
    info!("Starting logging!");
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let num_threads = match matches.value_of("THREADS") {
        Some(number) => {
            match number.parse::<usize>() {
                Ok(v) => { v }
                Err(_e) => {
                    println!("Invalid argument for number of threads: {}. Using default.", number);
                    num_cpus::get()
                }
            }
        }
        None => {
            num_cpus::get()
        }
    };

    let target_file = matches.value_of("INPUT_FILE");
    match target_file {
        Some(filename) => {
            let program = read_file(filename);
            let mut asm = Assembler::new();
            let mut vm = VM::new();
            vm.logical_cores = num_threads;
            let program = asm.assemble(&program);
            match program {
                Ok(p) => {
                    vm.add_bytes(p);
                    let events = vm.run();
                    println!("VM Events");
                    println!("--------------------------");
                    for event in &events {
                        println!("{:#?}", event);
                    }
                    std::process::exit(0);
                }
                Err(_e) => {}
            }
        }
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
                Ok(_) => contents,
                Err(e) => {
                    println!("There was an error reading file: {:?}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            println!("File not found: {:?}", e);
            std::process::exit(1)
        }
    }
}
