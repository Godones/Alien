#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::{boxed::Box, string::ToString, sync::Arc};

use basic::sync::Mutex;
use dynfs::DynFsKernelProvider;
use generic::GenericFsDomain;
use interface::FsDomain;
use vfscore::utils::VfsTimeSpec;

#[derive(Clone)]
pub struct CommonFsProviderImpl;

impl DynFsKernelProvider for CommonFsProviderImpl {
    fn current_time(&self) -> VfsTimeSpec {
        VfsTimeSpec::new(0, 0)
    }
}

type PipeFs = dynfs::DynFs<CommonFsProviderImpl, Mutex<()>>;

type SysFsDomain = GenericFsDomain;

pub fn main() -> Box<dyn FsDomain> {
    let pipefs = Arc::new(PipeFs::new(CommonFsProviderImpl, "procfs"));
    Box::new(SysFsDomain::new(pipefs, "pipefs".to_string(), None))
}
