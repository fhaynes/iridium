use nom::types::CompleteStr;

/// Represents an opcode, which tells our interpreter what to do with the following operands
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Opcode {
    LOAD,
    ADD,
    SUB,
    MUL,
    DIV,
    HLT,
    JMP,
    JMPF,
    JMPB,
    EQ,
    NEQ,
    GTE,
    LTE,
    LT,
    GT,
    JMPE,
    NOP,
    ALOC,
    INC,
    DEC,
    DJMPE,
    IGL,
    PRTS,
    LOADF64,
    ADDF64,
    SUBF64,
    MULF64,
    DIVF64,
    EQF64,
    NEQF64,
    GTF64,
    GTEF64,
    LTF64,
    LTEF64,
    SHL,
    SHR,
    AND,
    OR,
    XOR,
    NOT,
    LUI,
    CLOOP,
    LOOP,
    LOADM,
    SETM,
    PUSH,
    POP,
    CALL,
    RET,
}

impl From<Opcode> for u8 {
    fn from(op: Opcode) -> Self {
        match op {
            Opcode::LOAD => 0,
            Opcode::ADD => 1,
            Opcode::SUB => 2,
            Opcode::MUL => 3,
            Opcode::DIV => 4,
            Opcode::HLT => 5,
            Opcode::JMP => 6,
            Opcode::JMPF => 7,
            Opcode::JMPB => 8,
            Opcode::EQ => 9,
            Opcode::NEQ => 10,
            Opcode::GTE => 11,
            Opcode::LTE => 12,
            Opcode::LT => 13,
            Opcode::GT => 14,
            Opcode::JMPE => 15,
            Opcode::NOP => 16,
            Opcode::ALOC => 17,
            Opcode::INC => 18,
            Opcode::DEC => 19,
            Opcode::DJMPE => 20,
            Opcode::PRTS => 21,
            Opcode::LOADF64 => 22,
            Opcode::ADDF64 => 23,
            Opcode::SUBF64 => 24,
            Opcode::MULF64 => 25,
            Opcode::DIVF64 => 26,
            Opcode::EQF64 => 27,
            Opcode::NEQF64 => 28,
            Opcode::GTF64 => 29,
            Opcode::GTEF64 => 30,
            Opcode::LTF64 => 31,
            Opcode::LTEF64 => 32,
            Opcode::SHL => 33,
            Opcode::SHR => 34,
            Opcode::AND => 35,
            Opcode::OR => 36,
            Opcode::XOR => 37,
            Opcode::NOT => 38,
            Opcode::LUI => 39,
            Opcode::CLOOP => 40,
            Opcode::LOOP => 41,
            Opcode::LOADM => 42,
            Opcode::SETM => 43,
            Opcode::PUSH => 44,
            Opcode::POP => 45,
            Opcode::CALL => 46,
            Opcode::RET => 47,
            Opcode::IGL => 100,
        }
    }
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
            5 => Opcode::HLT,
            6 => Opcode::JMP,
            7 => Opcode::JMPF,
            8 => Opcode::JMPB,
            9 => Opcode::EQ,
            10 => Opcode::NEQ,
            11 => Opcode::GTE,
            12 => Opcode::LTE,
            13 => Opcode::LT,
            14 => Opcode::GT,
            15 => Opcode::JMPE,
            16 => Opcode::NOP,
            17 => Opcode::ALOC,
            18 => Opcode::INC,
            19 => Opcode::DEC,
            20 => Opcode::DJMPE,
            21 => Opcode::PRTS,
            22 => Opcode::LOADF64,
            23 => Opcode::ADDF64,
            24 => Opcode::SUBF64,
            25 => Opcode::MULF64,
            26 => Opcode::DIVF64,
            27 => Opcode::EQF64,
            28 => Opcode::NEQF64,
            29 => Opcode::GTF64,
            30 => Opcode::GTEF64,
            31 => Opcode::LTF64,
            32 => Opcode::LTEF64,
            33 => Opcode::SHL,
            34 => Opcode::SHR,
            35 => Opcode::AND,
            36 => Opcode::OR,
            37 => Opcode::XOR,
            38 => Opcode::NOT,
            39 => Opcode::LUI,
            40 => Opcode::CLOOP,
            41 => Opcode::LOOP,
            42 => Opcode::LOADM,
            43 => Opcode::SETM,
            44 => Opcode::PUSH,
            45 => Opcode::POP,
            46 => Opcode::CALL,
            47 => Opcode::RET,
            _ => Opcode::IGL,
        }
    }
}

/// Convenience function to convert nom CompleteStr into an opcode
impl<'a> From<CompleteStr<'a>> for Opcode {
    fn from(v: CompleteStr<'a>) -> Self {
        let lowercased_opcode = v.to_lowercase();
        match CompleteStr(&lowercased_opcode) {
            CompleteStr("load") => Opcode::LOAD,
            CompleteStr("add") => Opcode::ADD,
            CompleteStr("sub") => Opcode::SUB,
            CompleteStr("mul") => Opcode::MUL,
            CompleteStr("div") => Opcode::DIV,
            CompleteStr("hlt") => Opcode::HLT,
            CompleteStr("jmp") => Opcode::JMP,
            CompleteStr("jmpf") => Opcode::JMPF,
            CompleteStr("jmpb") => Opcode::JMPB,
            CompleteStr("eq") => Opcode::EQ,
            CompleteStr("neq") => Opcode::NEQ,
            CompleteStr("gte") => Opcode::GTE,
            CompleteStr("gt") => Opcode::GT,
            CompleteStr("lte") => Opcode::LTE,
            CompleteStr("lt") => Opcode::LT,
            CompleteStr("jmpe") => Opcode::JMPE,
            CompleteStr("nop") => Opcode::NOP,
            CompleteStr("aloc") => Opcode::ALOC,
            CompleteStr("inc") => Opcode::INC,
            CompleteStr("dec") => Opcode::DEC,
            CompleteStr("djmpe") => Opcode::DJMPE,
            CompleteStr("prts") => Opcode::PRTS,
            CompleteStr("loadf64") => Opcode::LOADF64,
            CompleteStr("addf64") => Opcode::ADDF64,
            CompleteStr("subf64") => Opcode::SUBF64,
            CompleteStr("mulf64") => Opcode::MULF64,
            CompleteStr("divf64") => Opcode::DIVF64,
            CompleteStr("eqf64") => Opcode::EQF64,
            CompleteStr("neqf64") => Opcode::NEQF64,
            CompleteStr("gtf64") => Opcode::GTF64,
            CompleteStr("gtef64") => Opcode::GTEF64,
            CompleteStr("ltf64") => Opcode::LTF64,
            CompleteStr("ltef64") => Opcode::LTEF64,
            CompleteStr("shl") => Opcode::SHL,
            CompleteStr("shr") => Opcode::SHR,
            CompleteStr("and") => Opcode::AND,
            CompleteStr("or") => Opcode::OR,
            CompleteStr("xor") => Opcode::XOR,
            CompleteStr("not") => Opcode::NOT,
            CompleteStr("lui") => Opcode::LUI,
            CompleteStr("cloop") => Opcode::CLOOP,
            CompleteStr("loop") => Opcode::LOOP,
            CompleteStr("loadm") => Opcode::LOADM,
            CompleteStr("setm") => Opcode::SETM,
            CompleteStr("push") => Opcode::PUSH,
            CompleteStr("pop") => Opcode::POP,
            CompleteStr("call") => Opcode::CALL,
            CompleteStr("ret") => Opcode::RET,
            _ => Opcode::IGL,
        }
    }
}

/// Represents a combination of an opcode and operands for the VM to execute
#[derive(Debug, PartialEq)]
pub struct Instruction {
    opcode: Opcode,
}

impl Instruction {
    /// Creates and returns a new Instruction
    pub fn new(opcode: Opcode) -> Instruction {
        Instruction { opcode }
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

    #[test]
    fn test_str_to_opcode() {
        let opcode = Opcode::from(CompleteStr("load"));
        assert_eq!(opcode, Opcode::LOAD);
        let opcode = Opcode::from(CompleteStr("Load"));
        assert_eq!(opcode, Opcode::LOAD);
        let opcode = Opcode::from(CompleteStr("illegal"));
        assert_eq!(opcode, Opcode::IGL);
        let opcode = Opcode::from(CompleteStr("CLOOP"));
        assert_eq!(opcode, Opcode::CLOOP);
        let opcode = Opcode::from(CompleteStr("loop"));
        assert_eq!(opcode, Opcode::LOOP);
    }

    #[test]
    fn test_int_to_opcode() {
        let opcode = Opcode::from(41);
        assert_eq!(opcode, Opcode::LOOP);
    }
}
