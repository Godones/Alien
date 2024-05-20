use alloc::sync::Arc;

use basic::{constants::PrLimitResType, AlienError, AlienResult};
use interface::TaskDomain;

pub fn sys_prlimit64(
    task_domain: &Arc<dyn TaskDomain>,
    pid: usize,
    resource: usize,
    new_limit: usize,
    old_limit: usize,
) -> AlienResult<isize> {
    PrLimitResType::try_from(resource).map_err(|_| AlienError::EINVAL)?;
    task_domain.do_prlimit(pid, resource, new_limit, old_limit)
}
