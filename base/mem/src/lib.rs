#![feature(slice_ptr_get)]
#![feature(allocator_api)]
#![no_std]
extern crate alloc;
use core::{
    alloc::{Allocator, GlobalAlloc, Layout},
    ptr::NonNull,
};

use arch::activate_paging_mode;
use config::{FRAME_BITS, KERNEL_HEAP_SIZE};
use platform::println;
use talc::{ErrOnOom, Talc, Talck};
mod frame;

mod data;
mod vmm;

pub use data::INITRD_DATA;
pub use frame::{alloc_frame_trackers, alloc_frames, free_frames, FrameTracker};
use ksync::Mutex;
pub use memory_addr::{PhysAddr, VirtAddr};
pub use page_table::MappingFlags;
use pconst::LinuxErrno;
pub use ptable::*;
use spin::Lazy;
pub use vmm::{
    kernel_satp, map_kstack_for_task, query_kernel_space,
    unmap_kstack_for_task, map_domain_region, unmap_domain_area, VirtDomainArea, set_memory_x
};

type AlienError = LinuxErrno;
type AlienResult<T> = Result<T, AlienError>;

extern "C" {
    fn sheap();
}
pub fn init_memory_system(memory_end: usize, is_first_cpu: bool) {
    if is_first_cpu {
        frame::init_frame_allocator(sheap as usize + KERNEL_HEAP_SIZE, memory_end);
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

#[global_allocator]
static GLOBAL_HEAP_ALLOCATOR: TalcAllocator = TalcAllocator;

static HEAP_ALLOCATOR: Lazy<MyAllocator> = Lazy::new(|| MyAllocator::new());

pub struct TalcAllocator;

unsafe impl GlobalAlloc for TalcAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        HEAP_ALLOCATOR.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        HEAP_ALLOCATOR.dealloc(ptr, layout);
    }
}

pub struct MyAllocator(Talck<Mutex<()>, ErrOnOom>);

impl MyAllocator {
    fn new() -> Self {
        let talck = Talc::new(ErrOnOom).lock::<Mutex<()>>();
        unsafe {
            let heap = core::slice::from_raw_parts_mut(sheap as usize as *mut u8, KERNEL_HEAP_SIZE);
            let _res = talck.lock().claim(heap.as_mut().into()).unwrap();
        }

        Self(talck)
    }
}

unsafe impl GlobalAlloc for MyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.0.allocate(layout).unwrap().as_mut_ptr();
        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.deallocate(NonNull::new(ptr).unwrap(), layout);
    }
}
