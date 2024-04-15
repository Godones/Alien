use constants::{
    io::{MMapFlags, MMapType, ProtFlags, MMAP_TYPE_MASK},
    AlienError, AlienResult,
};

use crate::processor::current_task;

pub fn do_mmap(
    start: usize,
    len: usize,
    prot: u32,
    flags: u32,
    fd: usize,
    offset: usize,
) -> AlienResult<isize> {
    let prot = ProtFlags::from_bits_truncate(prot as _);
    let _ty = MMapType::try_from((flags & MMAP_TYPE_MASK) as u8).map_err(|_| AlienError::EINVAL)?;
    let flags = MMapFlags::from_bits_truncate(flags);

    if start == 0 && flags.contains(MMapFlags::MAP_FIXED) {
        return Err(AlienError::EINVAL);
    }
    let task = current_task().unwrap();
    // if the map in heap, now we ignore it
    if task.heap.lock().contains(start) && task.heap.lock().contains(start + len) {
        return Ok(start as _);
    }
    let fd = if flags.contains(MMapFlags::MAP_ANONYMOUS) {
        None
    } else {
        let file = task.get_file(fd).ok_or(AlienError::EBADF)?; // EBADF
        Some(file)
    };
    // let mut start = align_down_4k(start);
    // let v_range = if prot.contains(ProtFlags::PROT_EXEC) {
    //     let len = align_up_4k(len);
    //     if let Some(_region) = task.mmap.get_region(start) {
    //         return Err(AlienError::EINVAL);
    //     }
    //     if start == 0 {
    //         start = 0x1000;
    //     }
    //     start..start + len
    // }
    // Ok()
    Err(AlienError::EINVAL)
}
