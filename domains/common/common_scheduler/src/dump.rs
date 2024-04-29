use alloc::{vec, vec::Vec};

use basic::println;
use config::{FRAME_SIZE, USER_KERNEL_STACK_SIZE};
use interface::{CpuLocalData, KStackData, SchedulerDataContainer, TaskData};

use crate::{processor::current_cpu, scheduler::TASK_WAIT_QUEUE};

pub fn dump_meta_data(data: &mut SchedulerDataContainer) {
    let cpu = current_cpu();
    let cpu_context = cpu.context;
    let task = cpu.current().map(|task| {
        let task = task.lock();
        let kstack_data = KStackData {
            kstack_top: task.kstack.top().as_usize(),
            pages: USER_KERNEL_STACK_SIZE / FRAME_SIZE,
        };
        TaskData {
            task_meta: task.meta.clone(),
            kstack_data,
        }
    });
    println!("dump cpu data:{:?}", cpu_context);
    let task_wait_queue = TASK_WAIT_QUEUE
        .lock()
        .iter()
        .map(|(_, task)| {
            let task = task.lock();
            let kstack_data = KStackData {
                kstack_top: task.kstack.top().as_usize(),
                pages: USER_KERNEL_STACK_SIZE / FRAME_SIZE,
            };
            TaskData {
                task_meta: task.meta.clone(),
                kstack_data,
            }
        })
        .collect::<Vec<TaskData>>();
    println!("dump task_wait_queue:{:?}", task_wait_queue);
    let task_ready_queue = vec![];
    let cpu_local = CpuLocalData { cpu_context, task };
    data.cpu_local = cpu_local;
    data.task_wait_queue.clone_from_slice(&task_wait_queue);
    data.task_ready_queue.clone_from_slice(&task_ready_queue);
}
