use crate::config::KERNEL_HEAP_SIZE;
use buddy_system_allocator::LockedHeap;



#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::new();


/// 初始化堆区
pub fn init_heap(){
    static HEAP_DATA: [u8;KERNEL_HEAP_SIZE] = [0;KERNEL_HEAP_SIZE];
    unsafe {
        HEAP_ALLOCATOR.lock().init(
            HEAP_DATA.as_ptr() as usize,
            KERNEL_HEAP_SIZE
        );
    }
}