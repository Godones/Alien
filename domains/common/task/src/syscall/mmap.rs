use alloc::{boxed::Box, vec};

use basic::vm::frame::FrameTracker;
use config::FRAME_SIZE;
use constants::{
    io::{MMapFlags, MMapType, ProtFlags, MMAP_TYPE_MASK},
    AlienError, AlienResult,
};
use memory_addr::{align_down_4k, align_up_4k};
use page_table::MappingFlags;
use ptable::{PhysPage, VmArea, VmAreaType};

use crate::{elf::FrameTrackerWrapper, processor::current_task, resource::MMapRegion};

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
    let mut start = align_down_4k(start);
    let len = align_up_4k(len);
    let mut mmap = task.mmap.lock();

    let v_range = if prot.contains(ProtFlags::PROT_EXEC) {
        if start > task.heap.lock().start {
            // the mmap region is in heap
            return Err(AlienError::EINVAL);
        }
        if let Some(_region) = mmap.get_region(start) {
            return Err(AlienError::EINVAL);
        }
        if start == 0 {
            start = 0x1000;
        }
        start..start + len
    } else if flags.contains(MMapFlags::MAP_FIXED) {
        if start > task.heap.lock().start {
            error!("mmap fixed address conflict with heap");
            return Err(AlienError::EINVAL);
        }
        // check if the region is already mapped
        if let Some(region) = mmap.get_region(start).map(|region| region.clone()) {
            // split the region
            let (left, mut right) = region.split(start);
            // delete the old region
            mmap.remove_region(region.start);
            // add the left region
            mmap.add_region(left);
            if start + len < right.start + right.map_len {
                // slice the right region
                trace!(
                    "again slice the right region:{:#x?}, len:{:#x}",
                    right.start,
                    right.len
                );
                let (mut left, right) = right.split(start + len);
                // add the right region
                mmap.add_region(right);
                // update prot and flags
                left.set_prot(prot);
                left.set_flags(flags);
                left.offset = offset;
                left.fd = fd;
                mmap.add_region(left);
            } else {
                trace!(
                    "directly add the right region:{:#x?}, len:{:#x}",
                    right.start,
                    right.len
                );
                // update prot and flags
                right.set_prot(prot);
                right.set_flags(flags);
                right.offset = offset;
                right.fd = fd;
                mmap.add_region(right);
            }
            return Ok(start as isize);
        }
        start..start + len
    } else {
        let v_range = mmap.alloc(len);
        v_range
    };
    let region = MMapRegion::new(
        v_range.start,
        len,
        v_range.end - v_range.start,
        prot,
        flags,
        fd,
        offset,
    );
    // warn!("add mmap region:{:#x?}",region);
    mmap.add_region(region);
    let start = v_range.start;
    let map_flags = from_prot(prot); // no V  flag

    let mut phy_frames = vec![];
    for _ in 0..len / FRAME_SIZE {
        let frame = FrameTracker::new(1);
        phy_frames.push(Box::new(FrameTrackerWrapper(frame)) as Box<dyn PhysPage>);
    }
    let area = VmArea::new(v_range, map_flags, phy_frames);

    task.address_space
        .lock()
        .map(VmAreaType::VmArea(area))
        .unwrap();
    Ok(start as isize)
}

pub fn do_munmap(start: usize, len: usize) -> AlienResult<isize> {
    info!("munmap start:{:#x}, len:{:#x}", start, len);
    let task = current_task().unwrap();
    let mut mmap = task.mmap.lock();
    let x = mmap.get_region(start);
    if x.is_none() {
        return Err(AlienError::EINVAL);
    }
    // now we need make sure the start is equal to the start of the region, and the len is equal to the len of the region
    let region = x.unwrap();
    if region.start != start || len != region.len {
        return Err(AlienError::EINVAL);
    }
    task.address_space.lock().unmap(start).unwrap();
    mmap.remove_region(start);
    Ok(0)
}

fn from_prot(prot_flags: ProtFlags) -> MappingFlags {
    let mut perm = MappingFlags::USER;
    if prot_flags.contains(ProtFlags::PROT_READ) {
        perm |= MappingFlags::READ;
    }
    if prot_flags.contains(ProtFlags::PROT_WRITE) {
        perm |= MappingFlags::WRITE;
    }
    if prot_flags.contains(ProtFlags::PROT_EXEC) {
        perm |= MappingFlags::EXECUTE;
    }
    perm
}
