use alloc::sync::Arc;
use alloc::vec::Vec;
use core::ops::Range;

use bitflags::bitflags;
use page_table::addr::{align_up_4k, VirtAddr};
use page_table::pte::MappingFlags;

use constants::io::MapFlags;
use constants::LinuxErrno;
use syscall_table::syscall_func;

use config::{FRAME_SIZE, PROCESS_HEAP_MAX};
use crate::fs::file::File;
use crate::task::current_task;
use constants::AlienResult;

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
    pub fd: Option<Arc<dyn File>>,
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
        fd: Option<Arc<dyn File>>,
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

/// 一个函数调用，用于消除内存映射。
/// 注意：传入的`start`必须是某段内存映射的首地址，`len`必须是该段内存映射的长度，否则将导致函数返回`EINVAL`。函数正常执行将返回0。
#[syscall_func(215)]
pub fn do_munmap(start: usize, len: usize) -> isize {
    let task = current_task().unwrap();
    let res = task.access_inner().unmap(start, len);
    if res.is_err() {
        return res.err().unwrap();
    }
    0
}

/// 一个系统调用，用于将文件或设备映射到内存中。将一个普通文件映射到内存中，通常在需要对文件进行频繁读写时使用，这样用内存读写取代I/O读写，以获得较高的性能。
///
/// + `start`: 所要创建的映射区的起始地址。当该值为0时，内核将自动为其分配一段内存空间创建内存映射。该值在函数运行过程中将被调整为与4K对齐。
/// + `len`: 指明所要创建的映射区的长度。该值在函数运行过程中将被调整为与4K对齐。
/// + `prot`: 指明创建内存映射区的初始保护位。具体可见[`ProtFlags`]。
/// + `flags`: 指明mmap操作的相关设置。具体可见[`MapFlags`]。
/// + `fd`: 指明要创建内存映射的文件的文件描述符。
/// + `offset`: 将从文件中偏移量为`offset`处开始映射。该值需要和4K对齐。
///
/// 函数成功执行后将返回所创建的内存映射区的首地址；否则返回错误类型。
/// Reference: [do_mmap](https://man7.org/linux/man-pages/man2/mmap.2.html)
#[syscall_func(222)]
pub fn do_mmap(
    start: usize,
    len: usize,
    prot: u32,
    flags: u32,
    fd: usize,
    offset: usize,
) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let mut process_inner = process.access_inner();
    let prot = ProtFlags::from_bits_truncate(prot);
    let flags = MapFlags::from_bits_truncate(flags);
    warn!(
        "mmap: start: {:#x}, len: {:#x}, prot: {:?}, flags: {:?}, fd: {}, offset: {:#x}",
        start, len, prot, flags, fd, offset
    );
    process_inner
        .add_mmap(start, len, prot, flags, fd, offset)
        .map(|addr| addr as isize)
}

/// 一个系统调用，用于修改内存映射的保护位，从而修改对内存映射的访问权限。
/// 函数会检查传入的`start`和`len`所指示的内存映射区是否已经处于被映射状态，如果是，则将对应内存映射区的保护位与`prot`做或运算。
///
/// 如果函数正常执行，则返回0；如果`start`和`len`所指示的内存映射区未已经处于被映射状态，函数将返回-1。
#[syscall_func(226)]
pub fn map_protect(start: usize, len: usize, prot: u32) -> AlienResult<isize> {
    let process = current_task().unwrap();
    let mut process_inner = process.access_inner();
    let prot = ProtFlags::from_bits_truncate(prot);
    warn!(
        "mprotect: start: {:#x}, len: {:#x}, prot: {:?}",
        start, len, prot
    );
    process_inner.map_protect(start, len, prot)?;
    Ok(0)
}

/// (待实现)一个系统调用，用于同步文件在内存映射中的修改。一个文件通过[`do_mmap`]映射到内存中，可以在内存中对其进行快速的读写。
/// 当我们对文件的映射进行修改后，如果不调用`msync`系统调用，那么在调用[`do_munmap`]之前内存中的相应内容都不会写回磁盘文件，有可能导致不一致性问题。
/// 目前函数仅会检查所传入的`addr`是否已经被映射，如果没有被映射，则会返回`EFAULT`；否则直接返回0。
///
/// Reference: [madvise](https://man7.org/linux/man-pages/man2/madvise.2.html)
#[syscall_func(227)]
pub fn msync(addr: usize, len: usize, flags: usize) -> isize {
    warn!(
        "msync: addr: {:#x}, len: {:#x}, flags: {:#x}",
        addr, len, flags
    );
    let task = current_task().unwrap();
    let address_space = &task.access_inner().address_space;
    let res = address_space.lock().query(VirtAddr::from(addr));
    if res.is_err() {
        return LinuxErrno::EFAULT as isize;
    }
    0
}

/// (待实现)一个系统调用，用于向内核提供使用内存的建议。目前直接返回0。
///
/// Reference: [madvise](https://man7.org/linux/man-pages/man2/madvise.2.html)
#[syscall_func(233)]
pub fn madvise(addr: usize, len: usize, advice: usize) -> isize {
    warn!(
        "madvise: addr: {:#x}, len: {:#x}, advice: {:#x}",
        addr, len, advice
    );
    0
}
