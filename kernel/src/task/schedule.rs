//! CPU 调度
use alloc::sync::Arc;
use core::arch::asm;
use smpscheduler::FifoTask;

use constants::signal::SignalNumber;

use crate::ipc::send_signal;
use crate::task::context::switch;
use crate::task::cpu::current_cpu;
use crate::task::task::TaskState;
use crate::task::GLOBAL_TASK_MANAGER;
use crate::trap::check_timer_interrupt_pending;

/// 在 CPU 启动并初始化完毕后初次进入用户态时，或者在一个任务将要让渡 CPU 时 将会执行该函数。
///
/// 如果当前 CPU 上有任务正在执行，那么将根据该任务当前的状态进行操作。
/// - 如果该任务处于睡眠或等待状态，将会把其任务的控制块取出丢弃掉。
/// - 如果该任务处于僵尸状态，将会向其父进程发送信号，令其回收该任务的控制块。
/// - 如果该任务处于其他状态，我们将其放入线程池中等待下一次分配。
///
/// 之后如果在线程池中有任务需要调度，那么就把该任务的上下文切换到 CPU 上来运行；
/// 否则该 CPU 将进入等待状态，等待其它核的中断信号。
pub fn run_task() -> ! {
    loop {
        let cpu = current_cpu();
        if cpu.task.is_some() {
            let task = cpu.task.take().unwrap();
            match task.state() {
                TaskState::Waiting => {
                    // drop(task);
                }
                TaskState::Zombie => {
                    // 退出时向父进程发送信号，其中选项可被 sys_clone 控制
                    if task.send_sigchld_when_exit || task.pid == task.tid.0 {
                        let parent = task
                            .access_inner()
                            .parent
                            .as_ref()
                            .unwrap()
                            .upgrade()
                            .unwrap();
                        send_signal(parent.pid, SignalNumber::SIGCHLD as usize);
                    }
                    task.terminate();
                }
                _ => {
                    GLOBAL_TASK_MANAGER.add_task(Arc::new(FifoTask::new(task)));
                }
            }
        }
        if let Some(task) = GLOBAL_TASK_MANAGER.pick_next_task() {
            // if process.get_tid() >= 1 {
            //     warn!("switch to task {}", task.get_tid());
            // }
            // update state to running
            task.inner().update_state(TaskState::Running);
            // get the process context
            let context = task.inner().get_context_raw_ptr();
            cpu.task = Some(task.inner().clone());
            // switch to the process context
            let cpu_context = cpu.get_context_mut_raw_ptr();
            // warn!("switch to task {}", process.get_tid());
            drop(task);
            switch(cpu_context, context);
        } else {
            unsafe { asm!("wfi") }
        }
    }
}

/// 切换线程上下文，调度当前在 CPU 上执行的线程 让渡出 CPU
pub fn schedule() {
    check_timer_interrupt_pending();
    let cpu = current_cpu();
    let task = cpu.task.clone().unwrap();
    let cpu_context = cpu.get_context_raw_ptr();
    let context = task.get_context_mut_raw_ptr();
    drop(task);
    switch(context, cpu_context);
}
