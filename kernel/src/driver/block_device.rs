use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::cmp::min;
use core::fmt::{Debug, Formatter};
use core::num::NonZeroUsize;

use lazy_static::lazy_static;
use lru::LruCache;
use rvfs::info::VfsError;
use rvfs::superblock::Device;
use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport::mmio::MmioTransport;

use kernel_sync::Mutex;

use crate::config::BLOCK_CACHE_FRAMES;
use crate::config::FRAME_SIZE;
use crate::driver::hal::HalImpl;
use crate::memory::{frame_alloc, FrameTracker};

const PAGE_CACHE_SIZE: usize = FRAME_SIZE;

pub struct GenericBlockDevice {
    pub device: Mutex<Box<dyn LowBlockDriver>>,
    cache: Mutex<LruCache<usize, FrameTracker>>,
    dirty: Mutex<Vec<usize>>,
}

impl GenericBlockDevice {
    pub fn new(device: Box<dyn LowBlockDriver>) -> Self {
        Self {
            device: Mutex::new(device),
            cache: Mutex::new(LruCache::new(NonZeroUsize::new(BLOCK_CACHE_FRAMES).unwrap())), // 4MB cache
            dirty: Mutex::new(Vec::new()),
        }
    }
}

unsafe impl Send for GenericBlockDevice {}

unsafe impl Sync for GenericBlockDevice {}

lazy_static! {
    pub static ref QEMU_BLOCK_DEVICE: Mutex<Vec<Arc<dyn Device>>> = Mutex::new(Vec::new());
}

impl Debug for GenericBlockDevice {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("QemuBlockDevice").finish()
    }
}

impl Device for GenericBlockDevice {
    fn read(&self, buf: &mut [u8], offset: usize) -> Result<usize, VfsError> {
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
    fn write(&self, buf: &[u8], offset: usize) -> Result<usize, VfsError> {
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
        self.device.lock().capacity() as usize * 512
    }
    fn flush(&self) {
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
    }
}

pub trait LowBlockDriver {
    fn read_block(&mut self, block_id: usize, buf: &mut [u8]) -> Result<(), VfsError>;
    fn write_block(&mut self, block_id: usize, buf: &[u8]) -> Result<(), VfsError>;
    fn capacity(&self) -> usize;
    fn flush(&mut self) {}
}

impl LowBlockDriver for VirtIOBlk<HalImpl, MmioTransport> {
    fn read_block(&mut self, block_id: usize, buf: &mut [u8]) -> Result<(), VfsError> {
        self.read_block(block_id, buf).map_err(|_| VfsError::DiskFsError("read block error".to_string()))
    }
    fn write_block(&mut self, block_id: usize, buf: &[u8]) -> Result<(), VfsError> {
        self.write_block(block_id, buf).map_err(|_| VfsError::DiskFsError("write block error".to_string()))
    }

    fn capacity(&self) -> usize {
        self.capacity() as usize
    }
}

struct MemoryFat32Img {
    data: &'static mut [u8],
}

impl LowBlockDriver for MemoryFat32Img {
    fn read_block(&mut self, block_id: usize, buf: &mut [u8]) -> Result<(), VfsError> {
        let start = block_id * 512;
        let end = start + 512;
        buf.copy_from_slice(&self.data[start..end]);
        Ok(())
    }
    fn write_block(&mut self, block_id: usize, buf: &[u8]) -> Result<(), VfsError> {
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
        Self {
            data,
        }
    }
}

#[cfg(any(feature = "vf2", feature = "cv1811h"))]
pub fn init_fake_disk() {
    use crate::board::{img_end, img_start};
    let data = unsafe { core::slice::from_raw_parts_mut(img_start as *mut u8, img_end as usize - img_start as usize) };
    let device = GenericBlockDevice::new(Box::new(MemoryFat32Img::new(data)));
    QEMU_BLOCK_DEVICE.lock().push(Arc::new(device));
    println!("init fake disk success");
}