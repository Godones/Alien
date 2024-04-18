extern crate alloc;

use alloc::{collections::BTreeMap, sync::Arc};
use core::{
    fmt::Debug,
    ops::{Deref, DerefMut, Range},
    ptr::NonNull,
};

use basic::{println, vm::frame::FrameTracker};
use constants::AlienResult;
use interface::{Basic, DeviceBase};
use ksync::Mutex;
use log::info;
use rref::RRef;
use spin::{Lazy, Once};
use virtio_drivers::{
    device::blk::VirtIOBlk,
    transport::mmio::{MmioTransport, VirtIOHeader},
    BufferDirection, Hal, PhysAddr,
};

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
    fn init(&self, address_range: Range<usize>) -> AlienResult<()> {
        let virtio_blk_addr = address_range.start;
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

struct HalImpl;

static PAGE_RECORD: Lazy<Mutex<BTreeMap<usize, FrameTracker>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

unsafe impl Hal for HalImpl {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let frame = FrameTracker::new(pages);
        let phys_addr = frame.start_phy_addr();
        PAGE_RECORD.lock().insert(phys_addr.as_usize(), frame);
        (
            phys_addr.as_usize(),
            NonNull::new(phys_addr.as_usize() as _).unwrap(),
        )
    }

    unsafe fn dma_dealloc(paddr: PhysAddr, _vaddr: NonNull<u8>, _pages: usize) -> i32 {
        let mut page_record = PAGE_RECORD.lock();
        let _frame = page_record.remove(&(paddr)).unwrap();
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        let vaddr = PAGE_RECORD.lock().get(&(paddr)).unwrap().start_virt_addr();
        NonNull::new(vaddr.as_usize() as *mut u8).unwrap()
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> PhysAddr {
        let vaddr = buffer.as_ptr() as *mut u8 as usize;
        vaddr
    }

    unsafe fn unshare(_paddr: PhysAddr, _buffer: NonNull<[u8]>, _direction: BufferDirection) {}
}
