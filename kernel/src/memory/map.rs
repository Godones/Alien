use bitflags::bitflags;

bitflags! {
    pub struct ProtFlags: u32 {
        const PROT_NONE = 0x0;
        const PROT_READ = 0x1;
        const PROT_WRITE = 0x2;
        const PROT_EXEC = 0x4;
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

#[syscall_func(215)]
pub fn do_munmap(start: usize, len: usize) -> isize {
    // let process = current_process().unwrap();
    // let mut inner = process.access_inner();
    // let heap_info = inner.heap_info();
    // if start == 0 {
    //     return heap_info.end as isize;
    // }
    // if start < heap_info.start {
    //     return -1;
    // }
    // if start > heap_info.end {
    //     let additional = start - heap_info.end;
    //     let res = inner.extend_heap(additional);
    //     if res.is_err() {
    //         return -1;
    //     }
    // }
    // start as isize
    0
}


/// #Reference: https://man7.org/linux/man-pages/man2/mmap.2.html
#[syscall_func(222)]
pub fn do_mmap(start: usize, len: usize, prot: u32, flags: u32, fd: usize, offset: usize) -> isize {
    0
}


