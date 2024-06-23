use alloc::{
    alloc::{alloc, dealloc},
    boxed::Box,
    collections::BTreeMap,
    vec,
};
use core::{alloc::Layout, any::TypeId};

use ksync::Mutex;
use rref::{SharedHeapAlloc, SharedHeapAllocation};
use spin::Lazy;

static SHARED_HEAP: Lazy<Mutex<BTreeMap<usize, SharedHeapAllocation>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

pub struct SharedHeapAllocator;
impl SharedHeapAlloc for SharedHeapAllocator {
    unsafe fn alloc(
        &self,
        layout: Layout,
        type_id: TypeId,
        drop_fn: fn(TypeId, *mut u8),
    ) -> Option<SharedHeapAllocation> {
        let ptr = alloc(layout);
        if ptr.is_null() {
            return None;
        }
        log::trace!(
            "<SharedHeap> alloc size: {}, ptr: {:#x}",
            layout.size(),
            ptr as usize
        );
        let domain_id_pointer = Box::into_raw(Box::new(0));
        let borrow_count_pointer = Box::into_raw(Box::new(0));
        let res = SharedHeapAllocation {
            value_pointer: ptr,
            domain_id_pointer,
            borrow_count_pointer,
            layout,
            type_id,
            drop_fn,
        };
        SHARED_HEAP.lock().insert(ptr as usize, res.clone());
        Some(res)
    }

    unsafe fn dealloc(&self, ptr: *mut u8) {
        let mut heap = SHARED_HEAP.lock();
        let allocation = heap.remove(&(ptr as usize));
        if let Some(allocation) = allocation {
            log::trace!("<SharedHeap> dealloc: {:p}", ptr);
            assert_eq!(allocation.value_pointer, ptr);
            dealloc(allocation.value_pointer, allocation.layout);
            dealloc(
                allocation.domain_id_pointer as *mut u8,
                Layout::new::<u64>(),
            );
            dealloc(
                allocation.borrow_count_pointer as *mut u8,
                Layout::new::<u64>(),
            );
        } else {
            panic!(
                "<SharedHeap> dealloc: {:#x}, but the data has been dropped",
                ptr as usize
            );
        }
    }
}

pub enum FreeShared {
    Free,
    NotFree(u64),
}

pub fn free_domain_shared_data(id: u64, free_shared: FreeShared) {
    let mut data = vec![];
    let heap = SHARED_HEAP.lock();
    heap.iter().for_each(|(_, v)| {
        if v.domain_id() == id {
            data.push(*v);
        }
    });
    drop(heap);
    println_color!(34, "<free_domain_shared_data> for domain_id: {}", id);
    println_color!(34, "domain has {} data", data.len());

    match free_shared {
        FreeShared::Free => {
            println_color!(34, "free_shared is Free, free data");
            data.into_iter().for_each(|v| unsafe {
                v.drop_fn();
                SharedHeapAllocator.dealloc(v.value_pointer);
            });
        }
        FreeShared::NotFree(domain_id) => {
            println_color!(34, "free_shared is NotFree, do not free data");
            data.into_iter()
                .for_each(|v| unsafe { v.set_domain_id(domain_id) });
        }
    }
}
