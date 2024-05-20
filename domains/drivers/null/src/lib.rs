#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::boxed::Box;
use core::fmt::Debug;

use basic::AlienResult;
use interface::{Basic, EmptyDeviceDomain};
use rref::RRefVec;

#[derive(Debug)]
pub struct NullDeviceDomainImpl;

impl Basic for NullDeviceDomainImpl {}

impl EmptyDeviceDomain for NullDeviceDomainImpl {
    fn init(&self) -> AlienResult<()> {
        Ok(())
    }

    fn read(&self, mut data: RRefVec<u8>) -> AlienResult<RRefVec<u8>> {
        data.as_mut_slice().fill(0);
        Ok(data)
    }
    fn write(&self, data: &RRefVec<u8>) -> AlienResult<usize> {
        Ok(data.len())
    }
}

pub fn main() -> Box<dyn EmptyDeviceDomain> {
    Box::new(NullDeviceDomainImpl)
}
