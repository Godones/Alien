use core::alloc::GlobalAlloc;

#[cfg(feature = "buddy")]
use buddy_system_allocator::LockedHeap;
use cfg_if::cfg_if;
use riscv::asm::sfence_vma_all;
use riscv::register::satp;
pub use rslab::*;
#[cfg(feature = "talloc")]
use talc::{Talc, Talck};

pub use frame::*;
use kernel_sync::Mutex;
pub use map::*;
use syscall_table::syscall_func;
pub use vmm::*;

use crate::arch::hart_id;
use crate::config::FRAME_SIZE;
#[cfg(any(feature = "talloc", feature = "buddy"))]
use crate::config::KERNEL_HEAP_SIZE;

mod elf;
mod frame;
mod manager;
mod map;
mod vmm;

#[global_allocator]
static HEAP_ALLOCATOR: HeapAllocator = HeapAllocator::new();

#[cfg(any(feature = "talloc", feature = "buddy"))]
static mut KERNEL_HEAP: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

pub fn init_memory_system(memory_end: usize, is_first_cpu: bool) {
    if is_first_cpu {
        init_frame_allocator(memory_end);
        println!("Frame allocator init success");
        cfg_if! {
            if #[cfg(feature = "talloc")] {
                init_talloc_system();
                println!("talloc allocator init success");
            } else if #[cfg(feature = "buddy")] {
                init_buddy_system();
                println!("buddy allocator init success");
            } else if #[cfg(feature = "slab")] {
                init_slab_system(FRAME_SIZE, 32);
                println!("slab allocator init success");
            }
        }
        build_kernel_address_space(memory_end);
        println!("build kernel address space success");
        activate_paging_mode();
        println!("activate paging mode success");
    } else {
        activate_paging_mode();
    }
}

/// 激活页表模式
pub fn activate_paging_mode() {
    // let ppn = KERNEL_SPACE.read().root_ppn().unwrap().0;
    unsafe {
        sfence_vma_all();
        satp::set(
            satp::Mode::Sv39,
            0,
            KERNEL_SPACE.read().root_paddr().as_usize() >> 12,
        );
        sfence_vma_all();
    }
}

struct HeapAllocator {
    #[cfg(feature = "talloc")]
    allocator: Mutex<Talck>,
    #[cfg(feature = "buddy")]
    allocator: Mutex<LockedHeap<32>>,
    #[cfg(feature = "slab")]
    allocator: Mutex<SlabAllocator>,
}

struct ExtraPage {
    record: [usize; 1024],
    pages: [u16; 1024],
    have_alloc: [u16; 1024],
    //4MB
    head: usize,
}

impl ExtraPage {
    pub const fn new() -> Self {
        Self {
            record: [0; 1024],
            pages: [0; 1024],
            have_alloc: [0; 1024],
            head: 0,
        }
    }
    pub fn insert(&mut self, start_addr: usize, pages: u16) {
        if self.head < 1024 {
            self.record[self.head] = start_addr;
            self.pages[self.head] = pages;
            self.have_alloc[self.head] = 0;
            self.head += 1;
        } else {
            panic!("ExtraPage is full");
        }
    }
    fn alloc(&mut self, layout: core::alloc::Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();
        for i in 0..self.head {
            if self.record[i] != 0 && self.have_alloc[i] < FRAME_SIZE as u16 {
                let end_addr = self.record[i] + self.pages[i] as usize * FRAME_SIZE;
                let have_alloc = self.have_alloc[i] as usize;
                let start_addr = self.record[i] + have_alloc;
                // align
                let start_addr = if start_addr % align == 0 {
                    start_addr
                } else {
                    start_addr + align - start_addr % align
                };
                if start_addr + size <= end_addr {
                    self.have_alloc[i] = (start_addr - self.record[i]) as u16;
                    return start_addr as *mut u8;
                }
            } else {
                break;
            }
        }
        return core::ptr::null_mut();
    }
    pub fn find(&mut self, val: usize) -> bool {
        for i in 0..self.head {
            if self.record[i] != 0 {
                if self.record[i] <= val
                    && val < self.record[i] + self.pages[i] as usize * FRAME_SIZE
                {
                    return true;
                }
            } else {
                break;
            }
        }
        return false;
    }
}

static TRICK_ALLOC: Mutex<ExtraPage> = Mutex::new(ExtraPage::new());

unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let need_page = if layout.size() % FRAME_SIZE == 0 {
            layout.size() / FRAME_SIZE
        } else {
            layout.size() / FRAME_SIZE + 1
        };
        if layout.size() >= 5 * 1024 * 1024 {
            // assert_eq!(layout.size() % FRAME_SIZE, 0);
            trace!("alloc big page: {:#x}", layout.size());
            let frame = alloc_frames(need_page);
            frame
        } else {
            let mut ptr = self.allocator.lock().alloc(layout);
            if ptr.is_null() {
                ptr = TRICK_ALLOC.lock().alloc(layout);
                if !ptr.is_null() {
                    return ptr;
                }
                let frame = alloc_frames(need_page);
                TRICK_ALLOC.lock().insert(frame as usize, need_page as u16);
                ptr = TRICK_ALLOC.lock().alloc(layout);
                // assert!(!ptr.is_null());
                assert_eq!(ptr.is_null(), false);
            }
            ptr
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        // check TRICK_ALLOC
        let start = ptr as usize;
        let need_page = if layout.size() % FRAME_SIZE == 0 {
            layout.size() / FRAME_SIZE
        } else {
            layout.size() / FRAME_SIZE + 1
        };
        if layout.size() >= 5 * 1024 * 1024 {
            // assert_eq!(layout.size() % FRAME_SIZE, 0);
            trace!("free big page: {:#x}", layout.size());
            free_frames(ptr, need_page);
        } else {
            if TRICK_ALLOC.lock().find(start) {
                return;
            }
            assert_eq!(ptr.is_null(), false);
            self.allocator.lock().dealloc(ptr, layout);
        }
    }
}

impl HeapAllocator {
    pub const fn new() -> Self {
        Self {
            #[cfg(feature = "talloc")]
            allocator: Mutex::new(Talc::new().spin_lock()),
            #[cfg(feature = "buddy")]
            allocator: Mutex::new(LockedHeap::<32>::new()),
            #[cfg(feature = "slab")]
            allocator: Mutex::new(SlabAllocator),
        }
    }
}

pub fn kernel_satp() -> usize {
    8usize << 60 | (KERNEL_SPACE.read().root_paddr().as_usize() >> 12)
}

/// This function will be call in slab allocator
#[no_mangle]
fn current_cpu_id() -> usize {
    hart_id()
}

#[syscall_func(283)]
pub fn membarrier() -> isize {
    0
}

#[cfg(feature = "talloc")]
fn init_talloc_system() {
    unsafe {
        HEAP_ALLOCATOR
            .allocator
            .lock()
            .talc()
            .init(KERNEL_HEAP.as_mut_slice().into())
    }
}

#[cfg(feature = "buddy")]
fn init_buddy_system() {
    unsafe {
        HEAP_ALLOCATOR
            .allocator
            .lock()
            .lock()
            .init(KERNEL_HEAP.as_ptr() as usize, KERNEL_HEAP_SIZE);
        println!("heap start: {:#x}", KERNEL_HEAP.as_ptr() as usize);
    }
}
