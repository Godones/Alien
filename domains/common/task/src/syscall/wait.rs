use alloc::sync::Arc;

use constants::{task::WaitOptions, AlienError, AlienResult};
use memory_addr::VirtAddr;
use task_meta::TaskStatus;

use crate::{processor::current_task, scheduler_domain};

pub fn do_wait4(
    pid: isize,
    exit_code_ptr: usize,
    options: u32,
    _rusage: usize,
) -> AlienResult<isize> {
    loop {
        let task = current_task().unwrap();
        let wait_task = task
            .inner()
            .children
            .values()
            .find(|child| child.pid() == pid as usize || pid == -1)
            .map(|task| task.clone());

        if wait_task.is_none() {
            return Err(AlienError::ECHILD);
        }
        let wait_options = WaitOptions::from_bits(options).unwrap();
        let wait_task = wait_task.unwrap();
        if wait_task.status() == TaskStatus::Terminated {
            let exit_code = wait_task.exit_code();
            if wait_options.contains(WaitOptions::WNOWAIT) {
                // recycle the task later
                if exit_code_ptr != 0 {
                    task.write_val_to_user(VirtAddr::from(exit_code_ptr), &exit_code)?;
                }
                assert_eq!(wait_task.pid(), wait_task.tid());
                return Ok(wait_task.pid() as _);
            } else {
                // recycle the task now
                task.inner().children.remove(&wait_task.pid());
                assert_eq!(
                    Arc::strong_count(&wait_task),
                    1,
                    "Father is [{}-{}], wait task is [{}-{}]",
                    task.pid(),
                    task.tid(),
                    wait_task.pid(),
                    wait_task.tid()
                );
            }
        }
        if wait_options.contains(WaitOptions::WNOHANG) {
            return Ok(0);
        } else {
            scheduler_domain!().yield_now().unwrap();
        }
    }
}
