use alloc::alloc::{alloc, dealloc};
use alloc::collections::BTreeMap;
use core::alloc::Layout;
use ksync::Mutex;
use log::trace;
use rref::{SharedHeap, SharedHeapAllocation};
use spin::Lazy;

static SHARED_HEAP: Lazy<Mutex<BTreeMap<usize, SharedHeapAllocation>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

pub struct SharedHeapAllocator;
impl SharedHeap for SharedHeapAllocator {
    unsafe fn alloc(&self, layout: Layout, type_id: u64) -> Option<SharedHeapAllocation> {
        trace!("[SharedHeap] alloc: {:?}, type_id: {}", layout, type_id);
        let ptr = alloc(layout);
        if ptr.is_null() {
            return None;
        }
        let domain_id_pointer = alloc(Layout::new::<u64>()) as *mut u64;
        let borrow_count_pointer = alloc(Layout::new::<u64>()) as *mut u64;
        let res = SharedHeapAllocation {
            value_pointer: ptr,
            domain_id_pointer,
            borrow_count_pointer,
            layout,
            type_id,
        };
        SHARED_HEAP.lock().insert(ptr as usize, res.clone());
        Some(res)
    }

    unsafe fn dealloc(&self, ptr: *mut u8) {
        trace!("[SharedHeap] dealloc: {:p}", ptr);
        let mut heap = SHARED_HEAP.lock();
        let allocation = heap.remove(&(ptr as usize)).unwrap();
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
    }
}
