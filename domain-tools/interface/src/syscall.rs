use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;

use super::AlienResult;
use crate::Basic;

#[proxy(SysCallDomainProxy)]
pub trait SysCallDomain: Basic + DowncastSync {
    fn init(&self) -> AlienResult<()>;
    fn call(&self, syscall_id: usize, args: [usize; 6]) -> AlienResult<isize>;
}

impl_downcast!(sync SysCallDomain);
