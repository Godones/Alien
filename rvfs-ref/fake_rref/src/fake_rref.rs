use alloc::{
    alloc::{alloc, dealloc},
    collections::BTreeMap,
};
use core::{alloc::Layout, any::TypeId};

use shared_heap::{SharedHeapAlloc, SharedHeapAllocation};
use spin::{Lazy, Mutex};

static SHARED_HEAP: Lazy<Mutex<BTreeMap<usize, SharedHeapAllocation>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

pub struct MYSharedHeapAllocator;
impl SharedHeapAlloc for MYSharedHeapAllocator {
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
        log::info!(
            "[SharedHeap] alloc size: {}, ptr: {:#x}",
            layout.size(),
            ptr as usize
        );
        let domain_id_pointer = alloc(Layout::new::<u64>()) as *mut u64;
        let res = SharedHeapAllocation {
            value_pointer: ptr,
            domain_id_pointer,
            layout,
            type_id,
            drop_fn,
        };
        SHARED_HEAP.lock().insert(ptr as usize, res.clone());
        Some(res)
    }

    unsafe fn dealloc(&self, ptr: *mut u8) {
        let allocation = SHARED_HEAP.lock().remove(&(ptr as usize));
        if let Some(allocation) = allocation {
            log::info!(
                "[SharedHeap] dealloc: {:p}, size:{}",
                ptr,
                allocation.layout.size()
            );
            assert_eq!(allocation.value_pointer, ptr);
            dealloc(allocation.value_pointer, allocation.layout);
            dealloc(
                allocation.domain_id_pointer as *mut u8,
                Layout::new::<u64>(),
            );
        } else {
            panic!("The data has been dropped");
        }
    }
}

pub fn fake_init_rref() {
    shared_heap::init(&MYSharedHeapAllocator, 0);
}
