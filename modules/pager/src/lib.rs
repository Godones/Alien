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
    use crate::error::BuddyError;
    use crate::PageAllocator;

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
        let mut zone = crate::buddy::Zone::<12>::new();
        init(&mut zone);
        let mut bitmap = crate::bitmap::Bitmap::<12>::new();
        init(&mut bitmap);
    }
}