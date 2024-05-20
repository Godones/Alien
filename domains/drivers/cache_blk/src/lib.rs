#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::{cmp::min, fmt::Debug, num::NonZeroUsize};

use basic::{config::FRAME_SIZE, sync::Mutex, AlienResult};
use interface::{Basic, CacheBlkDeviceDomain, DeviceBase, DomainType, ShadowBlockDomain};
use log::info;
use lru::LruCache;
use rref::{RRef, RRefVec};
use spin::Once;

static BLK: Once<Arc<dyn ShadowBlockDomain>> = Once::new();

struct PageCache(Vec<RRef<[u8; 512]>>);

impl PageCache {
    pub fn get(&self, index: usize) -> &RRef<[u8; 512]> {
        &self.0[index]
    }

    pub fn copy_to(&self, offset: usize, buf: &mut [u8]) {
        let mut count = 0;
        let start_slice = offset / 512;
        let mut start_offset = offset % 512;
        let mut index = start_slice;
        while count < buf.len() {
            let copy_len = min(512 - start_offset, buf.len() - count);
            buf[count..count + copy_len]
                .copy_from_slice(&self.0[index][start_offset..start_offset + copy_len]);
            count += copy_len;
            start_offset = 0;
            index += 1;
        }
    }

    pub fn copy_from(&mut self, offset: usize, buf: &[u8]) {
        let mut count = 0;
        let start_slice = offset / 512;
        let mut start_offset = offset % 512;
        let mut index = start_slice;
        while count < buf.len() {
            let copy_len = min(512 - start_offset, buf.len() - count);
            self.0[index][start_offset..start_offset + copy_len]
                .copy_from_slice(&buf[count..count + copy_len]);
            count += copy_len;
            start_offset = 0;
            index += 1;
        }
    }
}

pub struct GenericBlockDevice {
    cache: Mutex<LruCache<usize, PageCache>>,
    dirty: Mutex<Vec<usize>>,
}

impl Debug for GenericBlockDevice {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("GenericBlockDevice").finish()
    }
}

impl GenericBlockDevice {
    pub fn new(max_cache_frames: usize) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(NonZeroUsize::new(max_cache_frames).unwrap())),
            dirty: Mutex::new(Vec::new()),
        }
    }

    fn check(&self, page_id: usize) {
        let mut cache_lock = self.cache.lock();
        if !cache_lock.contains(&page_id) {
            let device = BLK.get().unwrap();
            // todo!(change interface)
            let start_block = page_id * FRAME_SIZE / 512;
            let end_block = start_block + FRAME_SIZE / 512;
            let mut cache = Vec::with_capacity(4);
            for i in start_block..end_block {
                let cache_slice = RRef::new([0u8; 512]);
                let cache_slice = device.read_block(i as u32, cache_slice).unwrap();
                cache.push(cache_slice);
            }
            let cache = PageCache(cache);
            let old_cache = cache_lock.push(page_id, cache);
            if let Some((id, old_cache)) = old_cache {
                let start_block = id * FRAME_SIZE / 512;
                let end_block = start_block + FRAME_SIZE / 512;
                for i in start_block..end_block {
                    let tmp_buf = old_cache.get(i - start_block);
                    device.write_block(i as u32, tmp_buf).unwrap();
                    self.dirty.lock().retain(|&x| x != id);
                }
            }
        }
    }
}

impl Basic for GenericBlockDevice {}

impl DeviceBase for GenericBlockDevice {
    fn handle_irq(&self) -> AlienResult<()> {
        BLK.get().unwrap().handle_irq()
    }
}

impl CacheBlkDeviceDomain for GenericBlockDevice {
    fn init(&self, blk_domain_name: &str) -> AlienResult<()> {
        let blk = basic::get_domain(blk_domain_name).unwrap();
        match blk {
            DomainType::ShadowBlockDomain(blk) => {
                info!(
                    "max_cache_frames: {}, blk size: {}MB",
                    MAX_BLOCK_CACHE_FRAMES,
                    blk.get_capacity().unwrap() / (1024 * 1024)
                );
                BLK.call_once(|| blk);
                Ok(())
            }
            _ => {
                panic!("blk domain not found");
            }
        }
    }

    fn read(&self, offset: u64, mut buf: RRefVec<u8>) -> AlienResult<RRefVec<u8>> {
        let mut page_id = offset as usize / FRAME_SIZE;
        let mut offset = offset as usize % FRAME_SIZE;
        let len = buf.len();
        let mut count = 0;
        while count < len {
            self.check(page_id);
            let mut cache_lock = self.cache.lock();
            let cache = cache_lock.get(&page_id).unwrap();
            let copy_len = min(FRAME_SIZE - offset, len - count);
            cache.copy_to(offset, &mut buf.as_mut_slice()[count..count + copy_len]);
            count += copy_len;
            offset = 0;
            page_id += 1;
        }
        Ok(buf)
    }

    fn write(&self, offset: u64, buf: &RRefVec<u8>) -> AlienResult<usize> {
        let mut page_id = offset as usize / FRAME_SIZE;
        let mut offset = offset as usize % FRAME_SIZE;
        let len = buf.len();
        let mut count = 0;
        while count < len {
            self.check(page_id);
            let mut cache_lock = self.cache.lock();
            let cache = cache_lock.get_mut(&page_id).unwrap();
            let copy_len = min(FRAME_SIZE - offset, len - count);
            cache.copy_from(offset, &buf.as_slice()[count..count + copy_len]);
            count += copy_len;
            offset = (offset + copy_len) % FRAME_SIZE;
            page_id += 1;
        }
        Ok(buf.len())
    }

    fn get_capacity(&self) -> AlienResult<u64> {
        BLK.get().unwrap().get_capacity()
    }

    fn flush(&self) -> AlienResult<()> {
        Ok(())
    }
}

pub const MAX_BLOCK_CACHE_FRAMES: usize = 1024 * 4 * 4;

pub fn main() -> Box<dyn CacheBlkDeviceDomain> {
    Box::new(GenericBlockDevice::new(MAX_BLOCK_CACHE_FRAMES))
}
