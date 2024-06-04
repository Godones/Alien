use alloc::{boxed::Box, collections::BTreeMap, vec::Vec};

use config::FRAME_BITS;
use ksync::Mutex;

use crate::domain_helper::{DomainSyscall, SharedHeapAllocator};

pub(super) static DOMAIN_RESOURCE: Mutex<DomainResource> = Mutex::new(DomainResource::new());
pub struct DomainResource {
    page_map: BTreeMap<u64, Vec<(usize, usize)>>,
    syscall: BTreeMap<u64, usize>,
    allocator: BTreeMap<u64, usize>,
}

impl DomainResource {
    pub const fn new() -> Self {
        Self {
            page_map: BTreeMap::new(),
            syscall: BTreeMap::new(),
            allocator: BTreeMap::new(),
        }
    }

    pub fn insert_page_map(&mut self, domain_id: u64, page: (usize, usize)) {
        let vec = self.page_map.entry(domain_id).or_insert(Vec::new());
        vec.push(page);
    }

    pub fn free_page_map(&mut self, domain_id: u64, page: usize) {
        let vec = self.page_map.get_mut(&domain_id).unwrap();
        vec.retain(|(s, _)| *s != page);
    }

    pub fn insert_syscall(&mut self, domain_id: u64, syscall_addr: usize) {
        self.syscall.insert(domain_id, syscall_addr);
    }

    pub fn insert_allocator(&mut self, domain_id: u64, allocator_addr: usize) {
        self.allocator.insert(domain_id, allocator_addr);
    }
}

pub fn register_domain_syscall_resource(domain_id: u64, syscall_addr: usize) {
    DOMAIN_RESOURCE
        .lock()
        .insert_syscall(domain_id, syscall_addr);
}

pub fn register_domain_heap_resource(domain_id: u64, heap_addr: usize) {
    DOMAIN_RESOURCE
        .lock()
        .insert_allocator(domain_id, heap_addr);
}

pub fn free_domain_resource(domain_id: u64) {
    println!("free_domain_resource for domain_id: {}", domain_id);
    let mut binding = DOMAIN_RESOURCE.lock();
    // free pages
    if let Some(vec) = binding.page_map.remove(&domain_id) {
        for (page_start, n) in vec {
            let page_end = page_start + n;
            warn!(
                "[Domain: {}] free pages: [{:#x}-{:#x}]",
                domain_id,
                page_start << FRAME_BITS,
                page_end << FRAME_BITS
            );
            mem::free_frames((page_start << FRAME_BITS) as *mut u8, n);
        }
    }

    // free Box<DomainSyscall>
    let ptr = binding.syscall.remove(&domain_id).unwrap();
    let _syscall_resource = unsafe { Box::from_raw(ptr as *mut DomainSyscall) };
    drop(_syscall_resource);
    warn!("[Domain: {}] free DomainSyscall resource", domain_id);

    // free Box<SharedHeapAllocator>
    let ptr = binding.allocator.remove(&domain_id).unwrap();
    let _allocator = unsafe { Box::from_raw(ptr as *mut SharedHeapAllocator) };
    drop(_allocator);
    warn!("[Domain: {}] free SharedHeapAllocator resource", domain_id);
}
