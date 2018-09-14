#[macro_use]
extern crate criterion;
extern crate iridium;

use criterion::Criterion;
use iridium::assembler::{PIE_HEADER_LENGTH, PIE_HEADER_PREFIX};
use iridium::vm::VM;

mod arithmetic {
    use super::*;

    fn execute_add(c: &mut Criterion) {
        let clos = {
            let mut test_vm = VM::get_test_vm();
            test_vm.program = vec![1, 0, 1, 2];
            test_vm.run_once();
        };

        c.bench_function("execute_add", move |b| b.iter(|| clos));
    }

    fn execute_sub(c: &mut Criterion) {
        let clos = {
            let mut test_vm = VM::get_test_vm();
            test_vm.program = vec![2, 1, 0, 2];
            test_vm.run_once();
        };

        c.bench_function("execute_sub", move |b| b.iter(|| clos));
    }

    fn execute_mul(c: &mut Criterion) {
        let clos = {
            let mut test_vm = VM::get_test_vm();
            test_vm.program = vec![3, 0, 1, 2];
            test_vm.run_once();
        };

        c.bench_function("execute_mul", move |b| b.iter(|| clos));
    }

    fn execute_div(c: &mut Criterion) {
        let clos = {
            let mut test_vm = VM::get_test_vm();
            test_vm.program = vec![4, 1, 0, 2];
            test_vm.run_once();
        };

        c.bench_function("execute_div", move |b| b.iter(|| clos));
    }

    criterion_group!{
        name = arithmetic;
        config = Criterion::default();
        targets = execute_add, execute_sub, execute_mul, execute_div,
    }
}

criterion_main!(arithmetic::arithmetic);
