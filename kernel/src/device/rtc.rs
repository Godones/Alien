use alloc::format;
use alloc::sync::Arc;
use constants::io::TeletypeCommand;
use core::cmp::min;
use rtc::RtcTime;

use crate::fs::dev::DeviceId;
use spin::Once;
use vfscore::error::VfsError;
use vfscore::file::VfsFile;
use vfscore::inode::{InodeAttr, VfsInode};
use vfscore::superblock::VfsSuperBlock;
use vfscore::utils::{VfsFileStat, VfsNodeType};
use vfscore::VfsResult;

use crate::interrupt::DeviceBase;
use crate::task::current_task;

pub static RTC_DEVICE: Once<Arc<dyn RtcDevice>> = Once::new();

pub fn get_rtc_time() -> Option<RtcTime> {
    RTC_DEVICE.get().map(|rtc| rtc.read_time_fmt())
}

pub fn init_rtc(rtc: Arc<dyn RtcDevice>) {
    RTC_DEVICE.call_once(|| rtc);
}

pub trait RtcDevice: Send + Sync + DeviceBase + rtc::LowRtcDevice + rtc::LowRtcDeviceExt {}

pub struct RTCDevice {
    device_id: DeviceId,
    device: Arc<dyn RtcDevice>,
}

impl RTCDevice {
    pub fn new(device_id: DeviceId, device: Arc<dyn RtcDevice>) -> Self {
        Self { device_id, device }
    }
    pub fn device_id(&self) -> DeviceId {
        self.device_id
    }
}

impl VfsFile for RTCDevice {
    fn read_at(&self, _offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let time = self.device.read_time_fmt();
        let str = format!("{:?}", time);
        let bytes = str.as_bytes();
        let min_len = min(buf.len(), bytes.len());
        buf[..min_len].copy_from_slice(&bytes[..min_len]);
        Ok(min_len)
    }
    fn write_at(&self, _offset: u64, _buf: &[u8]) -> VfsResult<usize> {
        todo!()
    }
    fn ioctl(&self, cmd: u32, arg: usize) -> VfsResult<usize> {
        let task = current_task().unwrap();
        let mut task_inner = task.access_inner();
        let cmd = TeletypeCommand::try_from(cmd).map_err(|_| VfsError::Invalid)?;
        match cmd {
            TeletypeCommand::RTC_RD_TIME => {
                let time = self.device.read_time_fmt();
                let c_rtc_time = constants::io::RtcTime {
                    sec: time.second as u32,
                    min: time.minute as u32,
                    hour: time.hour as u32,
                    mday: time.day as u32,
                    mon: time.month as u32,
                    year: time.year,
                    wday: 0,
                    yday: 0,
                    isdst: 0,
                };
                task_inner.copy_to_user(&c_rtc_time, arg as *mut constants::io::RtcTime);
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

#[allow(dead_code)]
fn example() {
    // let rtc = RTC_DEVICE.get().unwrap();
    // let time = rtc.read_time();
    // let alarm = rtc.read_alarm();
    // println!("time: {:#x}, alarm: {:#x}", time, alarm);
    // rtc.clear_irq();
    // rtc.enable_irq();
    // println!("set alarm");
    // rtc.set_alarm(time + 1_000_000_000 * 5); // wait 5s
    // let alarm = rtc.read_alarm_fmt();
    // let status = rtc.alarm_status();
    // let is_enable = rtc.is_irq_enabled();
    // println!(
    //     "At {:?}, rtc will interrupt, status: {} enable: {}",
    //     alarm, status, is_enable
    // );
    // loop {
    //     let time = rtc.read_time();
    //     let alarm = rtc.read_alarm();
    //     if time == alarm {
    //         let status = rtc.alarm_status();
    //         let enable = rtc.is_irq_enabled();
    //         println!("time == alarm, status: {}, enable: {}", status, enable);
    //     }
    // }
}
