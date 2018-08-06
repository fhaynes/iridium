/// Represents an opcode, which tells our interpreter what to do with the following operands
/// Opcodes are a nice way to represent each of our Opcodes
#[derive(Debug, PartialEq)]
pub enum Opcode {
    LOAD,
    ADD,
    SUB,
    MUL,
    DIV,
    HLT,
    IGL,
    JMP,
    JMPF,
    JMPB,
}

/// We implement this trait to make it easy to convert from a u8 to an Opcode
impl From<u8> for Opcode {
    fn from(v: u8) -> Self {
        match v {
            0 => Opcode::LOAD,
            1 => Opcode::ADD,
            2 => Opcode::SUB,
            3 => Opcode::MUL,
            4 => Opcode::DIV,
            6 => Opcode::HLT,
            7 => Opcode::JMP,
            8 => Opcode::JMPF,
            9 => Opcode::JMPB,
            _ => Opcode::IGL,
        }
    }
}

/// Represents a combination of an opcode and operands for the VM to execute
#[derive(Debug, PartialEq)]
pub struct Instruction {
  opcode: Opcode
}

impl Instruction {
    /// Creates and returns a new Instruction
    pub fn new(opcode: Opcode) -> Instruction {
        Instruction {
            opcode: opcode
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_hlt() {
        let opcode = Opcode::HLT;
        assert_eq!(opcode, Opcode::HLT);
    }

    #[test]
    fn test_create_instruction() {
      let instruction = Instruction::new(Opcode::HLT);
      assert_eq!(instruction.opcode, Opcode::HLT);
    }
}
