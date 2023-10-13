use alloc::sync::Arc;

use crate::fs::dev::DeviceId;
use spin::Once;
use vfscore::error::VfsError;
use vfscore::file::VfsFile;
use vfscore::inode::{InodeAttr, VfsInode};
use vfscore::superblock::VfsSuperBlock;
use vfscore::utils::{FileStat, PollEvents, VfsNodeType};
use vfscore::VfsResult;

use crate::interrupt::DeviceBase;

pub trait UartDevice: Send + Sync + DeviceBase {
    fn put(&self, c: u8);
    fn get(&self) -> Option<u8>;
    fn put_bytes(&self, bytes: &[u8]);
    fn have_data_to_get(&self) -> bool;
    fn have_space_to_put(&self) -> bool;
}

pub static UART_DEVICE: Once<Arc<dyn UartDevice>> = Once::new();

pub fn init_uart(uart: Arc<dyn UartDevice>) {
    UART_DEVICE.call_once(|| uart);
}

pub struct UARTDevice {
    device_id: DeviceId,
    device: Arc<dyn UartDevice>,
}
impl UARTDevice {
    pub fn new(device_id: DeviceId, device: Arc<dyn UartDevice>) -> Self {
        Self { device_id, device }
    }
    pub fn device_id(&self) -> DeviceId {
        self.device_id
    }
}

impl VfsFile for UARTDevice {
    fn read_at(&self, _offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        assert_eq!(buf.len(), 1);
        let ch = self.device.get();
        if let Some(ch) = ch {
            buf[0] = ch;
            Ok(1)
        } else {
            Ok(0)
        }
    }
    fn write_at(&self, _offset: u64, buf: &[u8]) -> VfsResult<usize> {
        self.device.put_bytes(buf);
        Ok(buf.len())
    }
    fn poll(&self, _event: PollEvents) -> VfsResult<PollEvents> {
        todo!()
    }
    fn ioctl(&self, _cmd: u32, _arg: u64) -> VfsResult<Option<u64>> {
        todo!()
    }
    fn flush(&self) -> VfsResult<()> {
        Ok(())
    }
    fn fsync(&self) -> VfsResult<()> {
        Ok(())
    }
}

impl VfsInode for UARTDevice {
    fn get_super_block(&self) -> VfsResult<Arc<dyn VfsSuperBlock>> {
        Err(VfsError::NoSys)
    }

    fn set_attr(&self, _attr: InodeAttr) -> VfsResult<()> {
        Ok(())
    }

    fn get_attr(&self) -> VfsResult<FileStat> {
        Ok(FileStat {
            st_rdev: self.device_id.id(),
            ..Default::default()
        })
    }

    fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::CharDevice
    }
}
