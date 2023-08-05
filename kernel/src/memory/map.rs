use alloc::sync::Arc;
use alloc::vec::Vec;
use core::ops::Range;

use bitflags::bitflags;
use page_table::addr::{align_up_4k, VirtAddr};
use page_table::pte::MappingFlags;

use syscall_define::io::MapFlags;
use syscall_define::LinuxErrno;
use syscall_table::syscall_func;

use crate::config::{FRAME_SIZE, PROCESS_HEAP_MAX};
use crate::fs::file::KFile;
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
    pub fd: Option<Arc<KFile>>,
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

    pub fn get_region_mut(&mut self, addr: usize) -> Option<&mut MMapRegion> {
        for region in self.regions.iter_mut() {
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
        fd: Option<Arc<KFile>>,
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
    // [a-b]
    // [a-c] [c-b]
    pub fn split(&self, addr: usize) -> (Self, Self) {
        let mut region1 = self.clone();
        let mut region2 = self.clone();
        region1.len = addr - self.start;
        region1.map_len = align_up_4k(region1.len);
        region2.start = addr;
        region2.len = self.start + self.len - addr;
        region2.map_len = align_up_4k(region2.len);
        region2.offset += region1.len;
        (region1, region2)
    }

    pub fn set_prot(&mut self, prot: ProtFlags) {
        self.prot = prot;
    }
    pub fn set_flags(&mut self, flags: MapFlags) {
        self.flags = flags;
    }
}

#[syscall_func(215)]
pub fn do_munmap(start: usize, len: usize) -> isize {
    let task = current_task().unwrap();
    let res = task.access_inner().unmap(start, len);
    if res.is_err() {
        return res.err().unwrap();
    }
    0
}

/// Reference: https://man7.org/linux/man-pages/man2/mmap.2.html
#[syscall_func(222)]
pub fn do_mmap(start: usize, len: usize, prot: u32, flags: u32, fd: usize, offset: usize) -> isize {
    let process = current_task().unwrap();
    let mut process_inner = process.access_inner();
    let prot = ProtFlags::from_bits_truncate(prot);
    let flags = MapFlags::from_bits_truncate(flags);
    warn!(
        "mmap: start: {:#x}, len: {:#x}, prot: {:?}, flags: {:?}, fd: {}, offset: {:#x}",
        start, len, prot, flags, fd, offset
    );
    let res = process_inner.add_mmap(start, len, prot, flags, fd, offset);
    if res.is_err() {
        return res.err().unwrap();
    }
    res.unwrap() as isize
}

#[syscall_func(226)]
pub fn map_protect(start: usize, len: usize, prot: u32) -> isize {
    let process = current_task().unwrap();
    let mut process_inner = process.access_inner();
    let prot = ProtFlags::from_bits_truncate(prot);
    warn!(
        "mprotect: start: {:#x}, len: {:#x}, prot: {:?}",
        start, len, prot
    );
    let res = process_inner.map_protect(start, len, prot);
    if res.is_err() {
        return -1;
    }
    0
}

#[syscall_func(227)]
pub fn msync(addr: usize, len: usize, flags: usize) -> isize {
    warn!(
        "msync: addr: {:#x}, len: {:#x}, flags: {:#x}",
        addr, len, flags
    );
    let task = current_task().unwrap();
    let address_space = &task.access_inner().address_space;
    let res = address_space.lock().query(VirtAddr::from(addr));
    // warn!("msync: res: {:?}", res);
    if res.is_err() {
        return LinuxErrno::EFAULT as isize;
    }
    0
}

#[syscall_func(233)]
pub fn madvise(addr: usize, len: usize, advice: usize) -> isize {
    warn!(
        "madvise: addr: {:#x}, len: {:#x}, advice: {:#x}",
        addr, len, advice
    );
    0
}
