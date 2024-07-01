use alloc::{
    alloc::{alloc, dealloc},
    boxed::Box,
    collections::BTreeMap,
    vec,
};
use core::{alloc::Layout, any::TypeId};

use ksync::Mutex;
use rref::{SharedHeapAlloc, SharedHeapAllocation};

static SHARED_HEAP: Mutex<BTreeMap<usize, SharedHeapAllocation>> = Mutex::new(BTreeMap::new());

pub static SHARED_HEAP_ALLOCATOR: &'static dyn SharedHeapAlloc = &SharedHeapAllocator;

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
        log::error!(
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
            log::error!("<SharedHeap> dealloc: {:p}", ptr);
            assert_eq!(allocation.value_pointer, ptr);
            dealloc(allocation.value_pointer, allocation.layout);
            let _ = Box::from_raw(allocation.domain_id_pointer);
            let _ = Box::from_raw(allocation.borrow_count_pointer);
        } else {
            panic!(
                "<SharedHeap> dealloc: {:#x}, but the data has been dropped",
                ptr as usize
            );
        }
    }
}

pub fn checkout_shared_data() {
    let heap = SHARED_HEAP.lock();
    let mut map = BTreeMap::new();
    heap.iter().for_each(|(_, v)| {
        let id = v.domain_id();
        let count = map.get(&id).unwrap_or(&0) + 1;
        map.insert(id, count);
    });
    for (id, count) in map {
        println_color!(34, "domain_id: {}, count: {}", id, count);
    }
    println_color!(
        34,
        "<checkout_shared_data> shared heap size: {}",
        heap.len()
    );
}

pub enum FreeShared {
    Free,
    NotFree(u64),
}

pub fn free_domain_shared_data(id: u64, free_shared: FreeShared) {
    checkout_shared_data();
    let mut data = vec![];
    let heap = SHARED_HEAP.lock();
    println_color!(
        34,
        "<free_domain_shared_data> shared heap size: {}",
        heap.len()
    );
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
