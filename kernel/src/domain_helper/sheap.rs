use alloc::{
    alloc::{alloc, dealloc},
    collections::BTreeMap,
    sync::Arc,
    vec,
    vec::Vec,
};
use core::{alloc::Layout, any::TypeId};

use config::FRAME_SIZE;
use hashbrown::HashMap;
use ksync::Mutex;
use rref::{SharedHeapAlloc, SharedHeapAllocation};
use spin::Lazy;

static SHARED_HEAP: Mutex<BTreeMap<usize, SharedHeapAllocation>> = Mutex::new(BTreeMap::new());
pub static SHARED_HEAP_ALLOCATOR: &'static dyn SharedHeapAlloc = &SharedHeapAllocator;

struct SharedHeapAllocationPart {
    value_pointer: *mut u8,
    domain_id_pointer: *mut u64,
}
unsafe impl Send for SharedHeapAllocationPart {}

pub struct SharedHeapCache {
    cache: Mutex<HashMap<Layout, Arc<Mutex<Vec<SharedHeapAllocationPart>>>>>,
}

impl SharedHeapCache {
    fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
        }
    }
    fn get(&self, layout: &Layout) -> Option<SharedHeapAllocationPart> {
        let vec = self.cache.lock().get(layout).map(|x| x.clone());
        if let Some(vec) = vec {
            let mut vec = vec.lock();
            if let Some(part) = vec.pop() {
                return Some(part);
            }
        }
        None
    }
    fn insert(&self, layout: Layout, part: SharedHeapAllocationPart) {
        let mut cache = self.cache.lock();
        let vec = cache
            .entry(layout)
            .or_insert_with(|| Arc::new(Mutex::new(Vec::with_capacity(16))))
            .clone();
        drop(cache);
        let mut vec = vec.lock();
        vec.push(part);
    }
}

static SHARED_HEAP_CACHE: Lazy<SharedHeapCache> = Lazy::new(|| SharedHeapCache::new());
pub struct SharedHeapAllocator;

impl SharedHeapAllocator {
    fn alloc_from_cache(
        layout: &Layout,
        type_id: TypeId,
        drop_fn: fn(TypeId, *mut u8),
    ) -> Option<(*mut u8, SharedHeapAllocation)> {
        let part = SHARED_HEAP_CACHE.get(&layout);
        if let Some(part) = part {
            let ptr = part.value_pointer;
            let domain_id_pointer = part.domain_id_pointer;
            let res = SharedHeapAllocation {
                value_pointer: ptr,
                domain_id_pointer,
                layout: *layout,
                type_id,
                drop_fn,
            };
            return Some((ptr, res));
        };
        None
    }

    unsafe fn alloc_from_heap(
        layout: Layout,
        type_id: TypeId,
        drop_fn: fn(TypeId, *mut u8),
    ) -> Option<(*mut u8, SharedHeapAllocation)> {
        let ptr = alloc(layout);
        if ptr.is_null() {
            panic!("<SharedHeap> alloc layout: {:?} failed", layout);
        }
        log::error!(
            "<SharedHeap> alloc size: {}, ptr: {:#x}",
            layout.size(),
            ptr as usize
        );
        let domain_id_pointer = alloc(Layout::for_value(&0u64)) as *mut u64;
        let res = SharedHeapAllocation {
            value_pointer: ptr,
            domain_id_pointer,
            layout,
            type_id,
            drop_fn,
        };
        Some((ptr, res))
    }
}

impl SharedHeapAlloc for SharedHeapAllocator {
    unsafe fn alloc(
        &self,
        layout: Layout,
        type_id: TypeId,
        drop_fn: fn(TypeId, *mut u8),
    ) -> Option<SharedHeapAllocation> {
        if layout.size() > FRAME_SIZE {
            let (ptr, res) = SharedHeapAllocator::alloc_from_heap(layout, type_id, drop_fn)?;
            let mut shared_heap = SHARED_HEAP.lock();
            shared_heap.insert(ptr as usize, res.clone());
            return Some(res);
        }
        let res = SharedHeapAllocator::alloc_from_cache(&layout, type_id, drop_fn);
        let (ptr, res) = if let Some((ptr, res)) = res {
            (ptr, res)
        } else {
            SharedHeapAllocator::alloc_from_heap(layout, type_id, drop_fn)?
        };
        let mut shared_heap = SHARED_HEAP.lock();
        shared_heap.insert(ptr as usize, res.clone());
        Some(res)
    }

    unsafe fn dealloc(&self, ptr: *mut u8) {
        let mut heap = SHARED_HEAP.lock();
        let allocation = heap.remove(&(ptr as usize));
        drop(heap);
        if let Some(allocation) = allocation {
            log::error!("<SharedHeap> dealloc: {:p}", ptr);
            assert_eq!(allocation.value_pointer, ptr);
            if allocation.layout.size() > FRAME_SIZE {
                dealloc(allocation.value_pointer, allocation.layout);
                dealloc(
                    allocation.domain_id_pointer as *mut u8,
                    Layout::for_value(&0u64),
                );
            } else {
                let part = SharedHeapAllocationPart {
                    value_pointer: allocation.value_pointer,
                    domain_id_pointer: allocation.domain_id_pointer,
                };
                SHARED_HEAP_CACHE.insert(allocation.layout, part);
            }
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
            println_color!(34, "free_shared is Free, free {} data", data.len());
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
