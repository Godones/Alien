#![feature(generic_const_exprs)]
#![cfg_attr(not(test), no_std)]
#![allow(incomplete_features)]
//! Buddy memory allocator


extern crate alloc;

use core::ops::Range;

#[cfg(feature = "bitmap")]
pub use crate::bitmap::Bitmap;
#[cfg(feature = "buddy")]
pub use crate::buddy::Zone;

mod buddy;
mod error;
mod bitmap;

type BuddyResult<T> = Result<T, error::BuddyError>;


pub trait PageAllocator {
    /// init the allocator according to the memory range
    fn init(&mut self, memory: Range<usize>) -> BuddyResult<()>;
    /// allocate 2^order pages
    /// # Return
    /// * `OK(usize)` - the start page
    fn alloc(&mut self, order: usize) -> BuddyResult<usize>;
    /// free 2^order pages
    /// # Params
    /// * `page` - the start page
    /// * `order` - the order of pages
    fn free(&mut self, page: usize, order: usize) -> BuddyResult<()>;
}


pub trait PageAllocatorExt {
    /// allocate pages
    /// # Params
    /// * `pages` - the number of pages, it may not be 2^order
    fn alloc_pages(&mut self, pages: usize) -> BuddyResult<usize>;
    /// free pages
    /// # Params
    /// * `page` - the start page
    /// * `pages` - the number of pages, it may not be 2^order
    fn free_pages(&mut self, page: usize, pages: usize) -> BuddyResult<()>;
}


#[cfg(test)]
mod common_test {
    use alloc::alloc::alloc;
    use alloc::boxed::Box;
    use alloc::vec;
    use core::alloc::Layout;
    use core::ops::Range;

    use crate::{bitmap, PageAllocator, PageAllocatorExt, Zone};
    use crate::error::BuddyError;

    fn init(allocator: &mut impl PageAllocator) {
        let memory = 0x1001..0x100000;
        assert_eq!(allocator.init(memory), Err(BuddyError::MemoryStartNotAligned));
        let memory = 0x0..0x0;
        assert_eq!(allocator.init(memory), Err(BuddyError::MemorySizeTooSmall));
        let memory = 0x1000..0x1001;
        assert_eq!(allocator.init(memory), Err(BuddyError::MemorySizeNotAligned));
    }

    #[test]
    fn test_init() {
        let mut zone = Zone::<12>::new();
        init(&mut zone);
        let mut bitmap = bitmap::Bitmap::<12>::new();
        init(&mut bitmap);
    }


    fn init_allocator_success(allocator: &mut impl PageAllocator, size: usize) -> *mut u8 {
        let memory = unsafe { alloc(Layout::from_size_align(size, 0x1000).unwrap()) };
        let memory = memory as usize;
        let range = Range {
            start: memory,
            end: memory + 0x1000000,
        };
        allocator.init(range).unwrap();
        memory as *mut u8
    }

    fn dealloc(ptr: *mut u8, size: usize) {
        unsafe { alloc::alloc::dealloc(ptr, Layout::from_size_align(size, 0x1000).unwrap()) }
    }


    fn alloc_dealloc<T: PageAllocator + PageAllocatorExt>(allocator: &mut T) {
        let mut vec = vec![];
        for _ in 0..4096 {
            let page = allocator.alloc(0);
            assert!(page.is_ok());
            vec.push(page.unwrap());
        }
        for i in 0..4096 {
            let page = allocator.free(vec[i], 0);
            assert!(page.is_ok());
        }
        vec.clear();


        let page_list = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512];
        page_list.iter().for_each(|&x| {
            let page = allocator.alloc_pages(x).unwrap();
            vec.push(page);
        });
        page_list.iter().for_each(|&x| {
            allocator.free_pages(vec[x], x).unwrap();
        });
        vec.clear();
    }

    #[test]
    fn test_alloc_dealloc() {
        const SIZE: usize = 0x1000000;
        let mut zone = Zone::<12>::new();
        let ptr = init_allocator_success(&mut zone, SIZE);
        dealloc(ptr, SIZE);

        let mut bitmap = bitmap::Bitmap::<{ SIZE / 0x1000 / 8 }>::new();
        let ptr = init_allocator_success(&mut bitmap, SIZE);
        dealloc(ptr, SIZE);
    }
}