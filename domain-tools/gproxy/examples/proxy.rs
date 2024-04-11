extern crate alloc;
use std::fmt::Debug;

use gproxy::proxy;

pub enum AlienError {
    DOMAINCRASH,
}
type AlienResult<T> = Result<T, AlienError>;
pub trait Basic: Debug {
    fn is_active(&self) -> bool;
}

pub trait DeviceBase {
    fn handle_irq(&self) -> AlienResult<()> {
        Err(AlienError::DOMAINCRASH)
    }
}

#[proxy(XXXDomainProxy)]
pub trait XXXDomain: Basic + DeviceBase {
    fn init(&self) -> AlienResult<()>;
    fn xxxx(&self, x: usize) -> AlienResult<()>;
}

gen_for_XXXDomain!();

fn main() {}
