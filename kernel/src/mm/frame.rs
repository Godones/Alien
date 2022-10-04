#![allow(unused)]
/// 物理页帧管理器
/// 使用位图实现
/// 位图的每一位代表一个物理页帧

use bitmap_allocator::{BitAlloc16M,BitAlloc};
use lazy_static::lazy_static;
use spin::Mutex;
use crate::config::FRAME_SIZE;

lazy_static!(
    pub static ref FRAME_ALLOCATOR: Mutex<BitAlloc16M> = Mutex::new(BitAlloc16M::default());
);
extern "C"{
    fn ekernel();
}

pub fn init_frame_allocator(){
    let start = ekernel as usize;
    let end = crate::config::MEMORY_END;
    println!("memory start:{:#x},end:{:#x}",start,end);
    // 计算页面数量
    let page_start = start / FRAME_SIZE;
    let page_end = end / FRAME_SIZE;
    let page_count = page_end - page_start;
    println!("page start:{:#x},end:{:#x},count:{:#x}",page_start,page_end,page_count);
    FRAME_ALLOCATOR.lock().insert(0..page_count);
}

pub struct Frame{
    number: usize,
    ref_count: usize,
}

unsafe fn zero_init_frame(start_addr:usize){
    core::ptr::write_bytes(start_addr as *mut u8, 0, FRAME_SIZE);
}
pub fn frame_to_addr(index:usize) -> usize{
    index * FRAME_SIZE + ekernel as usize
}
pub fn addr_to_frame(addr: usize) -> Frame{
    Frame::new((addr - ekernel as usize) / FRAME_SIZE)
}

impl Frame {
    fn new(number: usize) -> Self {
        Frame {
            number,
            ref_count: 1,
        }
    }
    fn alloc() -> Option<Frame> {
        let frame = FRAME_ALLOCATOR.lock().alloc().map_or(None,|x| {
            unsafe {
                zero_init_frame(frame_to_addr(x));
            }
            Some(Frame::new(x))
        });
        frame
    }
    fn alloc_contiguous(count: usize,align_log2:usize) -> Option<Frame> {
        let frame = FRAME_ALLOCATOR.lock().alloc_contiguous(count,align_log2).map_or(None,|x| {
            (x..x+count).into_iter().for_each(|i|{
                unsafe{zero_init_frame(frame_to_addr(i));}
            });
            Some(Frame::new(x))
        });
        frame
    }
    pub fn start(&self) -> usize {
        frame_to_addr(self.number)
    }
    pub fn end(&self) -> usize {
        self.start() + FRAME_SIZE
    }
}
impl Drop for Frame{
    fn drop(&mut self) {
        FRAME_ALLOCATOR.lock().dealloc(self.number);
    }
}
pub fn alloc_frame() -> Option<Frame>{
    Frame::alloc()
}
pub fn alloc_frames(count: usize) -> Option<Frame>{
    Frame::alloc_contiguous(count,0)
}
