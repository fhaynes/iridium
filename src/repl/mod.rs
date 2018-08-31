use std;
use std::io;
use std::io::Write;
use std::io::prelude::*;
use std::fs::File;
use std::num::ParseIntError;
use std::path::Path;

use assembler::program_parsers::program;
use assembler::Assembler;
use vm::VM;

/// Core structure for the REPL for the Assembler
pub struct REPL {
    command_buffer: Vec<String>,
    vm: VM,
    asm: Assembler,
}

impl REPL {
    /// Creates and returns a new assembly REPL
    pub fn new() -> REPL {
        REPL {
            vm: VM::new(),
            command_buffer: vec![],
            asm: Assembler::new()
        }
    }

    /// Run loop similar to the VM execution loop, but the instructions are taken from the user directly
    /// at the terminal and not from pre-compiled bytecode
    pub fn run(&mut self) {
        println!("Welcome to Iridium! Let's be productive!");
        loop {
            // This allocates a new String in which to store whatever the user types each iteration.
            // TODO: Figure out how allocate this outside of the loop and re-use it every iteration
            let mut buffer = String::new();

            // Blocking call until the user types in a command
            let stdin = io::stdin();

            // Annoyingly, `print!` does not automatically flush stdout like `println!` does, so we
            // have to do that there for the user to see our `>>> ` prompt.
            print!(">>> ");
            io::stdout().flush().expect("Unable to flush stdout");

            // Here we'll look at the string the user gave us.
            stdin
                .read_line(&mut buffer)
                .expect("Unable to read line from user");
            let buffer = buffer.trim();
            self.command_buffer.push(buffer.to_string());
            match buffer {
                ".quit" => {
                    println!("Farewell! Have a great day!");
                    std::process::exit(0);
                }
                ".history" => {
                    for command in &self.command_buffer {
                        println!("{}", command);
                    }
                }
                ".program" => {
                    println!("Listing instructions currently in VM's program vector:");
                    for instruction in &self.vm.program {
                        println!("{}", instruction);
                    }
                    println!("End of Program Listing");
                }
                ".clear_program" => {
                    println!("Removing all bytes from VM's program vector...");
                    self.vm.program.truncate(0);
                    println!("Done!");
                }
                ".clear_registers" => {
                    println!("Setting all registers to 0");
                    for i in 0..self.vm.registers.len() {
                        self.vm.registers[i] = 0;
                    }
                    println!("Done!");
                }
                ".registers" => {
                    println!("Listing registers and all contents:");
                    println!("{:#?}", self.vm.registers);
                    println!("End of Register Listing")
                }
                ".symbols" => {
                    println!("Listing symbols table:");
                    println!("{:#?}", self.asm.symbols);
                    println!("End of Symbols Listing");
                }
                ".load_file" => {
                    print!("Please enter the path to the file you wish to load: ");
                    io::stdout().flush().expect("Unable to flush stdout");
                    let mut tmp = String::new();
                    stdin.read_line(&mut tmp).expect("Unable to read line from user");
                    println!("Attempting to load program from file...");
                    let tmp = tmp.trim();
                    let filename = Path::new(&tmp);
                    let mut f = File::open(Path::new(&filename)).expect("File not found");
                    let mut contents = String::new();
                    f.read_to_string(&mut contents).expect("There was an error reading from the file");
                    match self.asm.assemble(&contents) {
                        Ok(mut assembled_program) => {
                            println!("Sending assembled program to VM");
                            self.vm.program.append(&mut assembled_program);
                            println!("{:#?}", self.vm.program);
                            self.vm.run();
                        },
                        Err(errors) => {
                            for error in errors {
                                println!("Unable to parse input: {}", error);
                            }
                            continue;
                        }
                    }
                }
                _ => {
                    let program = match program(buffer.into()) {
                        // Rusts pattern matching is pretty powerful an can even be nested
                        Ok((_remainder, program)) => {
                            program
                        },
                        Err(e) => {
                            println!("Unable to parse input: {:?}", e);
                            continue;
                        }
                    };

                    self.vm.program.append(&mut program.to_bytes(&self.asm.symbols));
                    self.vm.run_once();
                }
            }
        }
    }

    /// Accepts a hexadecimal string WITHOUT a leading `0x` and returns a Vec of u8
    /// Example for a LOAD command: 00 01 03 E8
    #[allow(dead_code)]
    fn parse_hex(&mut self, i: &str) -> Result<Vec<u8>, ParseIntError> {
        let split = i.split(' ').collect::<Vec<&str>>();
        let mut results: Vec<u8> = vec![];
        for hex_string in split {
            let byte = u8::from_str_radix(&hex_string, 16);
            match byte {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(results)
    }
}

impl Default for REPL {
    fn default() -> Self {
        Self::new()
    }
}
