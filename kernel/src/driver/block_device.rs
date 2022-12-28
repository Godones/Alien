use crate::driver::hal::HalImpl;
use alloc::sync::Arc;
use fat32::BlockDevice;
use lazy_static::lazy_static;
use spin::Mutex;
use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport::Transport;

pub struct QemuBlockDevice<T: Transport> {
    device: Mutex<VirtIOBlk<HalImpl, T>>,
}
impl<T: Transport> QemuBlockDevice<T> {
    pub fn new(device: VirtIOBlk<HalImpl, T>) -> Self {
        Self {
            device: Mutex::new(device),
        }
    }
}
unsafe impl<T: Transport> Send for QemuBlockDevice<T> {}
unsafe impl<T: Transport> Sync for QemuBlockDevice<T> {}

lazy_static! {
    pub static ref QEMU_BLOCK_DEVICE: Mutex<Option<Arc<dyn BlockDevice>>> = Mutex::new(None);
}

impl<T: Transport> BlockDevice for QemuBlockDevice<T> {
    fn read(&self, block: usize, buf: &mut [u8]) -> Result<usize, ()> {
        self.device.lock().read_block(block, buf).unwrap();
        Ok(buf.len())
    }
    fn write(&self, block: usize, buf: &[u8]) -> Result<usize, ()> {
        self.device.lock().write_block(block, buf).unwrap();
        Ok(buf.len())
    }
    fn flush(&self) -> Result<(), ()> {
        Ok(())
    }
}
