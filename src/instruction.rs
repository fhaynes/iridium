/// Represents an opcode, which tells our interpreter what to do with the following operands
#[derive(Debug, PartialEq)]
pub enum Opcode {
  HLT,
  IGL
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

impl From<u8> for Opcode {
    fn from(v: u8) -> Self {
        match v {
            0 => return Opcode::HLT,
            _ => return Opcode::IGL
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
