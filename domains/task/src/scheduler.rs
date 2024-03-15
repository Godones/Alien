use crate::processor::{add_task, current_cpu, current_task, pick_next_task, take_current_task};
use crate::task::{Task, TaskState};
use alloc::sync::Arc;
use core::hint::spin_loop;

pub fn run_task() -> ! {
    loop {
        let mut cpu = current_cpu();
        if let Some(task) = pick_next_task() {
            // update state to running
            task.update_state(TaskState::Running);
            // get the process context
            let context = task.get_context_raw_ptr();
            cpu.task = Some(task);
            // switch to the process context
            let cpu_context = cpu.get_idle_task_cx_ptr();
            // println!("hart {} switch to task {}", hart_id(),task.get_tid());
            drop(cpu);
            libsyscall::switch_task(cpu_context, context);
        } else {
            spin_loop();
        }
    }
}

pub fn schedule() {
    let task = take_current_task().unwrap();
    schedule_now(task)
}

pub fn schedule_now(task: Arc<Task>) {
    let context = task.get_context_mut_raw_ptr();
    match task.state() {
        TaskState::Waiting => {
            drop(task);
        }
        TaskState::Zombie => {
            // 退出时向父进程发送信号，其中选项可被 sys_clone 控制
            // if task.send_sigchld_when_exit || task.pid == task.tid.0 {
            //     let parent = task
            //         .access_inner()
            //         .parent
            //         .as_ref()
            //         .unwrap()
            //         .upgrade()
            //         .unwrap();
            //     // send_signal(parent.pid, SignalNumber::SIGCHLD as usize);
            // }
            // task.terminate(); // release some resources
        }
        _ => {
            // println!("add task to scheduler");
            add_task(task);
        }
    }
    let mut cpu = current_cpu();
    let cpu_context = cpu.get_idle_task_cx_ptr();
    drop(cpu);
    libsyscall::switch_task(context, cpu_context);
}

pub fn do_suspend() -> isize {
    {
        let task = current_task().unwrap();
        // task.access_inner().update_timer();
        // check_task_timer_expired();
        task.update_state(TaskState::Ready);
    }
    schedule();
    0
}
