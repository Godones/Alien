#![no_std]
extern crate alloc;

use arch::activate_paging_mode;
use config::FRAME_BITS;
use platform::println;
use talc::{ClaimOnOom, Span, Talc, Talck};
mod frame;

mod data;
mod vmm;

pub use data::INITRD_DATA;
pub use frame::{alloc_frame_trackers, alloc_frames, free_frames};
use ksync::Mutex;
pub use memory_addr::{PhysAddr, VirtAddr};
pub use page_table::MappingFlags;
pub use ptable::*;
pub use vmm::{
    alloc_free_region, kernel_satp, map_area_to_kernel, map_kstack_for_task, query_kernel_space,
    unmap_kstack_for_task, unmap_region_from_kernel,
};

#[global_allocator]
static HEAP_ALLOCATOR: Talck<Mutex<()>, ClaimOnOom> =
    Talc::new(unsafe { ClaimOnOom::new(Span::from_const_array(core::ptr::addr_of!(KERNEL_HEAP))) })
        .lock();
static mut KERNEL_HEAP: [u8; config::KERNEL_HEAP_SIZE] = [0; config::KERNEL_HEAP_SIZE];

extern "C" {
    fn ekernel();
}
pub fn init_memory_system(memory_end: usize, is_first_cpu: bool) {
    if is_first_cpu {
        frame::init_frame_allocator(ekernel as usize, memory_end);
        data::relocate_removable_data();
        println!("Frame allocator init success");
        println!("Talloc allocator init success");
        vmm::build_kernel_address_space(memory_end);
        println!("Build kernel address space success");
        activate_paging_mode(vmm::kernel_pgd() >> FRAME_BITS);
        println!("Activate paging mode success");
    } else {
        activate_paging_mode(vmm::kernel_pgd() >> FRAME_BITS);
    }
}
