use alloc::{format, sync::Arc};
use core::{cmp::min, ops::Deref};

use constants::{
    io::{RtcTime, TeletypeCommand},
    DeviceId,
};
use interface::{RtcDomain, TaskDomain};
use rref::RRef;
use vfscore::{
    error::VfsError,
    file::VfsFile,
    inode::{InodeAttr, VfsInode},
    superblock::VfsSuperBlock,
    utils::{VfsFileStat, VfsNodeType},
    VfsResult,
};

pub struct RTCDevice {
    device_id: DeviceId,
    device: Arc<dyn RtcDomain>,
    task_domain: Arc<dyn TaskDomain>,
}

impl RTCDevice {
    pub fn new(device_id: DeviceId, device: Arc<dyn RtcDomain>, task: Arc<dyn TaskDomain>) -> Self {
        Self {
            device_id,
            device,
            task_domain: task,
        }
    }
}

impl VfsFile for RTCDevice {
    fn read_at(&self, _offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let mut time = RRef::new(RtcTime::default());
        time = self.device.read_time(time).unwrap();
        let str = format!("{:?}", time.deref());
        let bytes = str.as_bytes();
        let min_len = min(buf.len(), bytes.len());
        buf[..min_len].copy_from_slice(&bytes[..min_len]);
        Ok(min_len)
    }
    fn write_at(&self, _offset: u64, _buf: &[u8]) -> VfsResult<usize> {
        todo!()
    }
    fn ioctl(&self, cmd: u32, arg: usize) -> VfsResult<usize> {
        let cmd = TeletypeCommand::try_from(cmd).map_err(|_| VfsError::Invalid)?;
        match cmd {
            TeletypeCommand::RTC_RD_TIME => {
                let mut time = RRef::new(RtcTime::default());
                time = self.device.read_time(time).unwrap();
                let size = core::mem::size_of::<RtcTime>();
                self.task_domain
                    .copy_to_user(time.deref() as *const RtcTime as _, arg as *mut u8, size)
                    .unwrap();
            }
            _ => return Err(VfsError::Invalid),
        }
        Ok(0)
    }
    fn flush(&self) -> VfsResult<()> {
        Ok(())
    }
    fn fsync(&self) -> VfsResult<()> {
        Ok(())
    }
}

impl VfsInode for RTCDevice {
    fn get_super_block(&self) -> VfsResult<Arc<dyn VfsSuperBlock>> {
        Err(VfsError::NoSys)
    }

    fn set_attr(&self, _attr: InodeAttr) -> VfsResult<()> {
        Ok(())
    }

    fn get_attr(&self) -> VfsResult<VfsFileStat> {
        Ok(VfsFileStat {
            st_rdev: self.device_id.id(),
            ..Default::default()
        })
    }

    fn inode_type(&self) -> VfsNodeType {
        VfsNodeType::CharDevice
    }
}
