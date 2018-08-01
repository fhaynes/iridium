use instruction::{Opcode};

/// Virtual machine struct that will execute bytecode
pub struct VM {
    /// Array that simulates having hardware registers
    registers: [i32; 32],
    /// Program counter that tracks which byte is being executed
    pc: usize,
    /// The bytecode of the program being run
    program: Vec<u8>
}

impl VM {
    /// Creates and returns a new VM
    pub fn new() -> VM {
        VM {
            registers: [0; 32],
            program: vec![],
            pc: 0,
        }
    }

    /// Starts the primary execution loop
    pub fn run(&mut self) {
        loop {
            // If our program counter has exceeded the length of the program itself, something has
            // gone awry
            if self.pc >= self.program.len() {
                break;
            }
            match self.decode_opcode() {
                Opcode::HLT => {
                    println!("HLT encountered");
                    return;
                },
                _ => {
                    println!("Unrecognized opcode found! Terminating!");
                    return;
                }
            }
        }
    }

    /// Attempts to decode the byte the VM's program counter is pointing at into an opcode
    fn decode_opcode(&mut self) -> Opcode {
        let opcode = Opcode::from(self.program[self.pc]);
        self.pc += 1;
        return opcode;
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
    fn test_opcode_hlt() {
      let mut test_vm = VM::new();
      let test_bytes = vec![0,0,0,0];
      test_vm.program = test_bytes;
      test_vm.run();
      assert_eq!(test_vm.pc, 1);
    }

    #[test]
    fn test_opcode_igl() {
      let mut test_vm = VM::new();
      let test_bytes = vec![254,0,0,0];
      test_vm.program = test_bytes;
      test_vm.run();
      assert_eq!(test_vm.pc, 1);
    }

}
