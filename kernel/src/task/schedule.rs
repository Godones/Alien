use crate::task::context::__switch;
use crate::task::cpu::{current_cpu, PROCESS_MANAGER};
use crate::task::process::ProcessState;

pub fn first_into_user() -> ! {
    let cpu = current_cpu();
    loop {
        let mut process_manager = PROCESS_MANAGER.lock();
        if let Some(process) = process_manager.pop_front() {
            // update state to running
            process.update_state(ProcessState::Running);
            // get the process context
            let context = process.get_context_raw_ptr();
            cpu.process = Some(process);
            // switch to the process context
            let cpu_context = cpu.get_context_raw_ptr();
            drop(process_manager);
            __switch(cpu_context, context);
        }
    }
}

pub fn schedule() {
    let mut process_manager = PROCESS_MANAGER.lock();
    let cpu = current_cpu();
    let process = cpu.take_process().unwrap();
    assert_ne!(process.state(), ProcessState::Running);
    match process.state() {
        ProcessState::Zombie => {
            drop(process);
        }
        _ => {
            process_manager.push_back(process);
        }
    }
    if let Some(process) = process_manager.pop_front() {
        process.update_state(ProcessState::Running);
        let context = process.get_context_raw_ptr();
        cpu.process = Some(process);
        let cpu_context = cpu.get_context_raw_ptr();
        drop(process_manager);
        __switch(cpu_context, context);
    }
}
