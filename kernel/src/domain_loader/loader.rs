use alloc::boxed::Box;
use core::{any::Any, fmt::Debug};

use loader::{DomainArea, DomainVmOps};
use mem::{VirtAddr, VirtDomainArea};
use storage::StorageArg;

use crate::{
    domain_helper,
    domain_helper::{DOMAIN_DATA_ALLOCATOR, DOMAIN_SYS, SHARED_HEAP_ALLOCATOR},
};
pub type DomainLoader = loader::DomainLoader<VmOpsImpl>;

pub trait DomainCall {
    fn call_main<T: ?Sized>(&self, id: u64, use_old_id: Option<u64>) -> Box<T>;
}

impl DomainCall for DomainLoader {
    fn call_main<T: ?Sized>(&self, id: u64, use_old_id: Option<u64>) -> Box<T> {
        let callback = |use_old_id: Option<u64>| {
            let syscall = DOMAIN_SYS;
            let heap = SHARED_HEAP_ALLOCATOR;
            let data_map = if let Some(old_id) = use_old_id
                && old_id != u64::MAX
            {
                let database = domain_helper::get_domain_database(old_id).unwrap();
                domain_helper::move_domain_database(old_id, id);
                database
            } else {
                domain_helper::create_domain_database(id);
                domain_helper::get_domain_database(id).unwrap()
            };
            let data_map_ptr = Box::into_raw(data_map);
            domain_helper::register_domain_resource(id, data_map_ptr as usize);
            let storage_arg =
                unsafe { StorageArg::new(DOMAIN_DATA_ALLOCATOR, Box::from_raw(data_map_ptr)) };
            (syscall, heap, storage_arg)
        };
        self.call::<_, dyn corelib::CoreFunction, _>(id, use_old_id, callback)
    }
}

#[derive(Debug)]
struct VirtDomainAreaWrapper(VirtDomainArea);

impl DomainArea for VirtDomainAreaWrapper {
    fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    fn as_mut_slice(&self) -> &mut [u8] {
        self.0.as_mut_slice()
    }

    fn start_virtual_address(&self) -> VirtAddr {
        VirtAddr::from(self.0.as_ptr() as usize)
    }

    fn any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

pub struct VmOpsImpl;

impl DomainVmOps for VmOpsImpl {
    fn map_domain_area(size: usize) -> Box<dyn DomainArea> {
        let domain_area = mem::map_domain_region(size);
        Box::new(VirtDomainAreaWrapper(domain_area))
    }

    fn unmap_domain_area(area: Box<dyn DomainArea>) {
        let area = area.any().downcast::<VirtDomainAreaWrapper>().unwrap().0;
        mem::unmap_domain_area(area);
    }

    fn set_memory_x(start: usize, pages: usize) -> Result<(), &'static str> {
        mem::set_memory_x(start, pages).map_err(|_| "set_memory_x failed")
    }
}
