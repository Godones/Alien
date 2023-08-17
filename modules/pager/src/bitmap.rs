//! Bitmap page allocator
use core::fmt::{Debug, Formatter};
use core::ops::Range;

use crate::error::{check, BuddyError};
use crate::{BuddyResult, PageAllocator, PageAllocatorExt};

/// The bitmap page allocator
///
/// It can manage N*8 pages
pub struct Bitmap<const N: usize> {
    /// Current number of allocated pages
    current: usize,
    /// Maximum number of pages
    max: usize,
    /// The bitmap data
    data: [u8; N],
    /// start page
    start: usize,
}

impl<const N: usize> Debug for Bitmap<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("Bitmap<{N}>:\n", N = N))?;
        f.write_fmt(format_args!("  current: {}\n", self.current))?;
        f.write_fmt(format_args!("  max: {}\n", self.max))?;
        // print bitmap
        f.write_str("  data: ")?;
        self.data.iter().for_each(|x| {
            f.write_fmt(format_args!("{:b}", x)).unwrap();
        });
        f.write_str("\n")
    }
}

impl<const N: usize> Bitmap<N> {
    /// after new, you should init
    #[allow(unused)]
    pub const fn new() -> Self {
        Self {
            current: 0,
            max: 0,
            data: [0; N],
            start: 0,
        }
    }
    #[inline]
    /// set the bit of index to 1
    fn set(&mut self, index: usize) {
        let (byte_index, bit_index) = (index / 8, index % 8);
        self.data[byte_index] |= 1 << bit_index;
    }
    #[inline]
    /// clear the bit of index to 0
    fn clear(&mut self, index: usize) {
        let (byte_index, bit_index) = (index / 8, index % 8);
        self.data[byte_index] &= !(1 << bit_index);
    }

    /// test the bit of index
    ///
    /// if the bit is 1, return true
    #[inline]
    fn test(&self, index: usize) -> bool {
        let (byte_index, bit_index) = (index / 8, index % 8);
        self.data[byte_index] & (1 << bit_index) != 0
    }

    #[inline]
    fn alloc_pages_inner(&mut self, pages: usize) -> BuddyResult<()> {
        let mut flag = false; // make sure we scan the whole bitmap once
        loop {
            let end = self.current + pages;
            if end > self.max && flag {
                return Err(BuddyError::OutOfMemory(self.max));
            }
            if end > self.max {
                self.current = 0;
                flag = true;
                continue;
            }
            let busy_index = (self.current..end).find(|x| self.test(*x));

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

    #[inline]
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
        if self.max > N * 8 {
            return Err(BuddyError::OutOfMemory(self.max));
        }
        self.start = start_page;
        Ok(())
    }
    fn alloc(&mut self, order: usize) -> BuddyResult<usize> {
        let need_pages = 1 << order;
        self.alloc_pages_inner(need_pages)?;

        let res = self.current - need_pages + self.start;
        // init page
        let page_addr = res << 12;
        unsafe {
            core::ptr::write_bytes(page_addr as *mut u8, 0, 4096 * need_pages);
        }
        Ok(res)
    }
    fn free(&mut self, page: usize, order: usize) -> BuddyResult<()> {
        let need_pages = 1 << order;
        self.free_pages_inner(page - self.start, need_pages)?;
        Ok(())
    }
}

impl<const N: usize> PageAllocatorExt for Bitmap<N> {
    fn alloc_pages(&mut self, pages: usize, align: usize) -> BuddyResult<usize> {
        assert_eq!(align, 0x1000);
        self.alloc_pages_inner(pages)?;
        let res = self.current - pages + self.start;
        // init page
        let page_addr = res << 12;
        unsafe {
            core::ptr::write_bytes(page_addr as *mut u8, 0, 4096 * pages);
        }
        Ok(res)
    }
    fn free_pages(&mut self, page: usize, pages: usize) -> BuddyResult<()> {
        self.free_pages_inner(page - self.start, pages)?;
        Ok(())
    }
}
