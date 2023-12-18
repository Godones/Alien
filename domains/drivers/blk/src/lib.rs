#![no_std]
#![deny(unsafe_code)]
extern crate alloc;
extern crate malloc;

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;
use rref::RpcResult;

pub fn main() -> Box<dyn interface::BlkDevice> {
    Box::new(NullDev::new())
}

pub struct NullDev{
    data:Vec<u8>
}

impl NullDev{
    pub fn new() -> Self {
        Self {
            data: vec![0; 4096],
        }
    }
}


impl interface::BlkDevice for NullDev {
    fn read(
        &self,
        _block: u32,
        data: rref::RRef<[u8; 4096]>,
    ) -> RpcResult<rref::RRef<[u8; 4096]>> {
        Ok(data)
    }

    fn write(&self, _block: u32, data: &rref::RRef<[u8; 4096]>) -> rref::RpcResult<usize> {
        Ok(data.len())
    }

    fn get_capacity(&self) -> RpcResult<u64> {
        Ok(self.data.len() as u64)
    }
}
