mod frame;
pub use rslab::*;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::alloc::{GlobalAlloc, Layout};
use core::mem::forget;
use lazy_static::lazy_static;
use rbuddy::{Buddy, Locked};
// pub use buddy::init_heap;
pub use frame::{alloc_frames_t, init_frame_allocator};

use crate::mm::frame::dealloc_frames;

const KERNEL_HEAP_SIZE:usize = 1024*1024*10;
static mut HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];
#[global_allocator]
static HEAP_ALLOCATOR: SlabAllocator = SlabAllocator;


use spin::Mutex;

pub struct Process{
    id:usize
}

pub fn test_heap() {
    let mut v = Vec::<i32>::new();
    v.reserve(100);
    for i in 0..100 {
        v.push(i);
    }
    println!("vector size: {}",core::mem::size_of_val(&v));
    assert_eq!(v.capacity(), 100);
    drop(v);
    let x = Box::new(5);
    assert_eq!(*x, 5);
    let str = String::from("Test heap should success！");
    println!("{}: {}", core::mem::size_of_val(&str),str);
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



#[no_mangle]
unsafe fn alloc_frames(num: usize) -> *mut u8 {
    let frame = alloc_frames_t(num);
    let start = frame.as_ref().unwrap().start();
    forget(frame);
    start as *mut u8
}

#[no_mangle]
fn free_frames(addr: *mut u8, num: usize) {
    dealloc_frames(addr as usize, num);
}

#[no_mangle]
fn current_cpu_id() -> usize {
    0
}