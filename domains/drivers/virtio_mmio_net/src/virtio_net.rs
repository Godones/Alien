use alloc::collections::BTreeMap;
use core::{
    ops::{Deref, DerefMut, Range},
    ptr::NonNull,
};

use basic::vm::frame::FrameTracker;
use ksync::Mutex;
use spin::Lazy;
use virtio_drivers::{
    device::net::VirtIONet,
    transport::mmio::{MmioTransport, VirtIOHeader},
    BufferDirection, Hal, PhysAddr,
};

pub const NET_QUEUE_SIZE: usize = 128;
pub const NET_BUF_LEN: usize = 4096;

pub struct VirtIoNetWrapper {
    net: VirtIONet<HalImpl, MmioTransport, NET_QUEUE_SIZE>,
}
unsafe impl Send for VirtIoNetWrapper {}
unsafe impl Sync for VirtIoNetWrapper {}

impl VirtIoNetWrapper {
    pub fn new(address_range: Range<usize>) -> Self {
        let virtio_net_addr = address_range.start;
        basic::println!("virtio_net_addr: {:#x?}", virtio_net_addr);

        let header = NonNull::new(virtio_net_addr as *mut VirtIOHeader).unwrap();
        let transport = unsafe { MmioTransport::new(header) }.unwrap();

        let net = VirtIONet::<HalImpl, MmioTransport, NET_QUEUE_SIZE>::new(transport, NET_BUF_LEN)
            .expect("failed to create gpu driver");
        Self { net }
    }
}

impl Deref for VirtIoNetWrapper {
    type Target = VirtIONet<HalImpl, MmioTransport, NET_QUEUE_SIZE>;

    fn deref(&self) -> &Self::Target {
        &self.net
    }
}

impl DerefMut for VirtIoNetWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.net
    }
}

pub struct HalImpl;

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
