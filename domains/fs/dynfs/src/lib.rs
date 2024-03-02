#![no_std]
#![deny(unsafe_code)]

extern crate alloc;

use alloc::sync::Arc;
use core::fmt::{Debug, Formatter};
use interface::{Basic, FsDomain};

pub struct DynFsDomain;

impl Basic for DynFsDomain {}

impl Debug for DynFsDomain {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "DevFsDomain")
    }
}

impl FsDomain for DynFsDomain {}

pub fn main() -> Arc<dyn FsDomain> {
    Arc::new(DynFsDomain)
}
