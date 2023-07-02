use alloc::sync::Arc;
use alloc::vec::Vec;
use core::ops::Range;

use bitflags::bitflags;
use page_table::pte::MappingFlags;
use rvfs::file::File;

use syscall_table::syscall_func;

use crate::config::{FRAME_SIZE, PROCESS_HEAP_MAX};
use crate::task::current_task;

bitflags! {
    pub struct ProtFlags: u32 {
        const PROT_NONE = 0x0;
        const PROT_READ = 0x1;
        const PROT_WRITE = 0x2;
        const PROT_EXEC = 0x4;
    }
}

impl Into<MappingFlags> for ProtFlags {
    fn into(self) -> MappingFlags {
        let mut perm = MappingFlags::empty();
        if self.contains(ProtFlags::PROT_READ) {
            perm |= MappingFlags::R;
        }
        if self.contains(ProtFlags::PROT_WRITE) {
            perm |= MappingFlags::W;
        }
        if self.contains(ProtFlags::PROT_EXEC) {
            perm |= MappingFlags::X;
        }
        perm |= MappingFlags::U;
        perm
    }
}

bitflags! {
    pub struct MapFlags: u32 {
        const MAP_SHARED = 0x01;
        const MAP_PRIVATE = 0x02;
        const MAP_FIXED = 0x10;
        const MAP_ANONYMOUS = 0x20;
    }
}

#[derive(Debug, Clone)]
/// The Process should manage the mmap info
pub struct MMapInfo {
    /// The start address of the mmap, it is a constant
    map_start: usize,
    /// The regions of the mmap
    regions: Vec<MMapRegion>,
}

#[derive(Debug, Clone)]
pub struct MMapRegion {
    /// The start address of the mapping
    pub start: usize,
    /// The length of the mapping
    pub len: usize,
    pub map_len: usize,
    /// The protection flags of the mapping
    pub prot: ProtFlags,
    /// The flags of the mapping
    pub flags: MapFlags,
    /// The file descriptor to map
    pub fd: Arc<File>,
    /// The offset in the file to start from
    pub offset: usize,
}

impl MMapInfo {
    pub fn new() -> Self {
        Self {
            map_start: PROCESS_HEAP_MAX,
            regions: Vec::new(),
        }
    }

    pub fn alloc(&mut self, len: usize) -> Range<usize> {
        let addr = self.map_start;
        self.map_start += len;
        // align to Frame size
        self.map_start = (self.map_start + FRAME_SIZE - 1) & !(FRAME_SIZE - 1);
        addr..self.map_start
    }

    pub fn add_region(&mut self, region: MMapRegion) {
        self.regions.push(region);
    }

    pub fn get_region(&self, addr: usize) -> Option<&MMapRegion> {
        for region in self.regions.iter() {
            if region.start <= addr && addr < region.start + region.len {
                return Some(region);
            }
        }
        None
    }

    pub fn remove_region(&mut self, addr: usize) {
        let mut index = 0;
        for region in self.regions.iter() {
            if region.start <= addr && addr < region.start + region.len {
                break;
            }
            index += 1;
        }
        self.regions.remove(index);
    }
}

impl MMapRegion {
    pub fn new(
        start: usize,
        len: usize,
        map_len: usize,
        prot: ProtFlags,
        flags: MapFlags,
        fd: Arc<File>,
        offset: usize,
    ) -> Self {
        Self {
            start,
            len,
            map_len,
            prot,
            flags,
            fd,
            offset,
        }
    }
}

#[syscall_func(215)]
pub fn do_munmap(start: usize, len: usize) -> isize {
    let process = current_task().unwrap();
    let res = process.access_inner().unmap(start, len);
    if res.is_err() {
        return -1;
    }
    0
}

/// #Reference: https://man7.org/linux/man-pages/man2/mmap.2.html
#[syscall_func(222)]
pub fn do_mmap(start: usize, len: usize, prot: u32, flags: u32, fd: usize, offset: usize) -> isize {
    let process = current_task().unwrap();
    let mut process_inner = process.access_inner();
    let prot = ProtFlags::from_bits_truncate(prot);
    let flags = MapFlags::from_bits_truncate(flags);

    let res = process_inner.add_mmap(start, len, prot, flags, fd, offset);
    if res.is_err() {
        return -1;
    }
    res.unwrap() as isize
}
