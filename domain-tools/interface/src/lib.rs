#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use rref::{RRef, RpcResult};

pub trait Basic {
    fn drop_self(self: Box<Self>) {
        drop(self);
    }
}

pub trait BlkDevice: Send + Sync + Basic {
    fn read(&mut self, block: u32, data: RRef<[u8; 512]>) -> RpcResult<RRef<[u8; 512]>>;
    fn write(&mut self, block: u32, data: &RRef<[u8; 512]>) -> RpcResult<usize>;
    fn get_capacity(&self) -> RpcResult<u64>;
    fn flush(&self) -> RpcResult<()>;
}

pub trait Fs: Send + Sync + Basic {
    fn ls(&self, path: RRef<[u8; 512]>) -> RpcResult<RRef<[u8; 512]>>;
}
