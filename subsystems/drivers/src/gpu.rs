use core::ptr::NonNull;
use device_interface::{DeviceBase, GpuDevice};
use virtio_drivers::device::gpu::VirtIOGpu;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};

use crate::hal::HalImpl;
use ksync::Mutex;

pub struct VirtIOGpuWrapper {
    gpu: Mutex<VirtIOGpu<HalImpl, MmioTransport>>,
    fb: &'static [u8],
    #[allow(unused)]
    resolution: (u32, u32),
}

unsafe impl Sync for VirtIOGpuWrapper {}

unsafe impl Send for VirtIOGpuWrapper {}

impl VirtIOGpuWrapper {
    pub fn new(addr: usize) -> Self {
        let header = NonNull::new(addr as *mut VirtIOHeader).unwrap();
        let mmio = unsafe { MmioTransport::new(header) }.unwrap();
        let mut gpu = VirtIOGpu::new(mmio).unwrap();
        let resolution = gpu.resolution().unwrap();
        unsafe {
            let fbuffer = gpu.setup_framebuffer().unwrap();
            let len = fbuffer.len();
            let ptr = fbuffer.as_mut_ptr();
            let fb = core::slice::from_raw_parts_mut(ptr, len);
            gpu.move_cursor(50, 50).unwrap();
            Self {
                gpu: Mutex::new(gpu),
                fb,
                resolution,
            }
        }
    }
}

impl DeviceBase for VirtIOGpuWrapper {
    fn hand_irq(&self) {
        self.gpu.lock().ack_interrupt();
    }
}

impl GpuDevice for VirtIOGpuWrapper {
    fn update_cursor(&self) {}
    fn get_framebuffer(&self) -> &mut [u8] {
        unsafe {
            let ptr = self.fb.as_ptr() as *const _ as *mut u8;
            core::slice::from_raw_parts_mut(ptr, self.fb.len())
        }
    }
    fn flush(&self) {
        self.gpu.lock().flush().unwrap();
    }
    fn resolution(&self) -> (u32, u32) {
        self.resolution
    }
}
