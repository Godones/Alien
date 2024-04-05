#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

use alloc::{boxed::Box, string::ToString, sync::Arc};
use core::fmt::Debug;

use fat_vfs::{FatFs, FatFsProvider};
use generic::GenericFsDomain;
use interface::FsDomain;
use ksync::Mutex;
use vfscore::utils::VfsTimeSpec;

#[derive(Debug, Clone)]
pub struct ProviderImpl;
impl FatFsProvider for ProviderImpl {
    fn current_time(&self) -> VfsTimeSpec {
        VfsTimeSpec::new(0, 0)
    }
}

type FatFsDomain = GenericFsDomain;

pub fn main() -> Box<dyn FsDomain> {
    let fatfs = Arc::new(FatFs::<_, Mutex<()>>::new(ProviderImpl));
    Box::new(FatFsDomain::new(fatfs, "fatfs".to_string(), None))
}
