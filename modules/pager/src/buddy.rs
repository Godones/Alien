use core::cmp::min;
use core::fmt::{Debug, Formatter};
use core::ops::Range;

use doubly_linked_list::{*};
use log::trace;

use crate::{BuddyResult, PageAllocator};
use crate::error::{BuddyError, check};

pub struct Zone<const MAX_ORDER: usize> {
    /// The pages in this zone
    manage_pages: usize,
    start_page: usize,
    free_areas: [FreeArea; MAX_ORDER],
}

#[derive(Copy, Clone)]
struct FreeArea {
    /// The number of free pages in this free area
    free_pages: usize,
    /// The list of free pages in this free area
    list_head: ListHead,
}


impl FreeArea {
    pub const fn new() -> Self {
        Self {
            free_pages: 0,
            list_head: ListHead::new(),
        }
    }
}

impl<const MAX_ORDER: usize> Debug for Zone<MAX_ORDER> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str("Zone:\n")?;
        f.write_fmt(format_args!("manage_pages: {}\nstart_page:{:#x?}\n", self.manage_pages, self.start_page))?;
        self.free_areas.iter().enumerate().for_each(|(order, free_area)| {
            let list_head = &free_area.list_head;
            if free_area.free_pages != 0 {
                f.write_fmt(format_args!("Order: {}, FreePages: {}\n", order, free_area.free_pages)).unwrap();
                list_head.iter().for_each(|l| {
                    f.write_fmt(format_args!("  {l:#x?}\n")).unwrap();
                })
            };
        });
        Ok(())
    }
}


impl<const MAX_ORDER: usize> Zone<MAX_ORDER> {
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
        trace!("--{:#x?}-{:#x?}-{:#x?}  {order}", start_page, buddy, end_page);
        if buddy <= end_page {
            let mut free_area = &mut self.free_areas[order];
            free_area.free_pages += 1;
            let addr = start_page << 12;
            let list_head = unsafe { &mut *(addr as *mut ListHead) };
            list_head_init!(*list_head);
            let ptr = to_list_head_ptr!(*list_head);
            list_add_tail!(ptr,to_list_head_ptr!(free_area.list_head));
            self.init_free_area(buddy, end_page, order);
        } else {
            self.init_free_area(start_page, end_page, order - 1);
        }
    }

    /// alloc pages from order+1
    fn alloc_inner(&mut self, order: usize) -> BuddyResult<()> {
        let order = order + 1;
        if order >= MAX_ORDER {
            return Err(BuddyError::OutOfMemory);
        }
        trace!("alloc_inner:{}, free_pages:{}", order, self.free_areas[order].free_pages);
        if self.free_areas[order].free_pages == 0 {
            self.alloc_inner(order)?;
        }
        let free_area = &mut self.free_areas[order];
        let list_head = free_area.list_head.next;
        list_del!(list_head);
        free_area.free_pages -= 1;
        let page_addr = list_head as usize;
        let buddy = page_addr + (1 << (order - 1)) * 0x1000;
        let buddy_list_head = unsafe { &mut *(buddy as *mut ListHead) };
        list_head_init!(*buddy_list_head);
        let ptr = to_list_head_ptr!(*buddy_list_head);
        list_add_tail!(list_head,to_list_head_ptr!(self.free_areas[order-1].list_head));
        list_add_tail!(ptr, to_list_head_ptr!(self.free_areas[order-1].list_head));
        self.free_areas[order - 1].free_pages += 2;
        Ok(())
    }


    fn free_inner(&mut self, page: usize, order: usize) -> BuddyResult<()> {
        let page_addr = page << 12;
        let list_head = unsafe { &mut *(page_addr as *mut ListHead) };
        list_head_init!(*list_head);
        // calculate buddy page number
        let buddy = self.start_page + ((page - self.start_page) ^ (1 << order));
        trace!("free_inner:{:#x?}, buddy:{:#x?}", page, buddy);
        let buddy_addr = buddy << 12;
        let buddy_list_head = unsafe { &mut *(buddy_addr as *mut ListHead) };
        // check buddy is free
        let is_free = self.free_areas[order].list_head.iter().any(|head| {
            head == buddy_list_head as *mut ListHead
        });
        if is_free && order != MAX_ORDER - 1 {
            // remove buddy from free area
            list_del!(to_list_head_ptr!(*buddy_list_head));
            list_head_init!(*buddy_list_head);
            self.free_areas[order].free_pages -= 1;
            // make sure the start page
            let start_page = min(page, buddy);
            self.free_inner(start_page, order + 1)?;
        } else {
            // add page to free area
            let ptr = to_list_head_ptr!(*list_head);
            list_add_tail!(ptr, to_list_head_ptr!(self.free_areas[order].list_head));
            self.free_areas[order].free_pages += 1;
        }
        Ok(())
    }
}


impl<const MAX_ORDER: usize> PageAllocator for Zone<MAX_ORDER> {
    fn init(&mut self, memory: Range<usize>) -> BuddyResult<()> {
        // init free areas
        self.free_areas.iter_mut().for_each(|area| {
            list_head_init!(area.list_head);
        });
        // check
        check(memory.clone())?;
        let start_page = memory.start >> 12;
        let end_page = memory.end >> 12;
        let manage_pages = end_page - start_page;
        self.manage_pages = manage_pages;
        self.start_page = start_page;
        trace!("page: {:#x?}-{:#x?}", start_page, end_page);
        // init free area
        self.init_free_area(start_page, end_page, MAX_ORDER - 1);
        Ok(())
    }
    fn alloc(&mut self, order: usize) -> BuddyResult<usize> {
        // check order
        if order >= MAX_ORDER {
            return Err(BuddyError::OrderTooLarge);
        }
        // check free area
        if self.free_areas[order].free_pages == 0 {
            self.alloc_inner(order)?;
            trace!("alloc success from order: {}", order + 1);
        }
        // get the first free page
        let free_area = &mut self.free_areas[order];
        let list_head = free_area.list_head.next;
        list_del!(list_head);
        trace!("{:#x?}, {:#x?}", list_head, unsafe { &*list_head });
        let page = list_head as usize >> 12;
        free_area.free_pages -= 1;
        Ok(page)
    }

    fn free(&mut self, page: usize, order: usize) -> BuddyResult<()> {
        // check order
        if order >= MAX_ORDER {
            return Err(BuddyError::OrderTooLarge);
        }
        // check page
        if page < self.start_page || page >= self.start_page + self.manage_pages {
            return Err(BuddyError::PageOutOfRange);
        }
        self.free_inner(page, order)
    }
}


#[cfg(test)]
mod buddy_test {
    use alloc::alloc::{alloc, dealloc};
    use alloc::vec;
    use core::ops::Range;

    use crate::{PageAllocator, Zone};

    #[test]
    fn test_buddy_alloc() {
        let memory = unsafe { alloc(alloc::alloc::Layout::from_size_align(0x1000000, 0x1000).unwrap()) };
        let memory = memory as usize;
        let range = Range {
            start: memory,
            end: memory + 0x1000000,
        };
        let mut zone = Zone::<12>::new();
        zone.init(range).unwrap();
        let mut vec = vec![];
        for _ in 0..4096 {
            let page = zone.alloc(0);
            assert!(page.is_ok());
            vec.push(page.unwrap());
        }
        for i in 0..4096 {
            let page = zone.free(vec[i], 0);
            assert!(page.is_ok());
        }
        vec.clear();
        unsafe { dealloc(memory as *mut u8, alloc::alloc::Layout::from_size_align(0x1000000, 0x1000).unwrap()) }
    }
}
