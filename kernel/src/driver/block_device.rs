use alloc::boxed::Box;
use alloc::vec::Vec;
use core::cmp::min;
use core::fmt::{Debug, Formatter};
use core::num::NonZeroUsize;
use core::ptr::NonNull;
use pconst::LinuxErrno;

use lru::LruCache;
use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport::mmio::{MmioTransport, VirtIOHeader};

use crate::error::AlienResult;
use crate::ksync::Mutex;

use crate::config::BLOCK_CACHE_FRAMES;
use crate::config::FRAME_SIZE;
use crate::device::BlockDevice;
use crate::driver::hal::HalImpl;
use crate::interrupt::DeviceBase;
use crate::memory::{frame_alloc, FrameTracker};

const PAGE_CACHE_SIZE: usize = FRAME_SIZE;

pub struct GenericBlockDevice {
    pub device: Mutex<Box<dyn LowBlockDriver>>,
    cache: Mutex<LruCache<usize, FrameTracker>>,
    dirty: Mutex<Vec<usize>>,
}

unsafe impl Send for GenericBlockDevice {}

unsafe impl Sync for GenericBlockDevice {}

impl GenericBlockDevice {
    pub fn new(device: Box<dyn LowBlockDriver>) -> Self {
        Self {
            device: Mutex::new(device),
            cache: Mutex::new(LruCache::new(
                NonZeroUsize::new(BLOCK_CACHE_FRAMES).unwrap(),
            )), // 4MB cache
            dirty: Mutex::new(Vec::new()),
        }
    }
}

impl DeviceBase for GenericBlockDevice {
    fn hand_irq(&self) {
        unimplemented!()
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
                let mut device = self.device.lock();
                let mut cache = frame_alloc().unwrap();
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
                let mut device = self.device.lock();
                let mut cache = frame_alloc().unwrap();
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
            if cache.as_ptr() as usize == 0x9000_0000 {
                panic!("cache is null");
            }
            // self.dirty.lock().push(page_id);
            let copy_len = min(PAGE_CACHE_SIZE - offset, len - count);
            cache[offset..offset + copy_len].copy_from_slice(&buf[count..count + copy_len]);
            count += copy_len;
            offset = (offset + copy_len) % PAGE_CACHE_SIZE;
            page_id += 1;
        }
        Ok(buf.len())
    }
    fn size(&self) -> usize {
        self.device.lock().capacity() * 512
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

pub trait LowBlockDriver {
    fn read_block(&mut self, block_id: usize, buf: &mut [u8]) -> AlienResult<()>;
    fn write_block(&mut self, block_id: usize, buf: &[u8]) -> AlienResult<()>;
    fn capacity(&self) -> usize;
    fn flush(&mut self) {}
}

pub struct VirtIOBlkWrapper {
    device: VirtIOBlk<HalImpl, MmioTransport>,
}

impl VirtIOBlkWrapper {
    pub fn new(addr: usize) -> Self {
        let header = NonNull::new(addr as *mut VirtIOHeader).unwrap();
        let transport = unsafe { MmioTransport::new(header) }.unwrap();
        let blk = VirtIOBlk::<HalImpl, MmioTransport>::new(transport)
            .expect("failed to create blk driver");
        let size = blk.capacity();
        println!("blk device size is {}MB", size * 512 / 1024 / 1024);
        Self { device: blk }
    }
}

impl LowBlockDriver for VirtIOBlkWrapper {
    fn read_block(&mut self, block_id: usize, buf: &mut [u8]) -> AlienResult<()> {
        let res = self
            .device
            .read_blocks(block_id, buf)
            .map_err(|_| LinuxErrno::EIO.into());
        res
    }
    fn write_block(&mut self, block_id: usize, buf: &[u8]) -> AlienResult<()> {
        self.device
            .write_blocks(block_id, buf)
            .map_err(|_| LinuxErrno::EIO.into())
    }

    fn capacity(&self) -> usize {
        self.device.capacity() as usize
    }
}

pub struct MemoryFat32Img {
    data: &'static mut [u8],
}

impl LowBlockDriver for MemoryFat32Img {
    fn read_block(&mut self, block_id: usize, buf: &mut [u8]) -> AlienResult<()> {
        let start = block_id * 512;
        let end = start + 512;
        buf.copy_from_slice(&self.data[start..end]);
        Ok(())
    }
    fn write_block(&mut self, block_id: usize, buf: &[u8]) -> AlienResult<()> {
        let start = block_id * 512;
        let end = start + 512;
        self.data[start..end].copy_from_slice(buf);
        Ok(())
    }

    fn capacity(&self) -> usize {
        self.data.len() / 512
    }
}

impl MemoryFat32Img {
    pub fn new(data: &'static mut [u8]) -> Self {
        Self { data }
    }
}
