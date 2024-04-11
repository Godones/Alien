use constants::AlienResult;
use gproxy::proxy;

use crate::Basic;

#[proxy(SysCallDomainProxy)]
pub trait SysCallDomain: Basic {
    fn init(&self) -> AlienResult<()>;
    fn call(&self, syscall_id: usize, args: [usize; 6]) -> AlienResult<isize>;
}
