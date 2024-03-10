#![no_std]

use constants::io::RtcTime;
use constants::AlienResult;
use core::any::Any;

pub trait DeviceBase: Sync + Send {
    fn handle_irq(&self);
}

pub trait BlockDevice: DeviceBase {
    fn read(&self, buf: &mut [u8], offset: usize) -> AlienResult<usize>;
    fn write(&self, buf: &[u8], offset: usize) -> AlienResult<usize>;
    fn size(&self) -> usize;
    fn flush(&self) -> AlienResult<()>;
}
pub trait LowBlockDevice {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) -> AlienResult<()>;
    fn write_block(&self, block_id: usize, buf: &[u8]) -> AlienResult<()>;
    fn capacity(&self) -> usize;
    fn read_block_async(&self, block_id: usize, buf: &mut [u8]) -> AlienResult<()>;
    fn write_block_async(&self, block_id: usize, buf: &[u8]) -> AlienResult<()>;
    fn handle_irq(&self);
    fn flush(&self) {}
}

pub trait GpuDevice: Any + DeviceBase {
    fn update_cursor(&self);
    fn get_framebuffer(&self) -> &mut [u8];
    fn flush(&self);
    fn resolution(&self) -> (u32, u32);
}

pub trait InputDevice: DeviceBase {
    fn is_empty(&self) -> bool;
    fn read_event_async(&self) -> u64;
    fn read_event_without_block(&self) -> Option<u64>;
}

pub trait RtcDevice: DeviceBase {
    fn read_time(&self) -> RtcTime;
}

pub trait UartDevice: DeviceBase {
    fn put(&self, c: u8);
    fn get(&self) -> Option<u8>;
    fn put_bytes(&self, bytes: &[u8]);
    fn have_data_to_get(&self) -> bool;
    fn have_space_to_put(&self) -> bool;
}

pub trait NetDevice: DeviceBase {}
