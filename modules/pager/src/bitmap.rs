use core::ops::Range;

use crate::{BuddyResult, PageAllocator, PageAllocatorExt};
use crate::error::{BuddyError, check};

#[derive(Debug)]
pub struct Bitmap<const N: usize> {
    /// Current number of allocated pages
    current: usize,
    /// Maximum number of pages
    max: usize,
    /// The bitmap data
    data: [u8; N],
}

impl<const N: usize> Bitmap<N> {
    /// after new, you should init
    pub const fn new() -> Self {
        Self {
            current: 0,
            max: 0,
            data: [0; N],
        }
    }
    /// set the bit of index to 1
    fn set(&mut self, index: usize) {
        let (byte_index, bit_index) = (index / 8, index % 8);
        self.data[byte_index] |= 1 << bit_index;
    }

    /// clear the bit of index to 0
    fn clear(&mut self, index: usize) {
        let (byte_index, bit_index) = (index / 8, index % 8);
        self.data[byte_index] &= !(1 << bit_index);
    }

    /// test the bit of index
    ///
    /// if the bit is 1, return true
    fn test(&self, index: usize) -> bool {
        let (byte_index, bit_index) = (index / 8, index % 8);
        self.data[byte_index] & (1 << bit_index) != 0
    }


    fn alloc_pages_inner(&mut self, pages: usize) -> BuddyResult<()> {
        let flag = false; // make sure we scan the whole bitmap once
        loop {
            let end = self.current + pages;
            if end > self.max && flag {
                return Err(BuddyError::OutOfMemory);
            }
            if end > self.max {
                self.current = 0;
                continue;
            }
            let busy_index = (self.current..end).find(|x| {
                self.test(*x)
            });

            if let Some(index) = busy_index {
                self.current = index + 1;
                continue;
            }
            // set the bitmap
            for i in self.current..end {
                self.set(i);
            }
            self.current = end;
            break;
        }
        Ok(())
    }

    fn free_pages_inner(&mut self, page: usize, size: usize) -> BuddyResult<()> {
        let end = page + size;
        if end > self.max {
            return Err(BuddyError::PageOutOfRange);
        }
        for i in page..end {
            // check whether the page is allocated
            if !self.test(i) {
                return Err(BuddyError::PageNotAllocated);
            }
            self.clear(i);
        }
        Ok(())
    }
}


impl<const N: usize> PageAllocator for Bitmap<N> {
    fn init(&mut self, memory: Range<usize>) -> BuddyResult<()> {
        // check
        check(memory.clone())?;
        let start_page = memory.start >> 12;
        let end_page = memory.end >> 12;
        self.max = end_page - start_page;
        Ok(())
    }

    fn alloc(&mut self, order: usize) -> BuddyResult<usize> {
        let need_pages = 1 << order;
        self.alloc_pages_inner(need_pages)?;
        Ok(self.current - need_pages)
    }
    fn free(&mut self, page: usize, order: usize) -> BuddyResult<()> {
        let need_pages = 1 << order;
        self.free_pages_inner(page, need_pages)?;
        Ok(())
    }
}

impl<const N: usize> PageAllocatorExt for Bitmap<N> {
    fn alloc_pages(&mut self, pages: usize) -> BuddyResult<usize> {
        self.alloc_pages_inner(pages)?;
        Ok(self.current - pages)
    }
    fn free_pages(&mut self, page: usize, pages: usize) -> BuddyResult<()> {
        self.free_pages_inner(page, pages)?;
        Ok(())
    }
}


#[cfg(test)]
mod bitmap_test {
    use alloc::alloc::{alloc, dealloc};
    use alloc::vec;
    use core::ops::Range;

    use crate::bitmap::Bitmap;
    use crate::PageAllocator;

    #[test]
    fn test_bitmap_alloc() {
        let memory = unsafe { alloc(alloc::alloc::Layout::from_size_align(0x1000000, 0x1000).unwrap()) };
        let memory = memory as usize;
        let range = Range {
            start: memory,
            end: memory + 0x1000000,
        };
        let mut bitmap = Bitmap::<4096>::new();
        bitmap.init(range).unwrap();
        let mut vec = vec![];
        for _ in 0..4096 {
            let page = bitmap.alloc(0);
            assert!(page.is_ok());
            vec.push(page.unwrap());
            assert!(bitmap.test(page.unwrap()));
        }
        for i in 0..4096 {
            let page = bitmap.free(vec[i], 0);
            assert!(page.is_ok());
            assert_eq!(bitmap.test(vec[i]), false);
        }
        vec.clear();
        unsafe { dealloc(memory as *mut u8, alloc::alloc::Layout::from_size_align(0x1000000, 0x1000).unwrap()) }
    }
}