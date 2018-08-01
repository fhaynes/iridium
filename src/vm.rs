pub struct VM {
    registers: [i32; 32]
}

impl VM {
    pub fn new() -> VM {
        VM {
            registers: [0; 32]
        }
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
}
