use alloc::sync::Arc;

use constants::{AlienError, AlienResult};
use interface::{TaskDomain, VfsDomain};

pub fn sys_pipe2(
    task_domain: &Arc<dyn TaskDomain>,
    vfs: &Arc<dyn VfsDomain>,
    pipe: usize,
    _flag: usize,
) -> AlienResult<isize> {
    if pipe == 0 {
        return Err(AlienError::EINVAL);
    }
    let (r, w) = vfs.do_pipe2(_flag)?;
    task_domain.do_pipe2(r, w, pipe)
}
