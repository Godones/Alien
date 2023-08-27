//! Buddy system allocator
//!
//! The buddy system is a memory allocation and management algorithm that manages memory in power of
//! two increments. It operates by splitting memory into halves until each chunk is the size of the
//! allocation request.
//!
//! The buddy system is often used to allocate memory in a computer system for the execution of a
//! program by an application programmer (or a compiler or assembler). It is also used by kernels
//! for their internal data structures, and sometimes for user-space memory allocation.
//!
//! # References
//! <https://en.wikipedia.org/wiki/Buddy_memory_allocation>
use core::cmp::min;
use core::fmt::{Debug, Formatter};
use core::ops::Range;

use doubly_linked_list::*;
use log::trace;
use preprint::pprintln;

use crate::error::{check, PagerError};
use crate::{PageAllocator, PageAllocatorExt, PagerResult};

/// The buddy system allocator
pub struct Zone<const MAX_ORDER: usize> {
    /// The pages in this zone
    manage_pages: usize,
    start_page: usize,
    free_areas: [FreeArea; MAX_ORDER],
}

#[derive(Copy, Clone)]
struct FreeArea {
    /// The number of free pages in this free area
    free: usize,
    /// The list of free pages in this free area
    list_head: ListHead,
}

impl FreeArea {
    pub const fn new() -> Self {
        Self {
            free: 0,
            list_head: ListHead::new(),
        }
    }
}

impl<const MAX_ORDER: usize> Debug for Zone<MAX_ORDER> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("Zone<{MAX_ORDER}>:\n", MAX_ORDER = MAX_ORDER))?;
        f.write_fmt(format_args!(
            "  manage_pages: {}\n  start_page:{:#x?}\n",
            self.manage_pages, self.start_page
        ))?;
        self.free_areas
            .iter()
            .enumerate()
            .for_each(|(order, free_area)| {
                let list_head = &free_area.list_head;
                if free_area.free != 0 {
                    f.write_fmt(format_args!(
                        "  Order: {}, Free: {}\n",
                        order, free_area.free
                    ))
                    .unwrap();
                    list_head.iter().for_each(|l| {
                        f.write_fmt(format_args!("      {l:#x?}\n")).unwrap();
                    })
                };
            });
        Ok(())
    }
}

impl<const MAX_ORDER: usize> Zone<MAX_ORDER> {
    /// after new, you should init
    pub const fn new() -> Self {
        Self {
            manage_pages: 0,
            start_page: 0,
            free_areas: [FreeArea::new(); MAX_ORDER],
        }
    }

    fn init_free_area(&mut self, start_page: usize, end_page: usize, order: usize) {
        if start_page >= end_page {
            return;
        }
        let buddy = start_page + (1 << order);
        trace!(
            "--{:#x?}-{:#x?}-{:#x?}  {order}",
            start_page,
            buddy,
            end_page
        );
        if buddy <= end_page {
            let free_area = &mut self.free_areas[order];
            free_area.free += 1;
            let addr = start_page << 12;
            let list_head = unsafe { &mut *(addr as *mut ListHead) };
            list_head_init!(*list_head);
            let ptr = to_list_head_ptr!(*list_head);
            list_add_tail!(ptr, to_list_head_ptr!(free_area.list_head));
            self.init_free_area(buddy, end_page, order);
        } else {
            self.init_free_area(start_page, end_page, order - 1);
        }
    }

    /// alloc pages from order+1
    fn alloc_inner(&mut self, order: usize) -> PagerResult<()> {
        let order = order + 1;
        if order >= MAX_ORDER {
            return Err(PagerError::OutOfMemory(1 << MAX_ORDER));
        }
        trace!(
            "alloc_inner:{}, free_pages:{}",
            order,
            self.free_areas[order].free
        );
        if self.free_areas[order].free == 0 {
            self.alloc_inner(order)?;
        }
        let free_area = &mut self.free_areas[order];
        let list_head = free_area.list_head.next;
        list_del!(list_head);
        free_area.free -= 1;
        let page_addr = list_head as usize;
        let buddy = page_addr + (1 << (order - 1)) * 0x1000;
        let buddy_list_head = unsafe { &mut *(buddy as *mut ListHead) };
        list_head_init!(*buddy_list_head);
        let ptr = to_list_head_ptr!(*buddy_list_head);
        list_add_tail!(
            list_head,
            to_list_head_ptr!(self.free_areas[order - 1].list_head)
        );
        list_add_tail!(ptr, to_list_head_ptr!(self.free_areas[order - 1].list_head));
        self.free_areas[order - 1].free += 2;
        Ok(())
    }

    fn free_inner(&mut self, page: usize, order: usize) -> PagerResult<()> {
        let page_addr = page << 12;
        let list_head = unsafe { &mut *(page_addr as *mut ListHead) };
        list_head_init!(*list_head);
        // calculate buddy page number
        let buddy = self.start_page + ((page - self.start_page) ^ (1 << order));
        trace!("free_inner:{:#x?}, buddy:{:#x?}", page, buddy);
        let buddy_addr = buddy << 12;
        let buddy_list_head = unsafe { &mut *(buddy_addr as *mut ListHead) };
        // check buddy is free
        let is_free = self.free_areas[order]
            .list_head
            .iter()
            .any(|head| head == buddy_list_head as *mut ListHead);
        if is_free && order != MAX_ORDER - 1 {
            // remove buddy from free area
            list_del!(to_list_head_ptr!(*buddy_list_head));
            list_head_init!(*buddy_list_head);
            self.free_areas[order].free -= 1;
            // make sure the start page
            let start_page = min(page, buddy);
            self.free_inner(start_page, order + 1)?;
        } else {
            // add page to free area
            let ptr = to_list_head_ptr!(*list_head);
            list_add_tail!(ptr, to_list_head_ptr!(self.free_areas[order].list_head));
            self.free_areas[order].free += 1;
        }
        Ok(())
    }
}

impl<const MAX_ORDER: usize> PageAllocator for Zone<MAX_ORDER> {
    fn init(&mut self, memory: Range<usize>) -> PagerResult<()> {
        // check
        check(memory.clone())?;
        // init free areas
        self.free_areas.iter_mut().for_each(|area| {
            list_head_init!(area.list_head);
        });
        let start_page = memory.start >> 12;
        let end_page = memory.end >> 12;
        let manage_pages = end_page - start_page;
        self.manage_pages = manage_pages;
        self.start_page = start_page;
        trace!("page: {:#x?}-{:#x?}", start_page, end_page);
        // init free area
        self.init_free_area(start_page, end_page, MAX_ORDER - 1);
        pprintln!("Zone<{}> manages {} pages", MAX_ORDER, manage_pages);
        Ok(())
    }
    fn alloc(&mut self, order: usize) -> PagerResult<usize> {
        // check order
        if order >= MAX_ORDER {
            return Err(PagerError::OrderTooLarge);
        }
        // check free area
        if self.free_areas[order].free == 0 {
            self.alloc_inner(order)?;
            trace!("alloc success from order: {}", order + 1);
        }
        // get the first free page
        let free_area = &mut self.free_areas[order];
        let list_head = free_area.list_head.next;
        trace!("{:#x?}, {:#x?}", list_head, unsafe { &*list_head });
        list_del!(list_head);
        let page = list_head as usize >> 12;
        free_area.free -= 1;
        let page_addr = page << 12;
        unsafe {
            core::ptr::write_bytes(page_addr as *mut u8, 0, 0x1000 * (1 << order));
        }
        Ok(page)
    }

    fn free(&mut self, page: usize, order: usize) -> PagerResult<()> {
        // check order
        if order >= MAX_ORDER {
            return Err(PagerError::OrderTooLarge);
        }
        // check page
        if page < self.start_page || page >= self.start_page + self.manage_pages {
            return Err(PagerError::PageOutOfRange);
        }
        self.free_inner(page, order)
    }
}

impl<const MAX_ORDER: usize> PageAllocatorExt for Zone<MAX_ORDER> {
    fn alloc_pages(&mut self, pages: usize, align: usize) -> PagerResult<usize> {
        assert_eq!(align, 0x1000);
        let order = pages.next_power_of_two().trailing_zeros() as usize;
        self.alloc(order)
    }

    fn free_pages(&mut self, page: usize, pages: usize) -> PagerResult<()> {
        let order = pages.next_power_of_two().trailing_zeros() as usize;
        self.free(page, order)
    }
}

#[cfg(test)]
mod tests {
    use crate::Zone;
    extern crate std;
    use crate::PageAllocator;
    use core::ops::Range;
    use std::alloc::{alloc, dealloc};
    fn init() -> Zone<12> {
        let mut zone = Zone::<12>::new();
        let memory =
            unsafe { alloc(std::alloc::Layout::from_size_align(0x1000_000, 0x1000).unwrap()) };
        let memory = memory as usize;
        let range = Range {
            start: memory,
            end: memory + 0x1000_000,
        };
        println!("{range:#x?}");
        zone.init(range).unwrap();
        zone
    }
    #[test]
    fn test_alloc_pages() {
        assert_eq!(100usize.next_power_of_two().trailing_zeros(), 7);
        assert_eq!(1usize.next_power_of_two().trailing_zeros(), 0);
    }
}
