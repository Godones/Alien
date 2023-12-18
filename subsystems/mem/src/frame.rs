use alloc::format;
use config::{FRAME_BITS, FRAME_SIZE};
use ksync::Mutex;
use page_table::PagingIf;
use pager::{PageAllocator, PageAllocatorExt};
use platform::println;
#[cfg(feature = "pager_bitmap")]
pub static FRAME_ALLOCATOR: Mutex<pager::Bitmap<0>> = Mutex::new(pager::Bitmap::new());
#[cfg(feature = "pager_buddy")]
pub static FRAME_ALLOCATOR: Mutex<pager::Zone<12>> = Mutex::new(pager::Zone::new());

pub fn init_frame_allocator(start: usize, end: usize) {
    let page_start = start / FRAME_SIZE;
    let page_end = end / FRAME_SIZE;
    let page_count = page_end - page_start;
    println!(
        "Page start:{:#x},end:{:#x},count:{:#x}",
        page_start, page_end, page_count
    );
    FRAME_ALLOCATOR
        .lock()
        .init(start..end)
        .expect("init frame allocator failed");
}

#[no_mangle]
pub fn alloc_frames(num: usize) -> *mut u8 {
    assert_eq!(num.next_power_of_two(), num);
    let start_page = FRAME_ALLOCATOR
        .lock()
        .alloc_pages(num, FRAME_SIZE)
        .expect(format!("alloc {} frame failed", num).as_str());
    let start_addr = start_page << FRAME_BITS;
    start_addr as *mut u8
}

#[no_mangle]
pub fn free_frames(addr: *mut u8, num: usize) {
    assert_eq!(num.next_power_of_two(), num);
    let start = addr as usize >> FRAME_BITS;
    FRAME_ALLOCATOR
        .lock()
        .free_pages(start, num)
        .expect(format!("free frame start:{:#x},num:{} failed", start, num).as_str());
}

pub struct VmmPageAllocator;

impl PagingIf for VmmPageAllocator {
    fn alloc_frame() -> Option<memory_addr::PhysAddr> {
        let start_addr = alloc_frames(1);
        Some(memory_addr::PhysAddr::from(start_addr as usize))
    }

    fn dealloc_frame(paddr: memory_addr::PhysAddr) {
        free_frames(paddr.as_usize() as *mut u8, 1);
    }

    fn phys_to_virt(paddr: memory_addr::PhysAddr) -> memory_addr::VirtAddr {
        memory_addr::VirtAddr::from(paddr.as_usize())
    }
}
