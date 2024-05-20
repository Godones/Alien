#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::boxed::Box;
use core::fmt::Debug;

use basic::AlienResult;
use interface::{Basic, EmptyDeviceDomain};
use rref::RRefVec;

#[derive(Debug)]
pub struct RandomDeviceDomainImpl;

impl Basic for RandomDeviceDomainImpl {}

impl EmptyDeviceDomain for RandomDeviceDomainImpl {
    fn init(&self) -> AlienResult<()> {
        Ok(())
    }

    fn read(&self, mut data: RRefVec<u8>) -> AlienResult<RRefVec<u8>> {
        let mut current_time = basic::time::read_time_ms();
        data.as_mut_slice().iter_mut().for_each(|x| {
            *x = current_time as u8;
            current_time = current_time.wrapping_sub(1);
        });
        Ok(data)
    }
    fn write(&self, data: &RRefVec<u8>) -> AlienResult<usize> {
        Ok(data.len())
    }
}

pub fn main() -> Box<dyn EmptyDeviceDomain> {
    Box::new(RandomDeviceDomainImpl)
}
