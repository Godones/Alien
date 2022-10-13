use core::alloc::{GlobalAlloc, Layout};
use spin::{Mutex, Spin};
use crate::mm::slab::{alloc_from_slab, create_cache, dealloc_to_slab, MemCache, print_slab_system_info};
struct CacheInfo<'a> {
    size: u32,
    align: u32,
    name: &'a [u8],
}

impl CacheInfo<'_> {
    pub const fn new(size: u32, align: u32, name: &'static [u8]) -> Self {
        CacheInfo { size, align, name }
    }
}

const CACHE_INFO_MAX: usize = 9;
const KMALLOC_INFO: [CacheInfo; CACHE_INFO_MAX] = [
    CacheInfo::new(8, 8, b"kmalloc-8"),
    CacheInfo::new(16, 8, b"kmalloc-16"),
    CacheInfo::new(32, 8, b"kmalloc-32"),
    CacheInfo::new(64, 8, b"kmalloc-64"),
    CacheInfo::new(128, 8, b"kmalloc-128"),
    CacheInfo::new(256, 8, b"kmalloc-256"),
    CacheInfo::new(512, 8, b"kmalloc-512"),
    CacheInfo::new(1024, 8, b"kmalloc-1024"),
    CacheInfo::new(2048, 8, b"kmalloc-2048"),
];

pub fn init_kmalloc() {
    for i in 0..CACHE_INFO_MAX {
        let info = &KMALLOC_INFO[i];
        create_cache(info.name, info.size, 1, info.align);
    }
    print_slab_system_info();
}

pub struct SlabAllocator{
    cache:Mutex<usize>
}

impl SlabAllocator {
    pub const fn new() -> Self {
        SlabAllocator {
            cache: Mutex::new(0),
        }
    }
}

unsafe impl GlobalAlloc for SlabAllocator{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let align = layout.align();
        let _ = self.cache.lock();
        alloc_from_slab(size, align).unwrap()
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        dealloc_to_slab(ptr);
    }
}
