#![no_std]

use rref::{RRef, RpcResult};

pub trait BlkDevice: Send + Sync {
    fn read(&self, block: u32, data: RRef<[u8; 4096]>) -> RpcResult<RRef<[u8; 4096]>>;
    fn write(&self, block: u32, data: &RRef<[u8; 4096]>) -> RpcResult<usize>;
    fn get_capacity(&self) -> RpcResult<u64>;
}
