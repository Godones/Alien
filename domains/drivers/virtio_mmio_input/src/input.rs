use alloc::collections::BTreeMap;
use core::{
    ops::{Deref, DerefMut, Range},
    ptr::NonNull,
};

use basic::vm::frame::FrameTracker;
use ksync::Mutex;
use spin::Lazy;
use virtio_drivers::{
    device::input::VirtIOInput,
    transport::mmio::{MmioTransport, VirtIOHeader},
    BufferDirection, Hal, PhysAddr,
};

pub struct VirtioInputWrapper {
    input: VirtIOInput<HalImpl, MmioTransport>,
}

unsafe impl Send for VirtioInputWrapper {}
unsafe impl Sync for VirtioInputWrapper {}

impl VirtioInputWrapper {
    pub fn new(address_range: Range<usize>) -> Self {
        let input_addr = address_range.start;
        basic::println!("input_addr: {:#x?}", input_addr);

        let header = NonNull::new(input_addr as *mut VirtIOHeader).unwrap();
        let transport = unsafe { MmioTransport::new(header) }.unwrap();

        let input = VirtIOInput::<HalImpl, MmioTransport>::new(transport)
            .expect("failed to create input driver");
        Self { input }
    }
}
impl Deref for VirtioInputWrapper {
    type Target = VirtIOInput<HalImpl, MmioTransport>;
    fn deref(&self) -> &Self::Target {
        &self.input
    }
}

impl DerefMut for VirtioInputWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.input
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
