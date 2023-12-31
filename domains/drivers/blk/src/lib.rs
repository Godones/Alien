//! This crate should implement the block device driver according to the VirtIO specification.
//! The [virtio-blk](virtio_blk) crate provides the safety abstraction for the VirtIO registers and buffers.
//! So this crate should only implement the driver logic with safe Rust code.
#![no_std]
#![deny(unsafe_code)]
extern crate alloc;
extern crate malloc;

use alloc::boxed::Box;
use interface::Basic;
use libsyscall::println;
use log::info;
use rref::RpcResult;
use virtio_blk::VirtIoBlk;

pub fn main(virtio_blk_addr: usize) -> Box<dyn interface::BlkDevice> {
    println!("virtio_blk_addr: {:#x}", virtio_blk_addr);
    Box::new(VirtIOBlk::new(virtio_blk_addr))
}

pub struct VirtIOBlk {
    driver: VirtIoBlk,
}

impl VirtIOBlk {
    pub fn new(virtio_blk_addr: usize) -> Self {
        Self {
            driver: VirtIoBlk::new(virtio_blk_addr),
        }
    }
}

impl Basic for VirtIOBlk {
    fn drop_self(self: Box<Self>) {
        info!("Drop VirtIOBlk");
        drop(self);
    }
}

impl interface::BlkDevice for VirtIOBlk {
    fn read(
        &mut self,
        block: u32,
        data: rref::RRef<[u8; 512]>,
    ) -> RpcResult<rref::RRef<[u8; 512]>> {
        let mut buf = data;
        self.driver
            .read_block(block as usize, buf.as_mut())
            .unwrap();
        // warn!("read block: {}, buf:{:#x}", block, buf[0]);
        panic!("read block: {}, buf:{:#x}", block, buf[0]);
        // Ok(buf)
    }
    fn write(&mut self, block: u32, data: &rref::RRef<[u8; 512]>) -> RpcResult<usize> {
        self.driver
            .write_block(block as usize, data.as_ref())
            .unwrap();
        Ok(data.len())
    }

    fn get_capacity(&self) -> RpcResult<u64> {
        Ok(self.driver.capacity() as u64)
    }

    fn flush(&self) -> RpcResult<()> {
        Ok(())
    }
}
