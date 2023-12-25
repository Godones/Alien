#![no_std]

mod trampoline;

extern crate alloc;

use alloc::boxed::Box;
use interface::{Basic, BlkDevice, Fs};
use rref::{RRef, RpcResult};

pub struct BlkDomainProxy {
    domain_id: u64,
    domain: Box<dyn BlkDevice>,
}

impl BlkDomainProxy {
    pub fn new(domain_id: u64, domain: Box<dyn BlkDevice>) -> Self {
        Self { domain_id, domain }
    }
}

impl Basic for BlkDomainProxy {}

impl BlkDevice for BlkDomainProxy {
    fn read(&mut self, block: u32, data: RRef<[u8; 512]>) -> RpcResult<RRef<[u8; 512]>> {
        self.domain.read(block, data)
    }
    fn write(&mut self, block: u32, data: &RRef<[u8; 512]>) -> RpcResult<usize> {
        self.domain.write(block, data)
    }
    fn get_capacity(&self) -> RpcResult<u64> {
        self.domain.get_capacity()
    }
    fn flush(&self) -> RpcResult<()> {
        self.domain.flush()
    }
}

pub struct FsDomainProxy {
    domain_id: u64,
    domain: Box<dyn Fs>,
}

impl FsDomainProxy {
    pub fn new(domain_id: u64, domain: Box<dyn Fs>) -> Self {
        Self { domain_id, domain }
    }
}

impl Basic for FsDomainProxy {}

impl Fs for FsDomainProxy {
    fn ls(&self, path: RRef<[u8; 512]>) -> RpcResult<RRef<[u8; 512]>> {
        self.domain.ls(path)
    }
}

fn test() {}
