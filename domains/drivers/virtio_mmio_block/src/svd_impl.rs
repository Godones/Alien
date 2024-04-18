extern crate alloc;
use alloc::boxed::Box;
use core::ops::Range;

use basic::{
    io::{construct_ref_mut, SafeIORegion},
    println,
};
use constants::{AlienError, AlienResult};
use interface::{Basic, BlkDeviceDomain, DeviceBase};
use ksync::Mutex;
use log::error;
use rref::RRef;
use spin::Once;
use svdrivers::{BlkDriver, SvdOps, VirtQueue, PAGE_SIZE};

static BLK: Once<Mutex<BlkDriver>> = Once::new();

#[derive(Debug)]
pub struct BlkDomain;
impl Basic for BlkDomain {}
impl DeviceBase for BlkDomain {
    fn handle_irq(&self) -> constants::AlienResult<()> {
        todo!()
    }
}

impl BlkDeviceDomain for BlkDomain {
    fn init(&self, device_info: Range<usize>) -> AlienResult<()> {
        let region = &device_info;
        println!("virtio_blk_addr: {:#x}-{:#x}", region.start, region.end);
        let io_region = SafeIORW(SafeIORegion::from(region.clone()));
        let vq = make_queue();
        let blk = BlkDriver::new(Box::new(io_region), vq).map_err(to_alien_error)?;
        // blk.enable_receive_interrupt()?;
        BLK.call_once(|| Mutex::new(blk));
        Ok(())
    }
    fn read_block(&self, block: u32, mut data: RRef<[u8; 512]>) -> AlienResult<RRef<[u8; 512]>> {
        BLK.get()
            .unwrap()
            .lock()
            .read_blocks(block as _, data.as_mut_slice())
            .map_err(to_alien_error)?;
        Ok(data)
    }
    fn write_block(&self, block: u32, data: &RRef<[u8; 512]>) -> AlienResult<usize> {
        BLK.get()
            .unwrap()
            .lock()
            .write_blocks(block as _, data.as_ref())
            .map_err(to_alien_error)?;
        Ok(data.len())
    }
    fn get_capacity(&self) -> AlienResult<u64> {
        BLK.get().unwrap().lock().capacity().map_err(to_alien_error)
    }
    fn flush(&self) -> AlienResult<()> {
        BLK.get().unwrap().lock().flush().map_err(to_alien_error)
    }
}

fn to_alien_error(e: &str) -> AlienError {
    error!("{e}");
    AlienError::DOMAINCRASH
}

struct SafeIORW(SafeIORegion);
impl SvdOps for SafeIORW {
    fn read_at(&self, offset: usize) -> Result<u32, &'static str> {
        self.0.read_at(offset).map_err(|_| "OS read error")
    }
    fn write_at(&self, offset: usize, data: u32) -> Result<(), &'static str> {
        self.0.write_at(offset, data).map_err(|_| "OS write error")
    }
}

use alloc::collections::BTreeMap;

use basic::vm::frame::FrameTracker;
use spin::Lazy;
use svdrivers::{AvailRing, Descriptor, UsedRing};
// TODO: dealloc
static PAGE_RECORD: Lazy<Mutex<BTreeMap<usize, FrameTracker>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));
fn make_queue<'a>() -> VirtQueue<'a, { BlkDriver::QUEUE_SIZE }> {
    // get contiguous pages
    let tsize = VirtQueue::<{ BlkDriver::QUEUE_SIZE }>::total_size().unwrap();
    let frame = FrameTracker::new(tsize / PAGE_SIZE);
    let pa = frame.start_phy_addr();
    let va = frame.start_virt_addr();
    PAGE_RECORD.lock().insert(pa.as_usize(), frame);

    let avail_offset = VirtQueue::<{ BlkDriver::QUEUE_SIZE }>::avail_offset().unwrap();
    let used_offset = VirtQueue::<{ BlkDriver::QUEUE_SIZE }>::used_offset().unwrap();
    let q = VirtQueue::new(
        construct_ref_mut::<[Descriptor; BlkDriver::QUEUE_SIZE]>(va),
        construct_ref_mut::<AvailRing<{ BlkDriver::QUEUE_SIZE }>>(
            (va.as_usize() + avail_offset).into(),
        ),
        construct_ref_mut::<UsedRing<{ BlkDriver::QUEUE_SIZE }>>(
            (va.as_usize() + used_offset).into(),
        ),
        pa.as_usize(),
    )
    .unwrap();
    q
}
