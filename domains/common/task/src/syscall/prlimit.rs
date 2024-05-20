use basic::{constants::PrLimitResType, AlienResult};
use memory_addr::VirtAddr;

use crate::processor::current_task;

pub fn do_prlimit(
    pid: usize,
    resource: usize,
    new_limit: usize,
    old_limit: usize,
) -> AlienResult<isize> {
    let task = current_task().unwrap();
    assert!(pid == 0 || pid == task.pid());
    let resource = PrLimitResType::try_from(resource).unwrap();

    let task_inner = task.inner();
    let mut resource_limits = task_inner.resource_limits.lock();
    if old_limit != 0 {
        let limit = resource_limits.get_rlimit(resource);
        warn!("get rlimit nofile to {:?}", limit);
        task.write_val_to_user(VirtAddr::from(old_limit), limit)?;
    }
    if new_limit != 0 {
        let limit = task.read_val_from_user(VirtAddr::from(new_limit))?;
        warn!("set rlimit nofile to {:?}", limit);
        *resource_limits.get_rlimit_mut(resource) = limit;
    }
    Ok(0)
}
