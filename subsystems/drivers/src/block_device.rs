use alloc::{boxed::Box, collections::BTreeMap, sync::Arc, vec::Vec};
use core::{
    cmp::min,
    fmt::{Debug, Formatter},
    num::NonZeroUsize,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use config::FRAME_SIZE;
use constants::{AlienResult, LinuxErrno};
use device_interface::{BlockDevice, DeviceBase, LowBlockDevice};
use ksync::{Mutex, RwLock};
use lru::LruCache;
use mem::{alloc_frames, free_frames};
use platform::config::{BLOCK_CACHE_FRAMES, CLOCK_FREQ};
use shim::KTask;
use timer::read_timer;
use virtio_drivers::{
    device::blk::VirtIOBlk,
    transport::mmio::{MmioTransport, VirtIOHeader},
};
pub use visionfive2_sd::Vf2SdDriver;
use visionfive2_sd::{SDIo, SleepOps};

use crate::hal::HalImpl;
const PAGE_CACHE_SIZE: usize = FRAME_SIZE;

pub struct GenericBlockDevice {
    device: Box<dyn LowBlockDevice>,
    cache: Mutex<LruCache<usize, FrameTracker>>,
    dirty: Mutex<Vec<usize>>,
}

#[derive(Debug)]
struct FrameTracker {
    ptr: usize,
}

impl FrameTracker {
    pub fn new(ptr: usize) -> Self {
        Self { ptr }
    }
}

impl Deref for FrameTracker {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { core::slice::from_raw_parts(self.ptr as *const u8, FRAME_SIZE) }
    }
}

impl DerefMut for FrameTracker {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { core::slice::from_raw_parts_mut(self.ptr as *mut u8, FRAME_SIZE) }
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        free_frames(self.ptr as *mut u8, 1);
    }
}

unsafe impl Send for GenericBlockDevice {}

unsafe impl Sync for GenericBlockDevice {}

impl GenericBlockDevice {
    pub fn new(device: Box<dyn LowBlockDevice>) -> Self {
        Self {
            device,
            cache: Mutex::new(LruCache::new(
                NonZeroUsize::new(BLOCK_CACHE_FRAMES).unwrap(),
            )),
            dirty: Mutex::new(Vec::new()),
        }
    }
}

impl DeviceBase for GenericBlockDevice {
    fn handle_irq(&self) {
        self.device.handle_irq();
    }
}

impl Debug for GenericBlockDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("QemuBlockDevice").finish()
    }
}

impl BlockDevice for GenericBlockDevice {
    fn read(&self, buf: &mut [u8], offset: usize) -> AlienResult<usize> {
        let mut page_id = offset / PAGE_CACHE_SIZE;
        let mut offset = offset % PAGE_CACHE_SIZE;

        let mut cache_lock = self.cache.lock();
        let len = buf.len();
        let mut count = 0;

        while count < len {
            if !cache_lock.contains(&page_id) {
                let device = &self.device;
                let cache = alloc_frames(1);
                let mut cache = FrameTracker::new(cache as usize);
                let start_block = page_id * PAGE_CACHE_SIZE / 512;
                let end_block = start_block + PAGE_CACHE_SIZE / 512;
                for i in start_block..end_block {
                    let target_buf =
                        &mut cache[(i - start_block) * 512..(i - start_block + 1) * 512];
                    device.read_block(i, target_buf).unwrap();
                }
                let old_cache = cache_lock.push(page_id, cache);
                if let Some((id, old_cache)) = old_cache {
                    let start_block = id * PAGE_CACHE_SIZE / 512;
                    let end_block = start_block + PAGE_CACHE_SIZE / 512;
                    for i in start_block..end_block {
                        let target_buf =
                            &old_cache[(i - start_block) * 512..(i - start_block + 1) * 512];
                        device.write_block(i, target_buf).unwrap();
                        self.dirty.lock().retain(|&x| x != id);
                    }
                }
            }
            let cache = cache_lock.get(&page_id).unwrap();
            let copy_len = min(PAGE_CACHE_SIZE - offset, len - count);
            buf[count..count + copy_len].copy_from_slice(&cache[offset..offset + copy_len]);
            count += copy_len;
            offset = 0;
            page_id += 1;
        }
        Ok(buf.len())
    }
    fn write(&self, buf: &[u8], offset: usize) -> AlienResult<usize> {
        let mut page_id = offset / PAGE_CACHE_SIZE;
        let mut offset = offset % PAGE_CACHE_SIZE;
        let mut cache_lock = self.cache.lock();
        let len = buf.len();
        let mut count = 0;
        while count < len {
            if !cache_lock.contains(&page_id) {
                let device = &self.device;
                let cache = alloc_frames(1);
                let mut cache = FrameTracker::new(cache as usize);
                let start_block = page_id * PAGE_CACHE_SIZE / 512;
                let end_block = start_block + PAGE_CACHE_SIZE / 512;
                for i in start_block..end_block {
                    let target_buf =
                        &mut cache[(i - start_block) * 512..(i - start_block + 1) * 512];
                    device.read_block(i, target_buf).unwrap();
                }
                let old_cache = cache_lock.push(page_id, cache);
                if let Some((id, old_cache)) = old_cache {
                    let start_block = id * PAGE_CACHE_SIZE / 512;
                    let end_block = start_block + PAGE_CACHE_SIZE / 512;
                    for i in start_block..end_block {
                        let target_buf =
                            &old_cache[(i - start_block) * 512..(i - start_block + 1) * 512];
                        device.write_block(i, target_buf).unwrap();
                        self.dirty.lock().retain(|&x| x != id);
                    }
                }
            }
            let cache = cache_lock.get_mut(&page_id).unwrap();
            let copy_len = min(PAGE_CACHE_SIZE - offset, len - count);
            cache[offset..offset + copy_len].copy_from_slice(&buf[count..count + copy_len]);
            count += copy_len;
            offset = (offset + copy_len) % PAGE_CACHE_SIZE;
            page_id += 1;
        }
        Ok(buf.len())
    }
    fn size(&self) -> usize {
        self.device.capacity() * 512
    }
    fn flush(&self) -> AlienResult<()> {
        // let mut device = self.device.lock();
        // let mut lru = self.cache.lock();
        // self.dirty.lock().iter().for_each(|id|{
        //     let start = id * PAGE_CACHE_SIZE;
        //     let start_block = start / 512;
        //     let end_block = (start + PAGE_CACHE_SIZE) / 512;
        //     let cache = lru.get(id).unwrap();
        //     for i in start_block..end_block {
        //         let target_buf = &cache[(i - start_block) * 512..(i - start_block + 1) * 512];
        //         device.write_block(i, target_buf).unwrap();
        //     }
        // });
        // self.dirty.lock().clear();
        Ok(())
    }
}

pub struct VirtIOBlkWrapper {
    device: Mutex<VirtIOBlk<HalImpl, MmioTransport>>,
    wait_queue: Mutex<BTreeMap<u16, Arc<dyn KTask>>>,
}

impl VirtIOBlkWrapper {
    pub fn new(addr: usize) -> Self {
        let header = NonNull::new(addr as *mut VirtIOHeader).unwrap();
        let transport = unsafe { MmioTransport::new(header) }.unwrap();
        let blk = VirtIOBlk::<HalImpl, MmioTransport>::new(transport)
            .expect("failed to create blk driver");
        Self {
            device: Mutex::new(blk),
            wait_queue: Mutex::new(BTreeMap::new()),
        }
    }

    pub fn from_mmio(mmio_transport: MmioTransport) -> Self {
        let blk = VirtIOBlk::<HalImpl, MmioTransport>::new(mmio_transport)
            .expect("failed to create blk driver");
        Self {
            device: Mutex::new(blk),
            wait_queue: Mutex::new(BTreeMap::new()),
        }
    }
}

impl LowBlockDevice for VirtIOBlkWrapper {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) -> AlienResult<()> {
        let res = self
            .device
            .lock()
            .read_blocks(block_id, buf)
            .map_err(|_| LinuxErrno::EIO.into());
        res
    }
    fn write_block(&self, block_id: usize, buf: &[u8]) -> AlienResult<()> {
        self.device
            .lock()
            .write_blocks(block_id, buf)
            .map_err(|_| LinuxErrno::EIO.into())
    }

    fn capacity(&self) -> usize {
        self.device.lock().capacity() as usize
    }

    fn read_block_async(&self, block_id: usize, buf: &mut [u8]) -> AlienResult<()> {
        return self.read_block(block_id, buf);
        // let task = shim::take_current_task();
        // if task.is_none() {
        //     return self.read_block(block_id, buf);
        // }
        // let task = task.unwrap();
        //
        // use virtio_drivers::device::blk::{BlkReq,BlkResp,RespStatus};
        //
        // let mut resp = BlkResp::default();
        // let mut req = BlkReq::default();
        // let mut device = self.device.lock();
        // let token = unsafe { device.read_block_nb(block_id, &mut req, buf, &mut resp) };
        // match token {
        //     Ok(token) => {
        //         task.to_wait();
        //         self.wait_queue.lock().insert(token, task.clone());
        //         // platform::println!("insert token:{}, intr: {}", token, arch::is_interrupt_enable());
        //         drop(device);
        //         shim::schedule_now(task);
        //         unsafe {
        //             self.device
        //                 .lock()
        //                 .complete_read_block(token, &req, buf, &mut resp)
        //                 .unwrap();
        //         }
        //         assert_eq!(
        //             resp.status(),
        //             RespStatus::OK,
        //             "Error {:?} reading block.",
        //             resp.status()
        //         );
        //         Ok(())
        //     }
        //     Err(virtio_drivers::Error::QueueFull) => self.read_block(block_id, buf),
        //     Err(_) => Err(LinuxErrno::EIO.into()),
        // }
    }

    fn write_block_async(&self, block_id: usize, buf: &[u8]) -> AlienResult<()> {
        return self.write_block(block_id, buf);
        // let task = shim::take_current_task();
        // if task.is_none() {
        //     return self.write_block(block_id, buf);
        // }
        // let task = task.unwrap();
        // let mut resp = BlkResp::default();
        // let mut req = BlkReq::default();
        //
        // let mut device = self.device.lock();
        //
        // let token = unsafe { device.write_block_nb(block_id, &mut req, buf, &mut resp) };
        // match token {
        //     Ok(token) => {
        //         task.to_wait();
        //         self.wait_queue.lock().insert(token, task.clone());
        //         drop(device);
        //         shim::schedule_now(task);
        //         unsafe {
        //             self.device
        //                 .lock()
        //                 .complete_write_block(token, &req, buf, &mut resp)
        //                 .unwrap();
        //         }
        //         assert_eq!(
        //             resp.status(),
        //             RespStatus::OK,
        //             "Error {:?} writing block.",
        //             resp.status()
        //         );
        //         Ok(())
        //     }
        //     Err(virtio_drivers::Error::QueueFull) => self.write_block(block_id, buf),
        //     Err(_) => Err(LinuxErrno::EIO.into()),
        // }
    }

    fn handle_irq(&self) {
        let mut device = self.device.lock();
        device.ack_interrupt();
        if let Some(token) = device.peek_used() {
            let mut wait_queue = self.wait_queue.lock();
            let task = wait_queue.remove(&token).unwrap();
            task.to_wakeup();
            shim::put_task(task);
        }
    }
}

pub struct MemoryFat32Img {
    data: RwLock<&'static mut [u8]>,
}

impl LowBlockDevice for MemoryFat32Img {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) -> AlienResult<()> {
        let start = block_id * 512;
        let end = start + 512;
        buf.copy_from_slice(&self.data.read()[start..end]);
        Ok(())
    }
    fn write_block(&self, block_id: usize, buf: &[u8]) -> AlienResult<()> {
        let start = block_id * 512;
        let end = start + 512;
        self.data.write()[start..end].copy_from_slice(buf);
        Ok(())
    }
    fn capacity(&self) -> usize {
        self.data.read().len() / 512
    }
    fn read_block_async(&self, block_id: usize, buf: &mut [u8]) -> AlienResult<()> {
        self.read_block(block_id, buf)
    }

    fn write_block_async(&self, block_id: usize, buf: &[u8]) -> AlienResult<()> {
        self.write_block(block_id, buf)
    }
    fn handle_irq(&self) {}
}

impl MemoryFat32Img {
    pub fn new(data: &'static mut [u8]) -> Self {
        Self {
            data: RwLock::new(data),
        }
    }
}

pub struct VF2SDDriver {
    driver: Mutex<Vf2SdDriver<SdIoImpl, SleepOpsImpl>>,
}

impl VF2SDDriver {
    pub fn new() -> Self {
        Self {
            driver: Mutex::new(Vf2SdDriver::new(SdIoImpl)),
        }
    }
    pub fn init(&mut self) {
        self.driver.lock().init();
    }
}

impl LowBlockDevice for VF2SDDriver {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) -> AlienResult<()> {
        self.driver.lock().read_block(block_id, buf);
        Ok(())
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) -> AlienResult<()> {
        self.driver.lock().write_block(block_id, buf);
        Ok(())
    }
    fn capacity(&self) -> usize {
        // unimplemented!()
        // 32GB
        32 * 1024 * 1024 * 1024 / 512
    }
    fn read_block_async(&self, block_id: usize, buf: &mut [u8]) -> AlienResult<()> {
        self.read_block(block_id, buf)
    }

    fn write_block_async(&self, block_id: usize, buf: &[u8]) -> AlienResult<()> {
        self.write_block(block_id, buf)
    }
    fn handle_irq(&self) {
        unimplemented!()
    }
}

pub struct SdIoImpl;
pub const SDIO_BASE: usize = 0x16020000;

impl SDIo for SdIoImpl {
    fn read_reg_at(&self, offset: usize) -> u32 {
        let addr = (SDIO_BASE + offset) as *mut u32;
        unsafe { addr.read_volatile() }
    }
    fn write_reg_at(&mut self, offset: usize, val: u32) {
        let addr = (SDIO_BASE + offset) as *mut u32;
        unsafe { addr.write_volatile(val) }
    }
    fn read_data_at(&self, offset: usize) -> u64 {
        let addr = (SDIO_BASE + offset) as *mut u64;
        unsafe { addr.read_volatile() }
    }
    fn write_data_at(&mut self, offset: usize, val: u64) {
        let addr = (SDIO_BASE + offset) as *mut u64;
        unsafe { addr.write_volatile(val) }
    }
}

pub struct SleepOpsImpl;

impl SleepOps for SleepOpsImpl {
    fn sleep_ms(ms: usize) {
        sleep_ms(ms)
    }
    fn sleep_ms_until(ms: usize, f: impl FnMut() -> bool) {
        sleep_ms_until(ms, f)
    }
}

fn sleep_ms(ms: usize) {
    let start = read_timer();
    while read_timer() - start < ms * CLOCK_FREQ / 1000 {
        core::hint::spin_loop();
    }
}

fn sleep_ms_until(ms: usize, mut f: impl FnMut() -> bool) {
    let start = read_timer();
    while read_timer() - start < ms * CLOCK_FREQ / 1000 {
        if f() {
            return;
        }
        core::hint::spin_loop();
    }
}
