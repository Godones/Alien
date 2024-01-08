#![no_std]

extern crate alloc;

use arch::activate_paging_mode;
use config::{FRAME_BITS, KERNEL_HEAP_SIZE};
use heap::HeapAllocator;

mod frame;
mod heap;
mod vmm;

pub use frame::{alloc_frames, free_frames};

#[global_allocator]
static HEAP_ALLOCATOR: HeapAllocator = HeapAllocator::new();

#[cfg(any(feature = "talloc", feature = "buddy"))]
static mut KERNEL_HEAP: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

pub fn init_memory_system(memory_start: usize, memory_end: usize, is_first_cpu: bool) {
    if is_first_cpu {
        frame::init_frame_allocator(memory_start, memory_end);
        println!("Frame allocator init success");
        HEAP_ALLOCATOR.init(unsafe { &mut KERNEL_HEAP });
        #[cfg(feature = "talloc")]
        {
            println!("Talloc allocator init success");
        }
        #[cfg(feature = "slab")]
        {
            println!("Slab allocator init success");
        }
        #[cfg(feature = "buddy")]
        {
            println!("Buddy allocator init success");
        }
        vmm::build_kernel_address_space(memory_end);
        println!("Build kernel address space success");
        activate_paging_mode(vmm::kernel_pgd() >> FRAME_BITS);
        println!("Activate paging mode success");
    } else {
        activate_paging_mode(vmm::kernel_pgd() >> FRAME_BITS);
    }
}
