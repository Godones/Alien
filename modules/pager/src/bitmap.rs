//! Bitmap page allocator
use core::fmt::{Debug, Formatter};
use core::ops::Range;
use log::info;
use preprint::pprintln;

use crate::error::{check, PagerError};
use crate::{PageAllocator, PageAllocatorExt, PagerResult};

/// The bitmap page allocator
///
/// If the user specifies N>0, the partner allocator will manage N*8 pages, otherwise,
/// it will allocate some physical pages as bitmaps according to the memory range during initialization.
///
/// **Suggestion**:
/// 1. If you clearly know the range of physical memory, please specify the size of N.
/// 2. If you don't know the specific range of memory, specify N=0.
///
/// **Warning**:
/// 1. When specifying N=0, please ensure that the number of physical pages is greater than 1.
/// If the number of physical pages is less than 4096*8, space may be wasted
pub struct Bitmap<const N: usize> {
    /// Current number of allocated pages
    current: usize,
    /// Maximum number of pages
    max: usize,
    /// The bitmap data
    map: [u8; N],
    /// The bitmap start page
    map1: Option<usize>,
    /// start page
    start: usize,
}

impl<const N: usize> Debug for Bitmap<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("Bitmap<{N}>:\n", N = N))?;
        f.write_fmt(format_args!("  current: {}\n", self.current))?;
        f.write_fmt(format_args!("  max: {}\n", self.max))?;
        f.write_str("  data: ")?;
        if self.map1.is_none() {
            self.map.iter().for_each(|x| {
                f.write_fmt(format_args!("{:b}", x)).unwrap();
            });
        } else {
            let phys_addr = self.map1.unwrap() << 12;
            for i in 0..self.max / 8 {
                unsafe {
                    let ptr = phys_addr as *mut u8;
                    f.write_fmt(format_args!("{:b}", *ptr.add(i))).unwrap();
                }
            }
            if self.max % 8 != 0 {
                unsafe {
                    let ptr = phys_addr as *mut u8;
                    f.write_fmt(format_args!("{:b}", *ptr.add(self.max / 8)))
                        .unwrap();
                }
            }
        }
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
            map: [0; N],
            map1: None,
            start: 0,
        }
    }
    /// Get the total number of pages and the number of free pages
    pub fn page_info(&self) -> (usize, usize) {
        let free = (0..self.max).fold(
            0usize,
            |free, x| {
                if !self.test(x) {
                    free + 1
                } else {
                    free
                }
            },
        );
        (self.max, free)
    }
    #[inline]
    /// set the bit of index to 1
    fn set(&mut self, index: usize) {
        let (byte_index, bit_index) = (index / 8, index % 8);
        if self.map1.is_none() {
            self.map[byte_index] |= 1 << bit_index;
        } else {
            let phys_addr = self.map1.unwrap() << 12;
            unsafe {
                let ptr = phys_addr as *mut u8;
                *ptr.add(byte_index) |= 1 << bit_index;
            }
        }
    }
    #[inline]
    /// clear the bit of index to 0
    fn clear(&mut self, index: usize) {
        let (byte_index, bit_index) = (index / 8, index % 8);
        if self.map1.is_none() {
            self.map[byte_index] &= !(1 << bit_index);
        } else {
            let phys_addr = self.map1.unwrap() << 12;
            unsafe {
                let ptr = phys_addr as *mut u8;
                *ptr.add(byte_index) &= !(1 << bit_index);
            }
        }
    }

    /// test the bit of index
    ///
    /// if the bit is 1, return true
    #[inline]
    fn test(&self, index: usize) -> bool {
        let (byte_index, bit_index) = (index / 8, index % 8);
        if self.map1.is_none() {
            self.map[byte_index] & (1 << bit_index) != 0
        } else {
            let phys_addr = self.map1.unwrap() << 12;
            unsafe {
                let ptr = phys_addr as *mut u8;
                *ptr.add(byte_index) & (1 << bit_index) != 0
            }
        }
    }

    #[inline]
    fn alloc_pages_inner(&mut self, pages: usize) -> PagerResult<()> {
        let mut flag = false; // make sure we scan the whole bitmap once
        loop {
            let end = self.current + pages;
            if end > self.max && flag {
                return Err(PagerError::OutOfMemory(self.max));
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
    fn free_pages_inner(&mut self, page: usize, pages: usize) -> PagerResult<()> {
        let end = page + pages;
        if end > self.max {
            return Err(PagerError::PageOutOfRange);
        }
        for i in page..end {
            // check whether the page is allocated
            if !self.test(i) {
                return Err(PagerError::PageNotAllocated);
            }
            self.clear(i);
        }
        Ok(())
    }
}

impl<const N: usize> PageAllocator for Bitmap<N> {
    fn init(&mut self, memory: Range<usize>) -> PagerResult<()> {
        // check
        check(memory.clone())?;
        let mut start_page = memory.start >> 12;
        let end_page = memory.end >> 12;
        self.max = end_page - start_page;
        if N != 0 && self.max > N * 8 {
            return Err(PagerError::OutOfMemory(self.max));
        }
        // if N=0, we will allocate some physical pages as bitmaps
        if N == 0 {
            if self.max < 2 {
                return Err(PagerError::MemorySizeTooSmall);
            }
            let bitmap_pages = (self.max + 4096 * 8 - 1) / (4096 * 8);
            self.map1 = Some(start_page);
            self.max -= bitmap_pages;
            // clear bitmap
            (start_page..start_page + bitmap_pages).for_each(|x| {
                let phys_addr = x << 12;
                unsafe {
                    core::ptr::write_bytes(phys_addr as *mut u8, 0, 4096);
                }
            });
            start_page += bitmap_pages;
            pprintln!(
                "Bitmap manage {} pages using {} pages",
                self.max,
                bitmap_pages
            );
        }
        self.start = start_page;
        Ok(())
    }
    fn alloc(&mut self, order: usize) -> PagerResult<usize> {
        let need_pages = 1 << order;
        self.alloc_pages_inner(need_pages)?;
        let res = self.current - need_pages + self.start;
        // clear pages
        let page_addr = res << 12;
        unsafe {
            core::ptr::write_bytes(page_addr as *mut u8, 0, 0x1000 * need_pages);
        }
        Ok(res)
    }
    fn free(&mut self, page: usize, order: usize) -> PagerResult<()> {
        let need_pages = 1 << order;
        self.free_pages_inner(page - self.start, need_pages)?;
        Ok(())
    }
}

impl<const N: usize> PageAllocatorExt for Bitmap<N> {
    fn alloc_pages(&mut self, pages: usize, align: usize) -> PagerResult<usize> {
        assert_eq!(align, 0x1000);
        self.alloc_pages_inner(pages)?;
        let res = self.current - pages + self.start;
        // clear pages
        let page_addr = res << 12;
        unsafe {
            core::ptr::write_bytes(page_addr as *mut u8, 0, 0x1000 * pages);
        }
        Ok(res)
    }
    fn free_pages(&mut self, page: usize, pages: usize) -> PagerResult<()> {
        self.free_pages_inner(page - self.start, pages)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Bitmap;
    extern crate std;
    use crate::PageAllocator;
    use core::ops::Range;
    use std::alloc::{alloc, dealloc};
    #[test]
    fn test_not_specify_n_pages() {
        let mut bitmap = Bitmap::<0>::new();
        let memory =
            unsafe { alloc(std::alloc::Layout::from_size_align(0x1000_000, 0x1000).unwrap()) };
        let memory = memory as usize;
        let range = Range {
            start: memory,
            end: memory + 0x1000_000,
        }; // 4096 pages
           // because we not specify the number of pages, so the bitmap will use one page to store the bitmap
           // we only can use 4096 - 1 = 4095 pages
        bitmap.init(range).unwrap();
        let mut vec = vec![];
        for _ in 0..4095 {
            let page = bitmap.alloc(0).unwrap();
            vec.push(page);
        }
        let (total, free) = bitmap.page_info();
        assert_eq!(total, 4095);
        assert_eq!(free, 0);
        vec.iter().for_each(|x| {
            bitmap.free(*x, 0).unwrap();
        });
        vec.clear();
        let (total, free) = bitmap.page_info();
        assert_eq!(free, 4095);
        unsafe {
            dealloc(
                memory as *mut u8,
                std::alloc::Layout::from_size_align(0x1000000, 0x1000).unwrap(),
            )
        }
    }
}
