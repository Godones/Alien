#![no_std]
#![feature(allocator_api)]
#![feature(slice_ptr_get)]
extern crate alloc;
#[macro_use]
extern crate platform;
use arch::activate_paging_mode;
use config::FRAME_BITS;
pub mod data;
mod frame;
#[cfg(feature = "buddy")]
mod heap;
mod manager;
#[cfg(feature = "talloc")]
mod talc_wrapper;
mod vmm;

pub use frame::{alloc_frame_trackers, alloc_frames, free_frames, FrameTracker, VmmPageAllocator};
pub use manager::FRAME_REF_MANAGER;
use platform::config::HEAP_SIZE;
pub use vmm::{kernel_pgd, kernel_satp, kernel_space, map_region_to_kernel, query_kernel_space};

#[cfg(feature = "buddy")]
#[global_allocator]
static HEAP_ALLOCATOR: heap::HeapAllocator = heap::HeapAllocator::new();

#[cfg(feature = "talc")]
#[global_allocator]
static HEAP_ALLOCATOR: talc_wrapper::TalcAllocator = talc_wrapper::TalcAllocator;

extern "C" {
    fn sheap();
}

pub fn init_memory_system(memory_end: usize, is_first_cpu: bool) {
    if is_first_cpu {
        frame::init_frame_allocator(sheap as usize + HEAP_SIZE, memory_end);
        println!("Frame allocator init success");
        #[cfg(feature = "initrd")]
        data::relocate_removable_data();
        #[cfg(feature = "buddy")]
        HEAP_ALLOCATOR.init(sheap as usize, HEAP_SIZE);
        #[cfg(feature = "talloc")]
        println!("Talloc allocator init success");
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
