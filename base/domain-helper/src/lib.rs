#![no_std]

mod sheap;
mod syscall;
mod task;

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use core::sync::atomic::AtomicU64;
use ksync::Mutex;
use spin::Lazy;

pub use sheap::SharedHeapAllocator;
pub use syscall::{register_domain_heap_resource, register_domain_syscall_resource, DomainSyscall};

pub use interface::DomainType;

static DOMAIN_IDS: AtomicU64 = AtomicU64::new(0);

struct DomainContainer {
    domains: BTreeMap<String, DomainType>,
}

unsafe impl Send for DomainContainer {}

impl DomainContainer {
    pub fn new() -> Self {
        Self {
            domains: BTreeMap::new(),
        }
    }
}
impl DomainContainer {
    fn insert(&mut self, name: String, domain: DomainType) {
        self.domains.insert(name, domain);
    }
    fn get(&self, name: &str) -> Option<DomainType> {
        self.domains.get(name).map(|d| d.clone())
    }
}
// TODO! domain container
static DOMAIN_CONTAINER: Lazy<Mutex<DomainContainer>> =
    Lazy::new(|| Mutex::new(DomainContainer::new()));
pub fn alloc_domain_id() -> u64 {
    DOMAIN_IDS.fetch_add(1, core::sync::atomic::Ordering::SeqCst)
}

pub fn register_domain(name: &str, domain: DomainType) {
    platform::println!("register domain: {}", name);
    DOMAIN_CONTAINER.lock().insert(name.to_string(), domain);
}

pub fn query_domain(name: &str) -> Option<DomainType> {
    match DOMAIN_CONTAINER.lock().get(&name) {
        Some(domain) => Some(domain),
        None => None,
    }
}
