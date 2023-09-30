use alloc::vec::Vec;
use core::ptr::NonNull;

use embedded_graphics_core::pixelcolor::Rgb888;
use tinybmp::Bmp;
use virtio_drivers::device::gpu::VirtIOGpu;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};

use crate::ksync::Mutex;

use crate::device::GpuDevice;
use crate::driver::hal::HalImpl;
use crate::interrupt::DeviceBase;

pub struct VirtIOGpuWrapper {
    gpu: Mutex<VirtIOGpu<HalImpl, MmioTransport>>,
    fb: &'static [u8],
}

unsafe impl Sync for VirtIOGpuWrapper {}

unsafe impl Send for VirtIOGpuWrapper {}

static BMP_DATA: &[u8] = include_bytes!("../../../assert/mouse.bmp");

impl VirtIOGpuWrapper {
    pub fn new(addr: usize) -> Self {
        let header = NonNull::new(addr as *mut VirtIOHeader).unwrap();
        let mmio = unsafe { MmioTransport::new(header) }.unwrap();
        let mut gpu = VirtIOGpu::new(mmio).unwrap();
        unsafe {
            let fbuffer = gpu.setup_framebuffer().unwrap();
            let len = fbuffer.len();
            let ptr = fbuffer.as_mut_ptr();
            let fb = core::slice::from_raw_parts_mut(ptr, len);

            let bmp = Bmp::<Rgb888>::from_slice(BMP_DATA).unwrap();
            let raw = bmp.as_raw();
            let mut b = Vec::new();
            for i in raw.image_data().chunks(3) {
                let mut v = i.to_vec();
                b.append(&mut v);
                if i == [255, 255, 255] {
                    b.push(0x0)
                } else {
                    b.push(0xff)
                }
            }
            gpu.setup_cursor(b.as_slice(), 50, 50, 50, 50).unwrap();
            Self {
                gpu: Mutex::new(gpu),
                fb,
            }
        }
    }
}

impl DeviceBase for VirtIOGpuWrapper {
    fn hand_irq(&self) {
        todo!()
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
}
