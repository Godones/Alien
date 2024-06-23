use alloc::{
    boxed::Box,
    collections::BTreeMap,
    string::{String, ToString},
    sync::Arc,
};
use core::{
    alloc::{AllocError, Allocator, GlobalAlloc, Layout},
    any::Any,
    ops::Deref,
    ptr::NonNull,
};

use spin::{Lazy, Mutex};
use storage::{DataStorageHeap, DomainDataStorage, SendAllocator};

#[derive(Debug)]
pub struct DomainDataMap {
    data: Arc<
        Mutex<BTreeMap<String, Box<Arc<dyn Any + Send + Sync, DataStorageHeap>, DataStorageHeap>>>,
    >,
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
}
impl DomainDataStorage for DomainDataMap {
    fn insert(
        &self,
        key: &str,
        value: Box<Arc<dyn Any + Send + Sync, DataStorageHeap>, DataStorageHeap>,
    ) -> Option<Box<Arc<dyn Any + Send + Sync, DataStorageHeap>, DataStorageHeap>> {
        self.data.lock().insert(key.to_string(), value)
    }

    fn get(&self, key: &str) -> Option<Arc<dyn Any + Send + Sync, DataStorageHeap>> {
        let data = self.data.lock();
        let v = data.get(key);
        let res = v.map(|v| v.deref().clone());
        res
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
        println_color!(
            34,
            "<DomainDataHeap> allocate from DataStorageHeap, size: {}",
            layout.size()
        );
        self.alloc_impl(layout, false)
    }
    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        println_color!(
            34,
            "<DomainDataHeap> deallocate from DataStorageHeap, size: {}",
            layout.size()
        );
        if layout.size() != 0 {
            // SAFETY: `layout` is non-zero in size,
            // other conditions must be upheld by the caller
            unsafe { GlobalAlloc::dealloc(self, ptr.as_ptr(), layout) }
        }
    }
}

impl SendAllocator for DomainDataHeap {}

pub static DATA_BASE: Lazy<Box<DomainDataMap>> = Lazy::new(|| Box::new(DomainDataMap::new()));
