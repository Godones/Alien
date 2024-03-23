use alloc::sync::Arc;
use constants::AlienError;
use constants::AlienResult;
use interface::{TaskDomain, TmpHeapInfo, VfsDomain};
use rref::RRef;

pub fn sys_brk(
    _vfs: &Arc<dyn VfsDomain>,
    task_domain: &Arc<dyn TaskDomain>,
    addr: usize,
) -> AlienResult<isize> {
    let heap_info = RRef::new(TmpHeapInfo::default());
    let heap_info = task_domain.heap_info(heap_info)?;
    if addr == 0 {
        return Ok(heap_info.current as isize);
    }
    if addr < heap_info.start || addr < heap_info.current {
        // panic!("heap can't be shrinked");
        return Err(AlienError::EINVAL);
    }
    task_domain.do_brk(addr)
}
