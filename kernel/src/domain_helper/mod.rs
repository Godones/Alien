mod sheap;
mod syscall;

extern crate alloc;

use alloc::{
    boxed::Box,
    collections::BTreeMap,
    format,
    string::{String, ToString},
};
use core::sync::atomic::AtomicU64;

pub use interface::DomainType;
use ksync::Mutex;
pub use sheap::SharedHeapAllocator;
use spin::{Lazy, Once};
pub use syscall::{register_domain_heap_resource, register_domain_syscall_resource, DomainSyscall};

static DOMAIN_IDS: AtomicU64 = AtomicU64::new(0);

struct DomainContainer {
    domains: BTreeMap<String, DomainType>,
    ty_counter: BTreeMap<String, u64>,
}

unsafe impl Send for DomainContainer {}

impl DomainContainer {
    pub fn new() -> Self {
        Self {
            domains: BTreeMap::new(),
            ty_counter: BTreeMap::new(),
        }
    }
}
impl DomainContainer {
    fn insert(&mut self, identifier: String, domain: DomainType, unique: bool) {
        if unique {
            if self.domains.contains_key(&identifier) {
                panic!(
                    "domain {} already exists, but it should be unique",
                    identifier
                );
            }
            platform::println!(
                "<register domain>: {}, it's name is {}",
                identifier,
                identifier
            );
            self.domains.insert(identifier, domain);
        } else {
            let counter = self.ty_counter.entry(identifier.clone()).or_insert(0);
            *counter += 1;
            let name = format!("{}-{}", identifier, counter);
            platform::println!("<register domain>: {}, it's name is {}", identifier, name);
            self.domains.insert(name, domain);
        }
    }
    fn get(&self, name: &str) -> Option<DomainType> {
        self.domains.get(name).map(|d| d.clone())
    }
}
// TODO! domain container
static DOMAIN_CONTAINER: Lazy<Mutex<DomainContainer>> =
    Lazy::new(|| Mutex::new(DomainContainer::new()));

static DOMAIN_CREATE: Once<Box<dyn DomainCreate>> = Once::new();

pub fn alloc_domain_id() -> u64 {
    DOMAIN_IDS.fetch_add(1, core::sync::atomic::Ordering::SeqCst)
}

pub fn register_domain(identifier: &str, domain: DomainType, unique: bool) {
    DOMAIN_CONTAINER
        .lock()
        .insert(identifier.to_string(), domain, unique);
}

/// Initialize the domain creation function
pub fn init_domain_create(domain_create: Box<dyn DomainCreate>) {
    DOMAIN_CREATE.call_once(|| domain_create);
}

pub fn query_domain(name: &str) -> Option<DomainType> {
    match DOMAIN_CONTAINER.lock().get(&name) {
        Some(domain) => Some(domain),
        None => None,
    }
}

pub trait DomainCreate: Send + Sync {
    fn create_domain(&self, identifier: &str) -> Option<DomainType>;
}
