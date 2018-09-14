use std::thread;

use vm::{VMEvent, VM};

#[derive(Default)]
pub struct Scheduler {
    next_pid: u32,
    max_pid: u32,
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {
            next_pid: 0,
            max_pid: 50000,
        }
    }

    /// Takes a VM and runs it in a background thread
    pub fn get_thread(&mut self, mut vm: VM) -> thread::JoinHandle<Vec<VMEvent>> {
        thread::spawn(move || {
            let events = vm.run();
            println!("VM Events");
            println!("--------------------------");
            for event in &events {
                println!("{:#?}", event);
            }
            events
        })
    }

    fn _next_pid(&mut self) -> u32 {
        let result = self.next_pid;
        self.next_pid += 1;
        result
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_make_scheduler() {
        let s = Scheduler::new();
        assert_eq!(s.next_pid, 0);
    }
}
