mod buddy;
mod frame;
mod slab;


pub use buddy::init_heap;
pub use frame::init_frame_allocator;



pub fn test_simple_bitmap(){
    use simple_bitmap::Bitmap;
    let mut bitmap = Bitmap::<16>::new();
    assert_eq!(bitmap.alloc(),Some(0));
    bitmap.set(1);
    assert_eq!(bitmap.alloc(),Some(2));
    let x = bitmap.alloc_contiguous(3,0);
    assert_eq!(x,Some(3));
    let x = bitmap.alloc_contiguous(3,0);
    assert_eq!(x,Some(6));
    bitmap.dealloc(7);
    let x = bitmap.alloc_contiguous(3,0);
    assert_eq!(x,Some(9));
    info!("bitmap test passed");
}