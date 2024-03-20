//! This crate should implement the block device driver according to the VirtIO specification.
//! The [virtio-blk](virtio_blk) crate provides the safety abstraction for the VirtIO registers and buffers.
//! So this crate should only implement the driver logic with safe Rust code.
#![no_std]
// #![deny(unsafe_code)]
extern crate alloc;

use alloc::sync::Arc;
use basic::frame::FrameTracker;
use basic::println;
use constants::AlienResult;
use core::fmt::Debug;
use core::mem::forget;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use interface::{Basic, DeviceBase, DeviceInfo};
use ksync::Mutex;
use log::info;
use rref::RRef;
use spin::Once;
use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_drivers::{BufferDirection, Hal, PhysAddr};

pub struct VirtIOBlkDomain;

static VIRTIO_BLK: Once<Arc<Mutex<VirtIOBlkWrapper>>> = Once::new();

struct VirtIOBlkWrapper {
    blk: VirtIOBlk<HalImpl, MmioTransport>,
}

unsafe impl Send for VirtIOBlkWrapper {}
unsafe impl Sync for VirtIOBlkWrapper {}

impl Debug for VirtIOBlkWrapper {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VirtIOBlkWrapper")
    }
}

impl VirtIOBlkWrapper {
    fn new(blk: VirtIOBlk<HalImpl, MmioTransport>) -> Self {
        Self { blk }
    }
}

impl Deref for VirtIOBlkWrapper {
    type Target = VirtIOBlk<HalImpl, MmioTransport>;

    fn deref(&self) -> &Self::Target {
        &self.blk
    }
}

impl DerefMut for VirtIOBlkWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.blk
    }
}

impl Debug for VirtIOBlkDomain {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VirtIOBlk")
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
    fn handle_irq(&self) -> AlienResult<()> {
        todo!()
    }
}

impl interface::BlkDeviceDomain for VirtIOBlkDomain {
    fn init(&self, device_info: &DeviceInfo) -> AlienResult<()> {
        let virtio_blk_addr = device_info.address_range.start;
        println!("virtio_blk_addr: {:#x?}", virtio_blk_addr);
        let header = NonNull::new(virtio_blk_addr as *mut VirtIOHeader).unwrap();
        let transport = unsafe { MmioTransport::new(header) }.unwrap();
        let blk = VirtIOBlk::<HalImpl, MmioTransport>::new(transport)
            .expect("failed to create blk driver");
        let blk = Arc::new(Mutex::new(VirtIOBlkWrapper::new(blk)));
        VIRTIO_BLK.call_once(|| blk);
        Ok(())
    }

    fn read_block(&self, block: u32, data: RRef<[u8; 512]>) -> AlienResult<RRef<[u8; 512]>> {
        let mut buf = data;
        VIRTIO_BLK
            .get()
            .unwrap()
            .lock()
            .read_blocks(block as usize, buf.as_mut())
            .unwrap();
        // warn!("read block: {}, buf:{:#x}", block, buf[0]);
        // trick
        if basic::blk_crash_trick() {
            panic!("read block: {}, buf:{:#x}", block, buf[0]);
        }
        Ok(buf)
    }
    fn write_block(&self, block: u32, data: &rref::RRef<[u8; 512]>) -> AlienResult<usize> {
        VIRTIO_BLK
            .get()
            .unwrap()
            .lock()
            .write_blocks(block as usize, data.as_ref())
            .unwrap();
        Ok(data.len())
    }

    fn get_capacity(&self) -> AlienResult<u64> {
        Ok(VIRTIO_BLK.get().unwrap().lock().capacity() * 512)
    }

    fn flush(&self) -> AlienResult<()> {
        Ok(())
    }
}

pub fn main() -> Arc<dyn interface::BlkDeviceDomain> {
    Arc::new(VirtIOBlkDomain)
}

struct HalImpl;

unsafe impl Hal for HalImpl {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let frame = FrameTracker::new(pages);
        let ptr = frame.start();
        forget(frame);
        (ptr, NonNull::new(ptr as _).unwrap())
    }

    unsafe fn dma_dealloc(paddr: PhysAddr, _vaddr: NonNull<u8>, pages: usize) -> i32 {
        let _frame = FrameTracker::from_raw(paddr, pages);
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
