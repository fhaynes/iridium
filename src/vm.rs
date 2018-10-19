use std;
use std::io::Cursor;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::thread;

use byteorder::*;
use chrono::prelude::*;
use num_cpus;
use uuid::Uuid;

use assembler::{PIE_HEADER_LENGTH, PIE_HEADER_PREFIX};
use cluster;
use cluster::manager::Manager;
use instruction::Opcode;
use std::f64::EPSILON;

#[derive(Clone, Debug)]
pub enum VMEventType {
    Start,
    GracefulStop { code: u32 },
    Crash { code: u32 },
}

impl VMEventType {
    pub fn stop_code(&self) -> u32 {
        match &self {
            VMEventType::Start => 0,
            VMEventType::GracefulStop { code } => *code,
            VMEventType::Crash { code } => *code,
        }
    }
}
#[derive(Clone, Debug)]
pub struct VMEvent {
    pub event: VMEventType,
    at: DateTime<Utc>,
    application_id: Uuid,
}

pub const DEFAULT_HEAP_STARTING_SIZE: usize = 64;

/// Virtual machine struct that will execute bytecode
#[derive(Default, Clone)]
pub struct VM {
    /// Array that simulates having hardware registers
    pub registers: [i32; 32],
    /// Array that simulates having floating point hardware registers
    pub float_registers: [f64; 32],
    /// The bytecode of the program being run
    pub program: Vec<u8>,
    /// Number of logical cores the system reports
    pub logical_cores: usize,
    /// An alias that can be specified by the user and used to refer to the Node
    pub alias: Option<String>,
    /// Data structure to manage remote clients
    pub connection_manager: Arc<RwLock<Manager>>,
    /// Program counter that tracks which byte is being executed
    pc: usize,
    /// Used for heap memory
    heap: Vec<u8>,
    /// Used to represent the stack
    stack: Vec<u8>,
    /// Contains the remainder of modulo division ops
    remainder: usize,
    /// Contains the result of the last comparison operation
    equal_flag: bool,
    /// Loop counter field, used with the `LOOP` instruction
    loop_counter: usize,
    /// Contains the read-only section data
    ro_data: Vec<u8>,
    /// Is a unique, randomly generated UUID for identifying this VM
    pub id: Uuid,
    /// Keeps a list of events for a particular VM
    events: Vec<VMEvent>,
    // Server address that the VM will bind to for server-to-server communications
    server_addr: Option<String>,
    // Port the server will bind to for server-to-server communications
    server_port: Option<String>,
}

impl VM {
    /// Creates and returns a new VM
    pub fn new() -> VM {
        VM {
            registers: [0; 32],
            float_registers: [0.0; 32],
            program: vec![],
            ro_data: vec![],
            heap: vec![0; DEFAULT_HEAP_STARTING_SIZE],
            stack: vec![],
            connection_manager: Arc::new(RwLock::new(Manager::new())),
            pc: 0,
            loop_counter: 0,
            remainder: 0,
            equal_flag: false,
            id: Uuid::new_v4(),
            alias: None,
            events: Vec::new(),
            logical_cores: num_cpus::get(),
            server_addr: None,
            server_port: None,
        }
    }

    /// Wraps execution in a loop so it will continue to run until done or there is an error
    /// executing instructions.
    pub fn run(&mut self) -> Vec<VMEvent> {
        self.events.push(VMEvent {
            event: VMEventType::Start,
            at: Utc::now(),
            application_id: self.id,
        });
        // TODO: Should setup custom errors here
        if !self.verify_header() {
            self.events.push(VMEvent {
                event: VMEventType::Crash { code: 1 },
                at: Utc::now(),
                application_id: self.id,
            });
            println!("Header was incorrect");
            return self.events.clone();
        }

        self.pc = 68 + self.get_starting_offset();
        let mut is_done = None;
        while is_done.is_none() {
            is_done = self.execute_instruction();
        }
        self.events.push(VMEvent {
            event: VMEventType::GracefulStop {
                code: is_done.unwrap(),
            },
            at: Utc::now(),
            application_id: self.id,
        });
        self.events.clone()
    }

    pub fn with_alias(mut self, alias: String) -> Self {
        if alias == "" {
            self.alias = Some(alias)
        } else {
            self.alias = None
        }
        self
    }

    pub fn with_cluster_bind(mut self, server_addr: String, server_port: String) -> Self {
        self.server_addr = Some(server_addr);
        self.server_port = Some(server_port);
        self
    }

    /// Executes one instruction. Meant to allow for more controlled execution of the VM
    pub fn run_once(&mut self) {
        self.execute_instruction();
    }

    /// Adds an arbitrary byte to the VM's program
    pub fn add_byte(&mut self, b: u8) {
        self.program.push(b);
    }

    /// Adds an arbitrary byte to the VM's program
    pub fn add_bytes(&mut self, mut b: Vec<u8>) {
        self.program.append(&mut b);
    }

    /// Executes an instruction and returns a bool. Meant to be called by the various public run
    /// functions.
    fn execute_instruction(&mut self) -> Option<u32> {
        if self.pc >= self.program.len() {
            return Some(1);
        }
        match self.decode_opcode() {
            Opcode::LOAD => {
                let register = self.next_8_bits() as usize;
                let number = i32::from(self.next_16_bits());
                self.registers[register] = number;
            }
            Opcode::ADD => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 + register2;
            }
            Opcode::SUB => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 - register2;
            }
            Opcode::MUL => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 * register2;
            }
            Opcode::DIV => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 / register2;
                self.remainder = (register1 % register2) as usize;
            }
            Opcode::HLT => {
                info!("HLT encountered");
                return Some(0);
            }
            Opcode::IGL => {
                error!("Illegal instruction encountered");
                return Some(1);
            }
            Opcode::JMP => {
                let target = self.registers[self.next_8_bits() as usize];
                self.pc = target as usize;
            }
            Opcode::JMPF => {
                let value = self.registers[self.next_8_bits() as usize] as usize;
                self.pc += value;
            }
            Opcode::JMPB => {
                let value = self.registers[self.next_8_bits() as usize] as usize;
                self.pc -= value;
            }
            Opcode::EQ => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 == register2;
                self.next_8_bits();
            }
            Opcode::NEQ => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 != register2;
                self.next_8_bits();
            }
            Opcode::GT => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 > register2;
                self.next_8_bits();
            }
            Opcode::GTE => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 >= register2;
                self.next_8_bits();
            }
            Opcode::LT => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 < register2;
                self.next_8_bits();
            }
            Opcode::LTE => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 <= register2;
                self.next_8_bits();
            }
            Opcode::JMPE => {
                if self.equal_flag {
                    let register = self.next_8_bits() as usize;
                    let target = self.registers[register];
                    self.pc = target as usize;
                } else {
                    // TODO: Fix the bits
                }
            }
            Opcode::NOP => {
                self.next_8_bits();
                self.next_8_bits();
                self.next_8_bits();
            }
            Opcode::ALOC => {
                let register = self.next_8_bits() as usize;
                let bytes = self.registers[register];
                let new_end = self.heap.len() as i32 + bytes;
                self.heap.resize(new_end as usize, 0);
            }
            Opcode::INC => {
                let register_number = self.next_8_bits() as usize;
                self.registers[register_number] += 1;
                self.next_8_bits();
                self.next_8_bits();
            }
            Opcode::DEC => {
                let register_number = self.next_8_bits() as usize;
                self.registers[register_number] -= 1;
                self.next_8_bits();
                self.next_8_bits();
            }
            Opcode::DJMPE => {
                let destination = self.next_16_bits();
                if self.equal_flag {
                    self.pc = destination as usize;
                } else {
                    self.next_8_bits();
                }
            }
            Opcode::PRTS => {
                // PRTS takes one operand, either a starting index in the read-only section of the bytecode
                // or a symbol (in the form of @symbol_name), which will look up the offset in the symbol table.
                // This instruction then reads each byte and prints it, until it comes to a 0x00 byte, which indicates
                // termination of the string
                let starting_offset = self.next_16_bits() as usize;
                let mut ending_offset = starting_offset;
                let slice = self.ro_data.as_slice();
                // TODO: Find a better way to do this. Maybe we can store the byte length and not null terminate? Or some form of caching where we
                // go through the entire ro_data on VM startup and find every string and its ending byte location?
                while slice[ending_offset] != 0 {
                    ending_offset += 1;
                }
                let result = std::str::from_utf8(&slice[starting_offset..ending_offset]);
                match result {
                    Ok(s) => {
                        print!("{}", s);
                    }
                    Err(e) => println!("Error decoding string for prts instruction: {:#?}", e),
                };
            }
            // Begin floating point 64-bit instructions
            Opcode::LOADF64 => {
                let register = self.next_8_bits() as usize;
                let number = f64::from(self.next_16_bits());
                self.float_registers[register] = number;
            }
            Opcode::ADDF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.float_registers[self.next_8_bits() as usize] = register1 + register2;
            }
            Opcode::SUBF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.float_registers[self.next_8_bits() as usize] = register1 - register2;
            }
            Opcode::MULF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.float_registers[self.next_8_bits() as usize] = register1 * register2;
            }
            Opcode::DIVF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.float_registers[self.next_8_bits() as usize] = register1 / register2;
            }
            Opcode::EQF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.equal_flag = (register1 - register2).abs() < EPSILON;
                self.next_8_bits();
            }
            Opcode::NEQF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.equal_flag = (register1 - register2).abs() > EPSILON;
                self.next_8_bits();
            }
            Opcode::GTF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.equal_flag = register1 > register2;
                self.next_8_bits();
            }
            Opcode::GTEF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.equal_flag = register1 >= register2;
                self.next_8_bits();
            }
            Opcode::LTF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.equal_flag = register1 < register2;
                self.next_8_bits();
            }
            Opcode::LTEF64 => {
                let register1 = self.float_registers[self.next_8_bits() as usize];
                let register2 = self.float_registers[self.next_8_bits() as usize];
                self.equal_flag = register1 <= register2;
                self.next_8_bits();
            }
            Opcode::SHL => {
                let reg_num = self.next_8_bits() as usize;
                let num_bits = match self.next_8_bits() {
                    0 => 16,
                    other => other,
                };
                self.registers[reg_num] = self.registers[reg_num].wrapping_shl(num_bits.into());
                self.next_8_bits();
            }
            Opcode::SHR => {
                let reg_num = self.next_8_bits() as usize;
                let num_bits = match self.next_8_bits() {
                    0 => 16,
                    other => other,
                };
                self.registers[reg_num] = self.registers[reg_num].wrapping_shr(num_bits.into());
                self.next_8_bits();
            }
            Opcode::AND => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 & register2;
            }
            Opcode::OR => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 | register2;
            }
            Opcode::XOR => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 ^ register2;
            }
            Opcode::NOT => {
                let register1 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = !register1;
                self.next_8_bits();
            }
            Opcode::LUI => {
                let register = self.next_8_bits() as usize;
                let value = self.registers[register];
                let uv1 = i32::from(self.next_8_bits());
                let uv2 = i32::from(self.next_8_bits());
                let value = value.checked_shl(8).unwrap();
                let value = value | uv1;
                let value = value.checked_shl(8).unwrap();
                let value = value | uv2;
                self.registers[register] = value;
            }
            Opcode::LOOP => {
                if self.loop_counter != 0 {
                    self.loop_counter -= 1;
                    let target = self.next_16_bits();
                    self.pc = target as usize;
                } else {
                    self.pc += 3;
                }
            }
            Opcode::CLOOP => {
                let loop_count = self.next_16_bits();
                self.loop_counter = loop_count as usize;
                self.next_8_bits();
            }
            Opcode::LOADM => {
                let offset = self.registers[self.next_8_bits() as usize] as usize;
                let data: i32;
                // Explicit scoping is necessary here because we do an immutable borrow of self, then a mutable borrow to assign the result
                {
                    let mut slice = &self.heap[offset..offset + 4];
                    data = slice.read_i32::<LittleEndian>().unwrap();
                }
                self.registers[self.next_8_bits() as usize] = data;
            }
            Opcode::SETM => {
                let _offset = self.registers[self.next_8_bits() as usize] as usize;
                let data = self.registers[self.next_8_bits() as usize];
                let mut buf: [u8; 4] = [0, 0, 0, 0];
                let _ = buf.as_mut().write_i32::<LittleEndian>(data);
            }
            Opcode::PUSH => {
                let data = self.registers[self.next_8_bits() as usize];
                let mut buf: [u8; 4] = [0, 0, 0, 0];
                if buf.as_mut().write_i32::<LittleEndian>(data).is_ok() {
                    for b in &buf {
                        self.stack.push(*b);
                    }
                } else {
                    return Some(1);
                }
            }
            Opcode::POP => {
                let target_register = self.next_8_bits() as usize;
                let mut buf: [u8; 4] = [0, 0, 0, 0];
                let new_len = self.stack.len() - 4;
                //for removed_element in self.stack.drain(new_len..) {
                for (c, removed_element) in self.stack.drain(new_len..).enumerate() {
                    buf[c] = removed_element;
                }
                let data = LittleEndian::read_i32(&buf);
                self.registers[target_register] = data;
            }
            Opcode::CALL => {
                let return_destination = self.pc + 3;
                let destination = self.next_16_bits();
                let bytes: [u8; 4] = VM::i32_to_bytes(return_destination as i32);
                self.stack.extend_from_slice(&bytes);
                self.pc = destination as usize;
            }
            Opcode::RET => {
                let mut buf: [u8; 4] = [0, 0, 0, 0];
                let new_len = self.stack.len() - 4;
                for (c, removed_element) in self.stack.drain(new_len..).enumerate() {
                    buf[c] = removed_element;
                }
                let data = LittleEndian::read_i32(&buf);
                self.pc = data as usize;
            }
        };
        None
    }

    pub fn print_i32_register(&self, register: usize) {
        let bits = self.registers[register];
        println!("bits: {:#032b}", bits);
    }

    pub fn get_test_vm() -> VM {
        let mut test_vm = VM::new();
        test_vm.registers[0] = 5;
        test_vm.registers[1] = 10;
        test_vm.float_registers[0] = 5.0;
        test_vm.float_registers[1] = 10.0;
        test_vm
    }

    pub fn prepend_header(mut b: Vec<u8>) -> Vec<u8> {
        let mut prepension = vec![];
        for byte in &PIE_HEADER_PREFIX {
            prepension.push(byte.clone());
        }

        // The 4 is added here to allow for the 4 bytes that tell the VM where the executable code starts
        while prepension.len() < PIE_HEADER_LENGTH + 4 {
            prepension.push(0);
        }

        prepension.append(&mut b);
        prepension
    }

    pub fn bind_cluster_server(&mut self) {
        if let Some(ref addr) = self.server_addr {
            if let Some(ref port) = self.server_port {
                let socket_addr: SocketAddr = (addr.to_string() + ":" + port).parse().unwrap();
                let clone = self.connection_manager.clone();
                thread::spawn(move || {
                    cluster::server::listen(socket_addr, clone);
                });
            } else {
                error!("Unable to bind to cluster server address: {}", addr);
            }
        } else {
            error!(
                "Unable to bind to cluster server port: {:?}",
                self.server_port
            );
        }
    }

    fn get_starting_offset(&self) -> usize {
        let mut rdr = Cursor::new(&self.program[64..68]);
        rdr.read_u32::<LittleEndian>().unwrap() as usize
    }

    // Attempts to decode the byte the VM's program counter is pointing at into an opcode
    fn decode_opcode(&mut self) -> Opcode {
        let opcode = Opcode::from(self.program[self.pc]);
        self.pc += 1;
        opcode
    }

    fn i32_to_bytes(num: i32) -> [u8; 4] {
        let mut buf: [u8; 4] = [0, 0, 0, 0];
        buf.as_mut().write_i32::<LittleEndian>(num).unwrap();
        buf
    }

    // Attempts to decode the next byte into an opcode
    fn next_8_bits(&mut self) -> u8 {
        let result = self.program[self.pc];
        self.pc += 1;
        result
    }

    // Grabs the next 16 bits (2 bytes)
    fn next_16_bits(&mut self) -> u16 {
        let result =
            ((u16::from(self.program[self.pc])) << 8) | u16::from(self.program[self.pc + 1]);
        self.pc += 2;
        result
    }

    // Processes the header of bytecode the VM is asked to execute
    fn verify_header(&self) -> bool {
        if self.program[0..4] != PIE_HEADER_PREFIX {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_vm() {
        let test_vm = VM::new();
        assert_eq!(test_vm.registers[0], 0)
    }

    #[test]
    fn test_hlt_opcode() {
        let mut test_vm = VM::new();
        let test_bytes = vec![5, 0, 0, 0];
        test_vm.program = test_bytes;
        test_vm.run_once();
        assert_eq!(test_vm.pc, 1);
    }

    #[test]
    fn test_igl_opcode() {
        let mut test_vm = VM::new();
        let test_bytes = vec![254, 0, 0, 0];
        test_vm.program = test_bytes;
        test_vm.run_once();
        assert_eq!(test_vm.pc, 1);
    }

    #[test]
    fn test_load_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.program = vec![0, 0, 1, 244];
        test_vm.program = VM::prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.registers[0], 500);
    }

    #[test]
    fn test_add_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.program = vec![1, 0, 1, 2];
        println!("{:?}", test_vm.program);
        test_vm.program = VM::prepend_header(test_vm.program);
        println!("{:?}", test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.registers[2], 15);
    }

    #[test]
    fn test_sub_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.program = vec![2, 1, 0, 2];
        test_vm.program = VM::prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.registers[2], 5);
    }

    #[test]
    fn test_mul_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.program = vec![3, 0, 1, 2];
        test_vm.program = VM::prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.registers[2], 50);
    }

    #[test]
    fn test_div_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.program = vec![4, 1, 0, 2];
        test_vm.program = VM::prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.registers[2], 2);
    }

    #[test]
    fn test_jmp_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.registers[0] = 4;
        test_vm.program = vec![6, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.pc, 4);
    }

    #[test]
    fn test_jmpf_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.registers[0] = 2;
        test_vm.program = vec![7, 0, 0, 0, 5, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.pc, 4);
    }

    #[test]
    fn test_jmpb_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.registers[1] = 6;
        test_vm.program = vec![0, 0, 0, 10, 8, 1, 0, 0];
        test_vm.run_once();
        test_vm.run_once();
        assert_eq!(test_vm.pc, 0);
    }

    #[test]
    fn test_eq_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.registers[0] = 10;
        test_vm.registers[1] = 10;
        test_vm.program = vec![9, 0, 1, 0, 9, 0, 1, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.registers[1] = 20;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
    }

    #[test]
    fn test_neq_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.registers[0] = 10;
        test_vm.registers[1] = 20;
        test_vm.program = vec![10, 0, 1, 0, 10, 0, 1, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.registers[1] = 10;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
    }

    #[test]
    fn test_gte_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.registers[0] = 20;
        test_vm.registers[1] = 10;
        test_vm.program = vec![11, 0, 1, 0, 11, 0, 1, 0, 11, 0, 1, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.registers[0] = 10;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.registers[0] = 5;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
    }

    #[test]
    fn test_lte_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.registers[0] = 20;
        test_vm.registers[1] = 10;
        test_vm.program = vec![12, 0, 1, 0, 12, 0, 1, 0, 12, 0, 1, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
        test_vm.registers[0] = 10;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.registers[0] = 5;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
    }

    #[test]
    fn test_lt_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.registers[0] = 20;
        test_vm.registers[1] = 10;
        test_vm.program = vec![13, 0, 1, 0, 13, 0, 1, 0, 13, 0, 1, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
        test_vm.registers[0] = 10;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
        test_vm.registers[0] = 5;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
    }

    #[test]
    fn test_gt_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.registers[0] = 20;
        test_vm.registers[1] = 10;
        test_vm.program = vec![14, 0, 1, 0, 14, 0, 1, 0, 14, 0, 1, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.registers[0] = 10;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
        test_vm.registers[0] = 5;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
    }

    #[test]
    fn test_jmpe_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.registers[0] = 7;
        test_vm.equal_flag = true;
        test_vm.program = vec![15, 0, 0, 0, 15, 0, 0, 0, 15, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.pc, 7);
    }

    #[test]
    fn test_aloc_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.registers[0] = 1024;
        test_vm.program = vec![17, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.heap.len(), 1024 + DEFAULT_HEAP_STARTING_SIZE);
    }

    #[test]
    fn test_prts_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.ro_data.append(&mut vec![72, 101, 108, 108, 111, 0]);
        test_vm.program = vec![21, 0, 0, 0];
        test_vm.run_once();
        // TODO: How can we validate the output since it is just printing to stdout in a test?
    }

    #[test]
    fn test_load_floating_point_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.program = vec![22, 0, 1, 244];
        test_vm.program = VM::prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.float_registers[0], 500.0);
    }

    #[test]
    fn test_add_floating_point_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.program = vec![23, 0, 1, 2];
        test_vm.program = VM::prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.float_registers[2], 15.0);
    }

    #[test]
    fn test_sub_floating_point_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.program = vec![24, 1, 0, 2];
        test_vm.program = VM::prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.float_registers[2], 5.0);
    }

    #[test]
    fn test_mul_floating_point_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.program = vec![25, 1, 0, 2];
        test_vm.program = VM::prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.float_registers[2], 50.0);
    }

    #[test]
    fn test_div_floating_point_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.program = vec![26, 1, 0, 2];
        test_vm.program = VM::prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.float_registers[2], 2.0);
    }

    #[test]
    fn test_eq_floating_point_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.float_registers[0] = 10.0;
        test_vm.float_registers[1] = 10.0;
        test_vm.program = vec![27, 0, 1, 0, 27, 0, 1, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.float_registers[1] = 20.0;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
    }

    #[test]
    fn test_neq_floating_point_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.float_registers[0] = 10.0;
        test_vm.float_registers[1] = 20.0;
        test_vm.program = vec![28, 0, 1, 0, 28, 0, 1, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.float_registers[1] = 10.0;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
    }

    #[test]
    fn test_gt_floating_point_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.float_registers[0] = 20.0;
        test_vm.float_registers[1] = 10.0;
        test_vm.program = vec![29, 0, 1, 0, 29, 0, 1, 0, 29, 0, 1, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.float_registers[0] = 10.0;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
        test_vm.float_registers[0] = 5.0;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
    }

    #[test]
    fn test_gte_floating_point_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.float_registers[0] = 20.0;
        test_vm.float_registers[1] = 10.0;
        test_vm.program = vec![30, 0, 1, 0, 30, 0, 1, 0, 30, 0, 1, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.float_registers[0] = 10.0;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.float_registers[0] = 5.0;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
    }

    #[test]
    fn test_lt_floating_point_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.float_registers[0] = 20.0;
        test_vm.float_registers[1] = 10.0;
        test_vm.program = vec![31, 0, 1, 0, 31, 0, 1, 0, 31, 0, 1, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
        test_vm.float_registers[0] = 10.0;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
        test_vm.float_registers[0] = 5.0;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
    }

    #[test]
    fn test_lte_floating_point_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.float_registers[0] = 20.0;
        test_vm.float_registers[1] = 10.0;
        test_vm.program = vec![32, 0, 1, 0, 32, 0, 1, 0, 32, 0, 1, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
        test_vm.float_registers[0] = 10.0;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.float_registers[0] = 5.0;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
    }

    #[test]
    fn test_shl_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.program = vec![33, 0, 0, 0];
        assert_eq!(5, test_vm.registers[0]);
        test_vm.run_once();
        assert_eq!(327680, test_vm.registers[0]);
    }

    #[test]
    fn test_shr_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.program = vec![34, 0, 0, 0];
        assert_eq!(5, test_vm.registers[0]);
        test_vm.run_once();
        assert_eq!(0, test_vm.registers[0]);
    }

    #[test]
    fn test_and_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.program = vec![35, 0, 1, 2, 35, 0, 1, 2];
        test_vm.run_once();
        assert_eq!(test_vm.registers[2], 0);
        test_vm.registers[0] = 5;
        test_vm.registers[1] = 5;
        test_vm.run_once();
        assert_eq!(test_vm.registers[2], 5);
    }

    #[test]
    fn test_or_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.program = vec![36, 0, 1, 2, 36, 0, 1, 2];
        test_vm.run_once();
        assert_eq!(test_vm.registers[2], 15);
        test_vm.registers[0] = 5;
        test_vm.registers[1] = 5;
        test_vm.run_once();
        assert_eq!(test_vm.registers[2], 5);
    }

    #[test]
    fn test_xor_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.program = vec![37, 0, 1, 2, 37, 0, 1, 2];
        test_vm.run_once();
        assert_eq!(test_vm.registers[2], 15);
        test_vm.registers[0] = 5;
        test_vm.registers[1] = 5;
        test_vm.run_once();
        assert_eq!(test_vm.registers[2], 0);
    }

    #[test]
    fn test_not_opcode() {
        let mut test_vm = VM::get_test_vm();
        test_vm.program = vec![38, 0, 1, 2];
        test_vm.run_once();
        assert_eq!(test_vm.registers[1], -6);
    }

    #[test]
    fn test_lui_opcode() {
        let mut test_vm = VM::new();
        test_vm.program = vec![39, 0, 0, 1];
        test_vm.run_once();
        assert_eq!(test_vm.registers[0], 1);
    }

    #[test]
    fn test_cloop_opcode() {
        let mut test_vm = VM::new();
        test_vm.program = vec![40, 0, 10, 0];
        test_vm.run_once();
        assert_eq!(test_vm.loop_counter, 10);
    }

    #[test]
    fn test_loadm_opcode() {
        let mut test_vm = VM::new();
        test_vm.registers[0] = 10;
        test_vm.heap[10] = 100;
        test_vm.program = vec![42, 0, 1, 0];
        test_vm.run_once();
        assert_eq!(test_vm.registers[1], 100);
    }

    #[test]
    fn test_setm_opcode() {
        let mut test_vm = VM::new();
        test_vm.registers[0] = 10;
        test_vm.registers[1] = 200;
        test_vm.program = vec![43, 0, 1, 0];
        test_vm.run_once();
    }

    #[test]
    fn test_push_opcode() {
        let mut test_vm = VM::new();
        test_vm.registers[0] = 10;
        test_vm.program = vec![44, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.stack, vec![10, 0, 0, 0]);
    }

    #[test]
    fn test_pop_opcode() {
        let mut test_vm = VM::new();
        test_vm.stack.push(10);
        test_vm.stack.push(0);
        test_vm.stack.push(0);
        test_vm.stack.push(0);
        test_vm.program = vec![45, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.registers[0], 10);
    }
}
