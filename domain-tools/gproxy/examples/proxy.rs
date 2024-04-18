#![feature(naked_functions)]
extern crate alloc;
use std::fmt::Debug;

use gproxy::{no_check, proxy, recover};

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
    #[recover]
    #[no_check]
    fn xxxx(&self, x: usize) -> AlienResult<()>;
    #[no_check]
    fn yyy(&self) -> AlienResult<()>;
}

gen_for_XXXDomain!();

#[no_mangle]
fn register_cont() {}

fn main() {}
