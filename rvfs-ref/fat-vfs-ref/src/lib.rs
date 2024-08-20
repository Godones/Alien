#![cfg_attr(not(test), no_std)]
#![feature(trait_alias)]
mod device;

mod fs;
mod inode;

extern crate alloc;

use alloc::sync::Arc;
use core::fmt::{Debug, Formatter};

use fatfs::*;
pub use fs::FatFs;
use lock_api::Mutex;
use vfscore::utils::VfsTimeSpec;

use crate::device::FatDevice;

pub trait VfsRawMutex = lock_api::RawMutex + Send + Sync;

pub trait FatFsProvider: Send + Sync + Clone {
    fn current_time(&self) -> VfsTimeSpec;
}

#[allow(unused)]
#[derive(Clone)]
struct TimeProviderImpl<T> {
    provider: T,
}

impl<T: FatFsProvider> Debug for TimeProviderImpl<T> {
    fn fmt(&self, _f: &mut Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}

impl<T: FatFsProvider> TimeProvider for TimeProviderImpl<T> {
    fn get_current_date(&self) -> Date {
        let _time_spec = self.provider.current_time();
        // todo!(translate time_spec to Date)
        Date::new(2023, 10, 10)
    }

    fn get_current_date_time(&self) -> DateTime {
        let _time_spec = self.provider.current_time();
        // todo!(translate time_spec to DateTime)
        DateTime::new(Date::new(2023, 10, 10), Time::new(12, 12, 12, 12))
    }
}

type FatDir = Dir<FatDevice, DefaultTimeProvider, LossyOemCpConverter>;
type FatFile = File<FatDevice, DefaultTimeProvider, LossyOemCpConverter>;
