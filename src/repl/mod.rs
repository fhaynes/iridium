pub mod command_parser;

use std;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::num::ParseIntError;
use std::path::Path;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use nom::types::CompleteStr;

use assembler::program_parsers::program;
use assembler::Assembler;
use cluster;
use repl::command_parser::CommandParser;
use scheduler::Scheduler;
use vm::VM;
const COMMAND_PREFIX: char = '!';

pub static REMOTE_BANNER: &'static str = "Welcome to Iridium! Let's be productive!";
pub static PROMPT: &'static str = ">>> ";

/// Core structure for the REPL for the Assembler
#[derive(Default)]
pub struct REPL {
    command_buffer: Vec<String>,
    vm: VM,
    asm: Assembler,
    scheduler: Scheduler,
    pub tx_pipe: Option<Box<Sender<String>>>,
    pub rx_pipe: Option<Box<Receiver<String>>>,
}

impl REPL {
    /// Creates and returns a new assembly REPL
    pub fn new(vm: VM) -> REPL {
        let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        REPL {
            vm,
            command_buffer: vec![],
            asm: Assembler::new(),
            scheduler: Scheduler::new(),
            tx_pipe: Some(Box::new(tx)),
            rx_pipe: Some(Box::new(rx)),
        }
    }

    /// Run loop similar to the VM execution loop, but the instructions are taken from the user directly
    /// at the terminal and not from pre-compiled bytecode
    pub fn run(&mut self) {
        self.send_message(REMOTE_BANNER.to_string());
        loop {
            // This allocates a new String in which to store whatever the user types each iteration.
            // TODO: Figure out how allocate this outside of the loop and re-use it every iteration
            let mut buffer = String::new();

            // Blocking call until the user types in a command
            let stdin = io::stdin();

            // Annoyingly, `print!` does not automatically flush stdout like `self.send_message` does, so we
            // have to do that there for the user to see our `>>> ` prompt.
            print!(">>> ");
            io::stdout().flush().expect("Unable to flush stdout");

            // Here we'll look at the string the user gave us.
            stdin
                .read_line(&mut buffer)
                .expect("Unable to read line from user");

            let historical_copy = buffer.clone();
            self.command_buffer.push(historical_copy);

            if buffer.starts_with(COMMAND_PREFIX) {
                self.execute_command(&buffer);
            } else {
                let program = match program(CompleteStr(&buffer)) {
                    Ok((_remainder, program)) => program,
                    Err(_e) => {
                        self.send_message(REMOTE_BANNER.to_string());
                        self.send_prompt();
                        continue;
                    }
                };
                self.vm
                    .program
                    .append(&mut program.to_bytes(&self.asm.symbols));
                self.vm.run_once();
            }
        }
    }

    pub fn run_single(&mut self, buffer: &str) -> Option<String> {
        if buffer.starts_with(COMMAND_PREFIX) {
            self.execute_command(&buffer);
            None
        } else {
            let program = match program(CompleteStr(&buffer)) {
                Ok((_remainder, program)) => Some(program),
                Err(e) => {
                    self.send_message(format!("Unable to parse input: {:?}", e));
                    self.send_prompt();
                    None
                }
            };
            match program {
                Some(p) => {
                    let mut bytes = p.to_bytes(&self.asm.symbols);
                    self.vm.program.append(&mut bytes);
                    self.vm.run_once();
                    None
                }
                None => None,
            }
        }
    }

    pub fn send_message(&mut self, msg: String) {
        match &self.tx_pipe {
            Some(pipe) => {
                match pipe.send(msg) {
                    Ok(_) => {}
                    Err(_e) => {}
                };
            }
            None => {}
        }
    }
    pub fn send_prompt(&mut self) {
        match &self.tx_pipe {
            Some(pipe) => match pipe.send(PROMPT.to_owned()) {
                Ok(_) => {}
                Err(_e) => {}
            },
            None => {}
        }
    }

    fn get_data_from_load(&mut self) -> Option<String> {
        let stdin = io::stdin();
        self.send_message("Please enter the path to the file you wish to load: ".to_string());
        let mut tmp = String::new();

        stdin
            .read_line(&mut tmp)
            .expect("Unable to read line from user");
        self.send_message("Attempting to load program from file...".to_string());

        let tmp = tmp.trim();
        let filename = Path::new(&tmp);
        let mut f = match File::open(&filename) {
            Ok(f) => f,
            Err(e) => {
                self.send_message(format!("There was an error opening that file: {:?}", e));
                return None;
            }
        };
        let mut contents = String::new();
        match f.read_to_string(&mut contents) {
            Ok(_bytes_read) => Some(contents),
            Err(e) => {
                self.send_message(format!("there was an error reading that file: {:?}", e));
                None
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

    fn execute_command(&mut self, input: &str) {
        let args = CommandParser::tokenize(input);
        match args[0] {
            "!quit" => self.quit(&args[1..]),
            "!history" => self.history(&args[1..]),
            "!program" => self.program(&args[1..]),
            "!clear_program" => self.clear_program(&args[1..]),
            "!clear_registers" => self.clear_registers(&args[1..]),
            "!registers" => self.registers(&args[1..]),
            "!symbols" => self.symbols(&args[1..]),
            "!load_file" => self.load_file(&args[1..]),
            "!spawn" => self.spawn(&args[1..]),
            "!start_cluster" => self.start_cluster(&args[1..]),
            "!join_cluster" => self.join_cluster(&args[1..]),
            "!cluster_members" => self.cluster_members(&args[1..]),
            _ => {
                self.send_message("Invalid command!".to_string());
                self.send_prompt();
            }
        };
    }

    fn quit(&mut self, _args: &[&str]) {
        self.send_message("Farewell! Have a great day!".to_string());
        std::process::exit(0);
    }

    fn history(&mut self, _args: &[&str]) {
        let mut results = vec![];
        for command in &self.command_buffer {
            results.push(command.clone());
        }
        self.send_message(format!("{:#?}", results));
        self.send_prompt();
    }

    fn program(&mut self, _args: &[&str]) {
        self.send_message("Listing instructions currently in VM's program vector: ".to_string());
        let mut results = vec![];
        for instruction in &self.vm.program {
            results.push(instruction.clone())
        }
        self.send_message(format!("{:#?}", results));
        self.send_message("End of Program Listing".to_string());
        self.send_prompt();
    }

    fn clear_program(&mut self, _args: &[&str]) {
        self.vm.program.clear();
    }

    fn clear_registers(&mut self, _args: &[&str]) {
        self.send_message("Setting all registers to 0".to_string());
        for i in 0..self.vm.registers.len() {
            self.vm.registers[i] = 0;
        }
        self.send_message("Done!".to_string());
        self.send_prompt();
    }

    fn registers(&mut self, _args: &[&str]) {
        self.send_message("Listing registers and all contents:".to_string());
        let mut results = vec![];
        for register in &self.vm.registers {
            results.push(register.clone());
        }
        self.send_message(format!("{:#?}", results));
        self.send_message("End of Register Listing".to_string());
        self.send_prompt();
    }

    fn symbols(&mut self, _args: &[&str]) {
        let mut results = vec![];
        for symbol in &self.asm.symbols.symbols {
            results.push(symbol.clone());
        }
        self.send_message("Listing symbols table:".to_string());
        self.send_message(format!("{:#?}", results));
        self.send_message("End of Symbols Listing".to_string());
        self.send_prompt();
    }

    fn load_file(&mut self, _args: &[&str]) {
        let contents = self.get_data_from_load();
        if let Some(contents) = contents {
            match self.asm.assemble(&contents) {
                Ok(mut assembled_program) => {
                    self.send_message("Sending assembled program to VM".to_string());
                    self.vm.program.append(&mut assembled_program);
                    self.vm.run();
                }
                Err(errors) => {
                    for error in errors {
                        self.send_message(format!("Unable to parse input: {}", error));
                        self.send_prompt();
                    }
                    return;
                }
            }
        } else {
            return;
        }
    }

    fn spawn(&mut self, _args: &[&str]) {
        let contents = self.get_data_from_load();
        self.send_message(format!("Loaded contents: {:#?}", contents));
        if let Some(contents) = contents {
            match self.asm.assemble(&contents) {
                Ok(mut assembled_program) => {
                    self.send_message("Sending assembled program to VM".to_string());
                    self.vm.program.append(&mut assembled_program);
                    self.scheduler.get_thread(self.vm.clone());
                }
                Err(errors) => {
                    for error in errors {
                        self.send_message(format!("Unable to parse input: {}", error));
                        self.send_prompt();
                    }
                    return;
                }
            }
        } else {
            return;
        }
    }

    fn start_cluster(&mut self, _args: &[&str]) {
        self.send_message("Started cluster server!".to_string());
        self.vm.bind_cluster_server();
    }

    fn join_cluster(&mut self, args: &[&str]) {
        self.send_message("Attempting to join cluster...".to_string());
        let ip = args[0];
        let port = args[1];
        let addr = ip.to_owned() + ":" + port;
        if let Ok(stream) = TcpStream::connect(addr) {
            self.send_message("Connected to cluster!".to_string());
            let mut cc =
                cluster::client::ClusterClient::new(stream).with_alias(self.vm.id.to_string());
            cc.send_hello();
            if let Some(ref a) = self.vm.alias {
                if let Ok(mut lock) = self.vm.connection_manager.write() {
                    lock.add_client(a.to_string(), cc);
                }
            }
        } else {
            self.send_message("Could not connect to cluster!".to_string());
        }
    }

    fn cluster_members(&mut self, _args: &[&str]) {
        self.send_message("Listing Known Nodes:".to_string());
        let cluster_members = self
            .vm
            .connection_manager
            .read()
            .unwrap()
            .get_client_names();
        self.send_message(format!("{:#?}", cluster_members));
    }
}
