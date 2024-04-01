#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{boxed::Box, string::ToString, sync::Arc};
use core::fmt::Debug;

use generic::GenericFsDomain;
use interface::FsDomain;
use ksync::Mutex;
use ramfs::{RamFs, RamFsProvider};
use vfscore::utils::VfsTimeSpec;

#[derive(Debug, Clone)]
pub struct ProviderImpl;
impl RamFsProvider for ProviderImpl {
    fn current_time(&self) -> VfsTimeSpec {
        VfsTimeSpec::new(0, 0)
    }
}

type RamFsDomain = GenericFsDomain;

pub fn main() -> Box<dyn FsDomain> {
    let fatfs = Arc::new(RamFs::<_, Mutex<()>>::new(ProviderImpl));
    Box::new(RamFsDomain::new(fatfs, "ramfs".to_string()))
}
