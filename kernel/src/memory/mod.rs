use core::alloc::GlobalAlloc;

#[cfg(feature = "buddy")]
use buddy_system_allocator::LockedHeap;
use cfg_if::cfg_if;
#[cfg(feature = "talloc")]
use talc::{Talc, Talck};

pub use frame::*;
use ksync::Mutex;
pub use map::*;
use syscall_table::syscall_func;
pub use vmm::*;

use crate::config::FRAME_SIZE;
#[cfg(any(feature = "talloc", feature = "buddy"))]
use crate::config::KERNEL_HEAP_SIZE;
use arch::{activate_paging_mode, hart_id};

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
        activate_paging_mode(kernel_space_root_ppn());
        println!("activate paging mode success");
    } else {
        activate_paging_mode(kernel_space_root_ppn());
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

unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let need_page = if layout.size() % FRAME_SIZE == 0 {
            layout.size() / FRAME_SIZE
        } else {
            layout.size() / FRAME_SIZE + 1
        };
        if layout.size() >= 5 * 1024 * 1024 {
            trace!("alloc big page: {:#x}", layout.size());
            let frame = alloc_frames(need_page);
            frame
        } else {
            let ptr = self.allocator.lock().alloc(layout);
            assert!(!ptr.is_null());
            ptr
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let need_page = if layout.size() % FRAME_SIZE == 0 {
            layout.size() / FRAME_SIZE
        } else {
            layout.size() / FRAME_SIZE + 1
        };
        if layout.size() >= 5 * 1024 * 1024 {
            trace!("free big page: {:#x}", layout.size());
            free_frames(ptr, need_page);
        } else {
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

/// (待实现)在一组线程中，设置内存屏障，控制多核系统中的内存访问次序。目前直接返回0。
///
///<https://man7.org/linux/man-pages/man2/membarrier.2.html>
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
