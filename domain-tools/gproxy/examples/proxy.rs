#![feature(naked_functions)]
extern crate alloc;
use core::sync::atomic::AtomicU64;
use std::{fmt::Debug, ops::Range};

use gproxy::{no_check, proxy, recover};
use spin::{Mutex, Once, RwLock};
#[derive(Debug)]
pub enum AlienError {
    DOMAINCRASH,
}
type AlienResult<T> = Result<T, AlienError>;
pub trait Basic: Debug {
    fn is_active(&self) -> bool;
}

pub trait DomainReload {
    fn reload(&self) -> AlienResult<()>;
}

pub trait DeviceBase {
    fn handle_irq(&self) -> AlienResult<()> {
        Err(AlienError::DOMAINCRASH)
    }
}

#[proxy(XXXDomainProxy,Range<usize>)]
pub trait XXXDomain: Basic + DeviceBase {
    fn init(&self, t: Range<usize>) -> AlienResult<()>;
    #[recover]
    #[no_check]
    fn xxxx(&self, x: usize) -> AlienResult<()>;
    #[no_check]
    fn yyy(&self) -> AlienResult<()>;
}
#[derive(Debug)]
pub struct DomainLoader {}

impl DomainLoader {
    // pub fn call<T:?Sized>()->Box<T>{
    //
    // }
}

gen_for_XXXDomain!();

#[no_mangle]
fn register_cont() {}

fn main() {
    // Once::new();
}
