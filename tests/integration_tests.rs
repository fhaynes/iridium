extern crate iridium;

mod commons;

#[test]
fn create_vm() {
    commons::setup();
    let mut vm = iridium::vm::VM::new();
    assert_eq!(vm.registers.len(), 32);
}

#[test]
fn test_call_return() {
    commons::setup();
    let mut vm = iridium::vm::VM::new();
    let mut asm = iridium::assembler::Assembler::new();
    let code = r"
    .data
    .code
    LOAD $0 #400
    CALL @test
    HLT
    test: LOAD $0 #500
    RET";
    let program = asm.assemble(code);
    vm.add_bytes(program.unwrap());
    let events = vm.run();
    assert_eq!(events[1].event.stop_code(), 0);
}

#[test]
fn test_inc_loop() {
    commons::setup();
    let mut vm = iridium::vm::VM::new();
    let mut asm = iridium::assembler::Assembler::new();
    let code = r"
    .data
    .code
    cloop #10
    load $1 #20
    test: inc $1
    loop @test
    hlt";
    let program = asm.assemble(code);
    vm.add_bytes(program.unwrap());
    let events = vm.run();
    assert_eq!(events[1].event.stop_code(), 0);
    assert_eq!(vm.registers[1], 31);
}

#[test]
fn test_hlt() {
    commons::setup();
    let mut vm = iridium::vm::VM::new();
    let mut asm = iridium::assembler::Assembler::new();
    let code = r"
    .data
    .code
    hlt";
    let program = asm.assemble(code);
    vm.add_bytes(program.unwrap());
    let events = vm.run();
    assert_eq!(events[1].event.stop_code(), 0);
}
