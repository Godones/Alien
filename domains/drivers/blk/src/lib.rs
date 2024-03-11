//! This crate should implement the block device driver according to the VirtIO specification.
//! The [virtio-blk](virtio_blk) crate provides the safety abstraction for the VirtIO registers and buffers.
//! So this crate should only implement the driver logic with safe Rust code.
#![no_std]
// #![deny(unsafe_code)]
extern crate alloc;
extern crate malloc;

use alloc::sync::Arc;
use core::fmt::Debug;
use core::ptr::NonNull;
use interface::{Basic, DeviceBase, DeviceInfo};
use ksync::Mutex;
use libsyscall::println;
use log::info;
use rref::{RRef, RRefVec, RpcResult};
use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_drivers::{BufferDirection, Hal, PhysAddr};

pub struct VirtIOBlkDomain {
    device: Arc<Mutex<VirtIOBlk<HalImpl, MmioTransport>>>,
}
unsafe impl Send for VirtIOBlkDomain {}
unsafe impl Sync for VirtIOBlkDomain {}

impl Debug for VirtIOBlkDomain {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VirtIOBlk")
    }
}

impl VirtIOBlkDomain {
    pub fn new(virtio_blk_addr: usize) -> Self {
        let header = NonNull::new(virtio_blk_addr as *mut VirtIOHeader).unwrap();
        let transport = unsafe { MmioTransport::new(header) }.unwrap();
        Self {
            device: Arc::new(Mutex::new(VirtIOBlk::<HalImpl, MmioTransport>::new(transport)
            .expect("failed to create blk driver"))),
        }
    }
}

impl Drop for VirtIOBlkDomain {
    fn drop(&mut self) {
        info!("Drop VirtIOBlk");
    }
}

impl Basic for VirtIOBlkDomain {
    // fn drop_self(self: Box<Self>) {
    //     info!("Drop VirtIOBlk");
    //     drop(self);
    // }
}

impl DeviceBase for VirtIOBlkDomain {
    fn handle_irq(&self) -> RpcResult<()> {
        todo!()
    }
}

impl interface::BlkDeviceDomain for VirtIOBlkDomain {
    fn read_block(
        &self,
        block: u32,
        data: rref::RRef<[u8; 512]>,
    ) -> RpcResult<rref::RRef<[u8; 512]>> {
        let mut buf = data;
        self.device
            .lock()
            .read_blocks(block as usize, buf.as_mut())
            .unwrap();
        // warn!("read block: {}, buf:{:#x}", block, buf[0]);
        // trick
        if libsyscall::blk_crash_trick() {
            panic!("read block: {}, buf:{:#x}", block, buf[0]);
        }
        Ok(buf)
    }
    fn write_block(&self, block: u32, data: &rref::RRef<[u8; 512]>) -> RpcResult<usize> {
        self.device
            .lock()
            .write_blocks(block as usize, data.as_ref())
            .unwrap();
        Ok(data.len())
    }

    fn get_capacity(&self) -> RpcResult<u64> {
        Ok(self.device.lock().capacity() as u64 * 512)
    }

    fn flush(&self) -> RpcResult<()> {
        Ok(())
    }
}

pub fn main() -> Arc<dyn interface::BlkDeviceDomain> {
    let devices_domain = libsyscall::get_devices_domain().unwrap();
    let name = RRefVec::from_slice("virtio-mmio-block".as_bytes());

    let info = RRef::new(DeviceInfo {
        address_range: Default::default(),
        irq: RRef::new(0),
        compatible: RRefVec::new(0, 64),
    });

    let info = devices_domain.get_device(name, info).unwrap();

    let virtio_blk_addr = &info.address_range;

    println!("virtio_blk_addr: {:#x?}", virtio_blk_addr);
    Arc::new(VirtIOBlkDomain::new(virtio_blk_addr.start))
}

struct HalImpl;

unsafe impl Hal for HalImpl {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let ptr = libsyscall::alloc_raw_pages(pages);
        (ptr as usize, NonNull::new(ptr).unwrap())
    }

    unsafe fn dma_dealloc(paddr: PhysAddr, _vaddr: NonNull<u8>, pages: usize) -> i32 {
        libsyscall::free_raw_pages(paddr as *mut u8, pages);
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        NonNull::new(paddr as *mut u8).unwrap()
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> PhysAddr {
        let vaddr = buffer.as_ptr() as *mut u8 as usize;
        vaddr
    }

    unsafe fn unshare(_paddr: PhysAddr, _buffer: NonNull<[u8]>, _direction: BufferDirection) {}
}
