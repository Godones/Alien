use interface::DeviceInfo;
use virtio_drivers::device::net::VirtIONet;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};
use virtio_drivers::{Hal, BufferDirection, PhysAddr};
use basic::frame::FrameTracker;
use core::{ptr::NonNull, mem::forget};

pub const NET_QUEUE_SIZE: usize = 128;
pub const NET_BUF_LEN: usize = 4096;

pub struct VirtIoNetWrapper {
    driver: VirtIONet<HalImpl, MmioTransport, NET_QUEUE_SIZE>
}
unsafe impl Send for VirtIoNetWrapper {}
unsafe impl Sync for VirtIoNetWrapper {}

impl VirtIoNetWrapper {
    pub fn new(device_info: &DeviceInfo) -> Self {
        let virtio_net_addr = device_info.address_range.start;
        basic::println!("virtio_net_addr: {:#x?}", virtio_net_addr);

        let header = NonNull::new(virtio_net_addr as *mut VirtIOHeader).unwrap();
        let transport = unsafe { MmioTransport::new(header) }.unwrap();

        let net = 
            VirtIONet::<HalImpl, MmioTransport, NET_QUEUE_SIZE>::new(
                transport, NET_BUF_LEN
            ).expect("failed to create gpu driver");
        Self { 
            driver: net 
        }
    }
}

pub struct HalImpl;

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
