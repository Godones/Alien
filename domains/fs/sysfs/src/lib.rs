#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::{boxed::Box, string::ToString, sync::Arc};

use dynfs::DynFsKernelProvider;
use generic::GenericFsDomain;
use interface::FsDomain;
use ksync::Mutex;
use vfscore::utils::VfsTimeSpec;
#[derive(Clone)]
pub struct CommonFsProviderImpl;

impl DynFsKernelProvider for CommonFsProviderImpl {
    fn current_time(&self) -> VfsTimeSpec {
        VfsTimeSpec::new(0, 0)
    }
}

type SysFs = dynfs::DynFs<CommonFsProviderImpl, Mutex<()>>;

type SysFsDomain = GenericFsDomain;

pub fn main() -> Box<dyn FsDomain> {
    let sysfs = Arc::new(SysFs::new(CommonFsProviderImpl, "procfs"));
    Box::new(SysFsDomain::new(sysfs, "sysfs".to_string(), None))
}
