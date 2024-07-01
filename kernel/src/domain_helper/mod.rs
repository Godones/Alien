mod resource;
mod sheap;
mod storage_heap;
mod syscall;

extern crate alloc;

use alloc::{
    boxed::Box,
    collections::BTreeMap,
    format,
    string::{String, ToString},
};
use core::sync::atomic::AtomicU64;

use corelib::AlienResult;
pub use interface::DomainType;
use ksync::Mutex;
pub use resource::*;
pub use sheap::{checkout_shared_data, FreeShared, SHARED_HEAP_ALLOCATOR};
use spin::Once;
pub use storage_heap::*;
pub use syscall::DOMAIN_SYS;

static DOMAIN_IDS: AtomicU64 = AtomicU64::new(0);

struct DomainContainer {
    domains: BTreeMap<String, DomainType>,
    ty_counter: BTreeMap<String, u64>,
}

unsafe impl Send for DomainContainer {}

impl DomainContainer {
    pub const fn new() -> Self {
        Self {
            domains: BTreeMap::new(),
            ty_counter: BTreeMap::new(),
        }
    }
}
impl DomainContainer {
    fn insert(&mut self, identifier: String, domain: DomainType, unique: bool) -> String {
        if unique {
            if self.domains.contains_key(&identifier) {
                panic!(
                    "domain {} already exists, but it should be unique",
                    identifier
                );
            }
            platform::println!(
                "<attach domain>: {}, it's name is {}",
                identifier,
                identifier
            );
            self.domains.insert(identifier.clone(), domain);
            identifier
        } else {
            let counter = self.ty_counter.entry(identifier.clone()).or_insert(0);
            *counter += 1;
            let name = format!("{}-{}", identifier, counter);
            platform::println!("<attach domain>: {}, it's name is {}", identifier, name);
            self.domains.insert(name.clone(), domain);
            name
        }
    }
    fn get(&self, name: &str) -> Option<DomainType> {
        self.domains.get(name).map(|d| d.clone())
    }
}

static DOMAIN_CONTAINER: Mutex<DomainContainer> = Mutex::new(DomainContainer::new());

static DOMAIN_CREATE: Once<Box<dyn DomainCreate>> = Once::new();

/// Allocate a domain id
pub fn alloc_domain_id() -> u64 {
    DOMAIN_IDS.fetch_add(1, core::sync::atomic::Ordering::SeqCst)
}

/// Initialize the domain creation function
pub fn init_domain_create(domain_create: Box<dyn DomainCreate>) {
    DOMAIN_CREATE.call_once(|| domain_create);
}

/// find the domain which name is `domain_identifier`
pub fn query_domain(domain_identifier: &str) -> Option<DomainType> {
    match DOMAIN_CONTAINER.lock().get(&domain_identifier) {
        Some(domain) => Some(domain),
        None => None,
    }
}

/// Register a domain with a  identifier which may be unique
pub fn register_domain(identifier: &str, domain: DomainType, unique: bool) -> String {
    DOMAIN_CONTAINER
        .lock()
        .insert(identifier.to_string(), domain, unique)
}

pub trait DomainCreate: Send + Sync {
    fn create_domain(
        &self,
        domain_file_name: &str,
        identifier: &mut [u8],
    ) -> AlienResult<DomainType>;
}
