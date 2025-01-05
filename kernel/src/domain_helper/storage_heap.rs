use alloc::{
    boxed::Box,
    collections::BTreeMap,
    string::{String, ToString},
    sync::Arc,
};
use core::{
    alloc::{AllocError, Allocator, GlobalAlloc, Layout},
    any::Any,
    ptr::NonNull,
};

use ksync::Mutex;
use storage::{DataStorageHeap, DomainDataStorage, SendAllocator};

pub struct DomainDataMapManager {
    map_per_domain: BTreeMap<u64, DomainDataMap>,
}

impl DomainDataMapManager {
    const fn new() -> Self {
        Self {
            map_per_domain: BTreeMap::new(),
        }
    }

    /// Create a new domain data map with the given domain id.
    fn create(&mut self, domain_id: u64) -> Option<DomainDataMap> {
        let map = DomainDataMap::new();
        self.map_per_domain.insert(domain_id, map)
    }

    /// Get the domain data map with the given domain id.
    fn get(&self, domain_id: u64) -> Option<&DomainDataMap> {
        self.map_per_domain.get(&domain_id)
    }

    /// Remove the domain data map with the given domain id.
    fn remove(&mut self, domain_id: u64) -> Option<DomainDataMap> {
        self.map_per_domain.remove(&domain_id)
    }

    /// Move the domain data map from the source domain to the target domain.
    fn move_domain(&mut self, from: u64, to: u64) {
        if let Some(data) = self.remove(from) {
            // println_color!(32, "move domain database, it's length: {}", data.len());
            self.map_per_domain.insert(to, data);
        }
    }
}

type ArcValueType = Arc<dyn Any + Send + Sync, DataStorageHeap>;

#[derive(Debug)]
pub struct DomainDataMap {
    data: Arc<Mutex<BTreeMap<String, ArcValueType>>>,
}

impl Clone for DomainDataMap {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

unsafe impl Send for DomainDataMap {}
unsafe impl Sync for DomainDataMap {}

impl DomainDataMap {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    pub fn len(&self) -> usize {
        self.data.lock().len()
    }
}
impl DomainDataStorage for DomainDataMap {
    /// Insert a new key-value pair into the data map.
    ///
    /// If the key already exists, the value will be replaced and returned.
    /// Otherwise, `None` will be returned.
    fn insert(&self, key: &str, value: ArcValueType) -> Option<ArcValueType> {
        // println_color!(32, "insert key: {}", key);
        self.data.lock().insert(key.to_string(), value)
    }

    /// Get the value with the given key.
    fn get(&self, key: &str) -> Option<ArcValueType> {
        let data = self.data.lock();
        let v = data.get(key);
        // println_color!(32, "get key: {}, value: {:?}", key, v.is_some());

        v.cloned()
    }

    /// Remove the value with the given key.
    ///
    /// If the key exists, the value will be removed and returned.
    /// Otherwise, `None` will be returned.
    fn remove(&self, key: &str) -> Option<ArcValueType> {
        let mut data = self.data.lock();

        // println_color!(31, "remove key: {}", key);
        data.remove(key)
    }
}

#[derive(Debug, Clone)]
pub struct DomainDataHeap;

unsafe impl GlobalAlloc for DomainDataHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        alloc::alloc::alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        alloc::alloc::dealloc(ptr, layout)
    }
}

impl DomainDataHeap {
    #[inline]
    fn alloc_impl(&self, layout: Layout, zeroed: bool) -> Result<NonNull<[u8]>, AllocError> {
        match layout.size() {
            0 => Ok(NonNull::slice_from_raw_parts(layout.dangling(), 0)),
            // SAFETY: `layout` is non-zero in size,
            size => unsafe {
                let raw_ptr = if zeroed {
                    GlobalAlloc::alloc_zeroed(self, layout)
                } else {
                    GlobalAlloc::alloc(self, layout)
                };
                let ptr = NonNull::new(raw_ptr).ok_or(AllocError)?;
                Ok(NonNull::slice_from_raw_parts(ptr, size))
            },
        }
    }
}

unsafe impl Allocator for DomainDataHeap {
    #[inline]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        // println_color!(
        //     34,
        //     "<DomainDataHeap> allocate from DataStorageHeap, size: {}",
        //     layout.size()
        // );
        self.alloc_impl(layout, false)
    }
    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        // println_color!(
        //     34,
        //     "<DomainDataHeap> deallocate from DataStorageHeap, size: {}",
        //     layout.size()
        // );
        if layout.size() != 0 {
            // SAFETY: `layout` is non-zero in size,
            // other conditions must be upheld by the caller
            unsafe { GlobalAlloc::dealloc(self, ptr.as_ptr(), layout) }
        }
    }
}

impl SendAllocator for DomainDataHeap {}

pub static DOMAIN_DATA_ALLOCATOR: &'static dyn SendAllocator = &DomainDataHeap;
static DATA_BASE_MANAGER: Mutex<DomainDataMapManager> = Mutex::new(DomainDataMapManager::new());

/// Create a new domain data map with the given domain id.
pub fn create_domain_database(domain_id: u64) {
    let mut manager = DATA_BASE_MANAGER.lock();
    manager.create(domain_id);
    println_color!(32, "create domain database for domain_id: {}", domain_id);
}

/// Get the domain data map with the given domain id.
pub fn get_domain_database(domain_id: u64) -> Option<Box<DomainDataMap>> {
    let manager = DATA_BASE_MANAGER.lock();
    let res = manager.get(domain_id).map(|v| Box::new(v.clone()));
    res
}

/// Remove the domain data map with the given domain id.
#[allow(unused)]
pub fn remove_domain_database(domain_id: u64) -> Option<Box<DomainDataMap>> {
    let mut manager = DATA_BASE_MANAGER.lock();
    let res = manager.remove(domain_id).map(Box::new);
    println_color!(31, "remove domain database for domain_id: {}", domain_id);
    res
}

/// Move the domain data map from the source domain to the target domain.
pub fn move_domain_database(from: u64, to: u64) {
    let mut manager = DATA_BASE_MANAGER.lock();
    manager.move_domain(from, to);
    // println_color!(32, "move domain database from {} to {}", from, to);
}
