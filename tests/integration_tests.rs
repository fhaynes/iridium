extern crate iridium;

mod commons;

#[test]
fn create_vm() {
    commons::setup();
    let mut vm = iridium::vm::VM::new();
    assert_eq!(vm.registers.len(), 32);
}
