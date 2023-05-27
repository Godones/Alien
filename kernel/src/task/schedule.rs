use core::arch::asm;

use crate::task::context::switch;
use crate::task::cpu::{current_cpu, PROCESS_MANAGER};
use crate::task::process::ProcessState;

#[no_mangle]
pub fn first_into_user() -> ! {
    loop {
        let cpu = current_cpu();
        let mut process_manager = PROCESS_MANAGER.lock();
        if let Some(process) = process_manager.pop_front() {
            // warn!("switch to process {}", process.get_pid());
            // update state to running
            process.update_state(ProcessState::Running);
            // get the process context
            let context = process.get_context_raw_ptr();
            cpu.process = Some(process);
            // switch to the process context
            let cpu_context = cpu.get_context_mut_raw_ptr();
            drop(process_manager);
            switch(cpu_context, context);
        } else {
            drop(process_manager);
            unsafe { asm!("wfi") }
        }
    }
}

pub fn schedule() {
    let mut process_manager = PROCESS_MANAGER.lock();
    // println!("There are {} processes in the process pool", process_manager.len());
    let cpu = current_cpu();
    let process = cpu.take_process().unwrap();
    match process.state() {
        ProcessState::Zombie | ProcessState::Sleeping | ProcessState::Waiting => {}
        _ => {
            process_manager.push_back(process.clone());
        }
    }
    let cpu_context = cpu.get_context_raw_ptr();
    let context = process.get_context_mut_raw_ptr();
    drop(process);
    drop(process_manager);
    switch(context, cpu_context);
}
