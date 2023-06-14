use core::alloc::GlobalAlloc;

use riscv::asm::sfence_vma_all;
use riscv::register::satp;
pub use rslab::*;

pub use frame::*;
pub use map::*;
pub use vmm::*;

use crate::arch::hart_id;
use crate::config::FRAME_SIZE;

mod frame;
mod map;
mod vmm;

#[global_allocator]
static HEAP_ALLOCATOR: HeapAllocator = HeapAllocator {
    slab: SlabAllocator,
};

pub fn init_memory_system(memory_end: usize, is_first_cpu: bool) {
    if is_first_cpu {
        init_frame_allocator(memory_end);
        println!("Frame allocator init success");
        init_slab_system(FRAME_SIZE, 32);
        println!("slab allocator init success");
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
    slab: SlabAllocator,
}

unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        if layout.size() > 8 * 1024 * 1024 {
            let frame = alloc_frames(layout.size() / FRAME_SIZE);
            frame
        } else {
            let ptr = self.slab.alloc(layout);
            ptr
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        if layout.size() > 8 * 1024 * 1024 {
            free_frames(ptr, layout.size() / FRAME_SIZE);
        } else {
            self.slab.dealloc(ptr, layout);
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
