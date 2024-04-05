#![no_std]
extern crate alloc;

use arch::activate_paging_mode;
use config::FRAME_BITS;
use heap::HeapAllocator;
use platform::println;
mod frame;
mod heap;
mod vmm;

pub use frame::{alloc_frames, free_frames};
pub use memory_addr::VirtAddr;
pub use page_table::MappingFlags;
pub use ptable::*;
pub use vmm::{
    alloc_free_region, is_in_kernel_space, kernel_satp, map_region_to_kernel, query_kernel_space,
    unmap_region_from_kernel,
};

#[global_allocator]
static HEAP_ALLOCATOR: HeapAllocator = HeapAllocator::new();

#[cfg(any(feature = "talloc", feature = "buddy"))]
static mut KERNEL_HEAP: [u8; config::KERNEL_HEAP_SIZE] = [0; config::KERNEL_HEAP_SIZE];

extern "C" {
    fn ekernel();
}
pub fn init_memory_system(memory_end: usize, is_first_cpu: bool) {
    if is_first_cpu {
        frame::init_frame_allocator(ekernel as usize, memory_end);
        println!("Frame allocator init success");
        // #[allow(static_mut_refs)]
        HEAP_ALLOCATOR.init(unsafe { &mut KERNEL_HEAP });
        #[cfg(feature = "talloc")]
        println!("Talloc allocator init success");
        #[cfg(feature = "slab")]
        println!("Slab allocator init success");
        #[cfg(feature = "buddy")]
        println!("Buddy allocator init success");
        vmm::build_kernel_address_space(memory_end);
        println!("Build kernel address space success");
        activate_paging_mode(vmm::kernel_pgd() >> FRAME_BITS);
        println!("Activate paging mode success");
    } else {
        activate_paging_mode(vmm::kernel_pgd() >> FRAME_BITS);
    }
}
