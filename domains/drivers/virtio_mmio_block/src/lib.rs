//! This crate should implement the block device driver according to the VirtIO specification.
//! The [virtio-blk](virtio_blk) crate provides the safety abstraction for the VirtIO registers and buffers.
//! So this crate should only implement the driver logic with safe Rust code.
#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;
use alloc::boxed::Box;
use core::ops::Range;

use basic::{io::SafeIORegion, println};
use constants::AlienResult;
use interface::{Basic, BlkDeviceDomain, DeviceBase};
use ksync::Mutex;
use rref::RRef;
use spin::Once;
use virtio_drivers::{device::block::VirtIOBlk, transport::mmio::MmioTransport};
use virtio_mmio_common::{HalImpl, SafeIORW};

static BLK: Once<Mutex<VirtIOBlk<HalImpl, MmioTransport>>> = Once::new();

#[derive(Debug)]
pub struct BlkDomain;
impl Basic for BlkDomain {}

impl DeviceBase for BlkDomain {
    fn handle_irq(&self) -> AlienResult<()> {
        todo!()
    }
}

impl BlkDeviceDomain for BlkDomain {
    fn init(&self, device_info: Range<usize>) -> AlienResult<()> {
        let region = &device_info;
        println!("virtio_blk_addr: {:#x}-{:#x}", region.start, region.end);
        let io_region = SafeIORW(SafeIORegion::from(device_info));
        let transport = MmioTransport::new(Box::new(io_region)).unwrap();
        let blk = VirtIOBlk::<HalImpl, MmioTransport>::new(transport)
            .expect("failed to create virtio_blk");
        // blk.enable_receive_interrupt()?;
        BLK.call_once(|| Mutex::new(blk));
        Ok(())
    }
    fn read_block(&self, block: u32, mut data: RRef<[u8; 512]>) -> AlienResult<RRef<[u8; 512]>> {
        if basic::blk_crash_trick() {
            panic!("blk crash trick");
        }
        BLK.get()
            .unwrap()
            .lock()
            .read_blocks(block as _, data.as_mut_slice())
            .expect("failed to read block");
        Ok(data)
    }
    fn write_block(&self, block: u32, data: &RRef<[u8; 512]>) -> AlienResult<usize> {
        BLK.get()
            .unwrap()
            .lock()
            .write_blocks(block as _, data.as_ref())
            .expect("failed to write block");
        Ok(data.len())
    }
    fn get_capacity(&self) -> AlienResult<u64> {
        let size = BLK.get().unwrap().lock().capacity().unwrap();
        Ok(size)
    }
    fn flush(&self) -> AlienResult<()> {
        BLK.get().unwrap().lock().flush().unwrap();
        Ok(())
    }
}

pub fn main() -> Box<dyn BlkDeviceDomain> {
    Box::new(BlkDomain)
}
