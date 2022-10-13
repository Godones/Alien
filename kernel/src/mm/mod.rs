mod buddy;
mod frame;
mod kmalloc;
mod slab;


use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
// pub use buddy::init_heap;
pub use frame::{alloc_frames, init_frame_allocator};
pub use kmalloc::{init_kmalloc,SlabAllocator};
pub use slab::{test_slab_system,mem_cache_init,SLAB_CACHES};
use crate::mm::slab::print_slab_system_info;


#[global_allocator]
static HEAP_ALLOCATOR:SlabAllocator = SlabAllocator::new();


pub fn test_heap() {
    let mut v = Vec::new();
    for i in 0..100 {
        v.push(i);
    }
    assert_eq!(v.len(), 100);
    let x = Box::new(5);
    assert_eq!(*x, 5);
    let str = String::from("Test heap should success！");
    println!("{}", str);
    print_slab_system_info();
}


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
