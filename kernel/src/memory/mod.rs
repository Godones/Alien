mod frame;
mod vmm;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::alloc::GlobalAlloc;
pub use frame::{frame_allocator_test, init_frame_allocator};
use riscv::asm::sfence_vma_all;
use riscv::register::satp;
pub use rslab::*;
pub use vmm::{build_kernel_address_space, test_page_allocator, KERNEL_SPACE};

#[global_allocator]
static HEAP_ALLOCATOR: HeapAllocator = HeapAllocator {
    slab: SlabAllocator,
};

/// 激活页表模式
pub fn activate_paging_mode() {
    unsafe {
        sfence_vma_all();
        satp::set(
            satp::Mode::Sv39,
            0,
            KERNEL_SPACE.read().root_ppn().unwrap().0,
        );
        sfence_vma_all();
    }
}

struct HeapAllocator {
    slab: SlabAllocator,
}
unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        self.slab.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        self.slab.dealloc(ptr, layout)
    }
}

#[allow(unused)]
pub fn test_heap() {
    let mut v = Vec::<i32>::new();
    v.reserve(100);
    for i in 0..100 {
        v.push(i);
    }
    // println!("vector size: {}",core::mem::size_of_val(&v));
    assert_eq!(v.capacity(), 100);
    drop(v);
    let x = Box::new(5);
    assert_eq!(*x, 5);
    let _str = String::from("Test heap should success！");
    // println!("{}: {}", core::mem::size_of_val(&str),str);
    println!("heap test passed!");
}

#[allow(unused)]
pub fn test_simple_bitmap() {
    use simple_bitmap::Bitmap;
    let mut bitmap = Bitmap::<16>::new();
    assert_eq!(bitmap.alloc(), Some(0));
    bitmap.set(1);
    assert_eq!(bitmap.alloc(), Some(2));
    let x = bitmap.alloc_contiguous(3, 0);
    assert_eq!(x, Some(3));
    let x = bitmap.alloc_contiguous(3, 0);
    assert_eq!(x, Some(6));
    bitmap.dealloc(7);
    let x = bitmap.alloc_contiguous(3, 0);
    assert_eq!(x, Some(9));
    info!("bitmap test passed");
}

#[no_mangle]
fn current_cpu_id() -> usize {
    0
}
