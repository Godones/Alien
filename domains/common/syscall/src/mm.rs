use alloc::sync::Arc;

use basic::config::FRAME_SIZE;
use constants::{
    io::{MMapFlags, MMapType, ProtFlags, MMAP_TYPE_MASK},
    AlienError, AlienResult,
};
use interface::{TaskDomain, TmpHeapInfo, VfsDomain};
use log::info;
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

pub fn sys_mmap(
    task_domain: &Arc<dyn TaskDomain>,
    addr: usize,
    len: usize,
    prot: usize,
    flags: usize,
    fd: usize,
    offset: usize,
) -> AlienResult<isize> {
    if offset % FRAME_SIZE != 0 {
        return Err(AlienError::EINVAL);
    }
    let prot = ProtFlags::from_bits_truncate(prot as _);
    let _ty = MMapType::try_from((flags as u32 & MMAP_TYPE_MASK) as u8)
        .map_err(|_| AlienError::EINVAL)?;
    let flags = MMapFlags::from_bits_truncate(flags as u32);

    if flags.contains(MMapFlags::MAP_ANONYMOUS) && offset != 0 {
        return Err(AlienError::EINVAL);
    }

    info!(
        "mmap: start: {:#x}, len: {:#x}, prot: {:?}, flags: {:?}, fd: {}, offset: {:#x}",
        addr, len, prot, flags, fd, offset
    );
    task_domain
        .do_mmap(addr, len, prot.bits(), flags.bits(), fd, offset)
        .map(|addr| addr)
}
