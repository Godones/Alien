use core::arch::asm;

use crate::task::context::switch;
use crate::task::cpu::{current_cpu, TASK_MANAGER};
use crate::task::task::TaskState;

#[no_mangle]
pub fn first_into_user() -> ! {
    loop {
        {
            let mut task_manager = TASK_MANAGER.lock();
            let cpu = current_cpu();
            if cpu.task.is_some() {
                let task = cpu.task.take().unwrap();
                match task.state() {
                    TaskState::Sleeping | TaskState::Waiting => {
                        // drop(task);
                    }
                    TaskState::Zombie => {
                        task.update_state(TaskState::Terminated);
                    }
                    _ => {
                        task_manager.push_back(task);
                    }
                }
            }
        }
        let cpu = current_cpu();
        let mut task_manager = TASK_MANAGER.lock();
        if let Some(process) = task_manager.pop_front() {
            // warn!("switch to process {}", process.get_pid());
            // update state to running
            process.update_state(TaskState::Running);
            // get the process context
            let context = process.get_context_raw_ptr();
            cpu.task = Some(process);
            // switch to the process context
            let cpu_context = cpu.get_context_mut_raw_ptr();
            drop(task_manager);
            switch(cpu_context, context);
        } else {
            drop(task_manager);
            unsafe { asm!("wfi") }
        }
    }
}

pub fn schedule() {
    let cpu = current_cpu();
    let task = cpu.task.clone().unwrap();
    let cpu_context = cpu.get_context_raw_ptr();
    let context = task.get_context_mut_raw_ptr();
    drop(task);
    switch(context, cpu_context);
}
