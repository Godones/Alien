use alloc::sync::Arc;
use constants::AlienError;
use interface::{TaskDomain, TmpHeapInfo, VfsDomain};
use rref::{RRef, RpcError, RpcResult};

pub fn sys_brk(
    _vfs: &Arc<dyn VfsDomain>,
    task_domain: &Arc<dyn TaskDomain>,
    addr: usize,
) -> RpcResult<isize> {
    let heap_info = RRef::new(TmpHeapInfo::default());
    let heap_info = task_domain.heap_info(heap_info)?;
    if addr == 0 {
        return Ok(heap_info.current as isize);
    }
    if addr < heap_info.start || addr < heap_info.current {
        // panic!("heap can't be shrinked");
        return Err(RpcError::Alien(AlienError::EINVAL));
    }
    task_domain.brk(addr)
}
