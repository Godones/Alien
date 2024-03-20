#![no_std]
#![deny(unsafe_code)]
extern crate alloc;
use alloc::sync::Arc;
use alloc::vec::Vec;
use basic::frame::{FrameTracker, FRAME_SIZE};
use constants::AlienResult;
use core::cmp::min;
use core::fmt::Debug;
use core::num::NonZeroUsize;
use core::ops::Deref;
use interface::{
    Basic, CacheBlkDeviceDomain, DeviceBase, DomainType, ShadowBlockDomain,
};
use ksync::Mutex;
use log::info;
use lru::LruCache;
use rref::{RRef, RRefVec};
use spin::Once;

static BLK: Once<Arc<dyn ShadowBlockDomain>> = Once::new();

pub struct GenericBlockDevice {
    cache: Mutex<LruCache<usize, FrameTracker>>,
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

        let mut cache_lock = self.cache.lock();
        let len = buf.len();
        let mut count = 0;

        while count < len {
            if !cache_lock.contains(&page_id) {
                let device = BLK.get().unwrap();
                // todo!(change interface)
                let mut tmp_buf = RRef::new([0u8; 512]); // in shared heap
                let mut cache = FrameTracker::new(1);
                let start_block = page_id * FRAME_SIZE / 512;
                let end_block = start_block + FRAME_SIZE / 512;
                for i in start_block..end_block {
                    let target_buf =
                        &mut cache[(i - start_block) * 512..(i - start_block + 1) * 512];
                    tmp_buf = device.read_block(i as u32, tmp_buf).unwrap();
                    target_buf.copy_from_slice(tmp_buf.deref());
                }
                let old_cache = cache_lock.push(page_id, cache);
                if let Some((id, old_cache)) = old_cache {
                    let start_block = id * FRAME_SIZE / 512;
                    let end_block = start_block + FRAME_SIZE / 512;
                    for i in start_block..end_block {
                        let target_buf =
                            &old_cache[(i - start_block) * 512..(i - start_block + 1) * 512];
                        tmp_buf.copy_from_slice(target_buf);
                        device.write_block(i as u32, &tmp_buf).unwrap();
                        self.dirty.lock().retain(|&x| x != id);
                    }
                }
            }
            let cache = cache_lock.get(&page_id).unwrap();
            let copy_len = min(FRAME_SIZE - offset, len - count);
            buf.as_mut_slice()[count..count + copy_len]
                .copy_from_slice(&cache[offset..offset + copy_len]);
            count += copy_len;
            offset = 0;
            page_id += 1;
        }
        Ok(buf)
    }

    fn write(&self, offset: u64, buf: &RRefVec<u8>) -> AlienResult<usize> {
        let mut page_id = offset as usize / FRAME_SIZE;
        let mut offset = offset as usize % FRAME_SIZE;

        let mut cache_lock = self.cache.lock();
        let len = buf.len();
        let mut count = 0;
        while count < len {
            if !cache_lock.contains(&page_id) {
                let device = BLK.get().unwrap();
                // todo!(change interface)
                let mut cache = FrameTracker::new(1);
                let mut tmp_buf = RRef::new([0u8; 512]); // in shared heap
                let start_block = page_id * FRAME_SIZE / 512;
                let end_block = start_block + FRAME_SIZE / 512;
                for i in start_block..end_block {
                    let target_buf =
                        &mut cache[(i - start_block) * 512..(i - start_block + 1) * 512];
                    tmp_buf = device.read_block(i as u32, tmp_buf).unwrap();
                    target_buf.copy_from_slice(tmp_buf.deref());
                }
                let old_cache = cache_lock.push(page_id, cache);
                if let Some((id, old_cache)) = old_cache {
                    let start_block = id * FRAME_SIZE / 512;
                    let end_block = start_block + FRAME_SIZE / 512;
                    for i in start_block..end_block {
                        let target_buf =
                            &old_cache[(i - start_block) * 512..(i - start_block + 1) * 512];
                        tmp_buf.copy_from_slice(target_buf);
                        device.write_block(i as u32, &tmp_buf).unwrap();
                        self.dirty.lock().retain(|&x| x != id);
                    }
                }
            }
            let cache = cache_lock.get_mut(&page_id).unwrap();
            if cache.as_ptr() as usize == 0x9000_0000 {
                panic!("cache is null");
            }
            // self.dirty.lock().push(page_id);
            let copy_len = min(FRAME_SIZE - offset, len - count);
            cache[offset..offset + copy_len]
                .copy_from_slice(&buf.as_slice()[count..count + copy_len]);
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

pub const MAX_BLOCK_CACHE_FRAMES: usize = 1024 * 4 * 4;

pub fn main() -> Arc<dyn CacheBlkDeviceDomain> {
    Arc::new(GenericBlockDevice::new(MAX_BLOCK_CACHE_FRAMES))
}
