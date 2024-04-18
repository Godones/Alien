//! This crate should implement the block device driver according to the VirtIO specification.
//! The [virtio-blk](virtio_blk) crate provides the safety abstraction for the VirtIO registers and buffers.
//! So this crate should only implement the driver logic with safe Rust code.
#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;
use alloc::boxed::Box;

use interface::BlkDeviceDomain;

mod svd_impl;

pub fn main() -> Box<dyn BlkDeviceDomain> {
    Box::new(svd_impl::BlkDomain)
}
