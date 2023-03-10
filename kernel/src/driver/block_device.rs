use crate::driver::hal::HalImpl;
use alloc::sync::Arc;
use core::fmt::{Debug, Formatter};
use core::num::NonZeroUsize;
use lazy_static::lazy_static;
use lru::LruCache;
use rvfs::superblock::Device;
use spin::Mutex;
use virtio_drivers::device::blk::VirtIOBlk;
use virtio_drivers::transport::Transport;

type Cache = [u8; 512];
pub struct QemuBlockDevice<T: Transport> {
    device: Mutex<VirtIOBlk<HalImpl, T>>,
    cache: Mutex<LruCache<usize, Cache>>,
}

impl<T: Transport> QemuBlockDevice<T> {
    pub fn new(device: VirtIOBlk<HalImpl, T>) -> Self {
        Self {
            device: Mutex::new(device),
            cache: Mutex::new(LruCache::new(NonZeroUsize::new(32).unwrap())),
        }
    }
}
unsafe impl<T: Transport> Send for QemuBlockDevice<T> {}
unsafe impl<T: Transport> Sync for QemuBlockDevice<T> {}

lazy_static! {
    pub static ref QEMU_BLOCK_DEVICE: Mutex<Option<Arc<dyn Device>>> = Mutex::new(None);
}

impl<T: Transport> Debug for QemuBlockDevice<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("QemuBlockDevice").finish()
    }
}

impl<T: Transport> Device for QemuBlockDevice<T> {
    fn read(&self, buf: &mut [u8], offset: usize) -> Result<usize, ()> {
        let mut block = offset / 512;
        let mut offset = offset % 512;
        let mut cache_lock = self.cache.lock();
        let len = buf.len();
        let mut count = 0;
        while count < len {
            if !cache_lock.contains(&block) {
                let mut cache = [0u8; 512];
                let mut device = self.device.lock();
                device.read_block(block, &mut cache).unwrap();
                let old_cache = cache_lock.push(block, cache);
                if let Some((id, old_cache)) = old_cache {
                    device.write_block(id, &old_cache).unwrap();
                }
            }
            let cache = cache_lock.get(&block).unwrap();
            let mut copy_len = 512 - offset;
            if copy_len > len - count {
                copy_len = len - count;
            }
            buf[count..count + copy_len].copy_from_slice(&cache[offset..offset + copy_len]);
            count += copy_len;
            offset = 0;
            block += 1;
        }
        Ok(buf.len())
    }
    fn write(&self, buf: &[u8], offset: usize) -> Result<usize, ()> {
        let mut block = offset / 512;
        let mut offset = offset % 512;
        let mut cache_lock = self.cache.lock();
        let len = buf.len();
        let mut count = 0;
        while count < len {
            if !cache_lock.contains(&block) {
                let mut cache = [0u8; 512];
                let mut device = self.device.lock();
                device.read_block(block, &mut cache).unwrap();
                let old_cache = cache_lock.push(block, cache);
                if let Some((id, old_cache)) = old_cache {
                    device.write_block(id, &old_cache).unwrap();
                }
            }
            let cache = cache_lock.get_mut(&block).unwrap();
            let mut copy_len = 512 - offset;
            if copy_len > len - count {
                copy_len = len - count;
            }
            cache[offset..offset + copy_len].copy_from_slice(&buf[count..count + copy_len]);
            count += copy_len;
            offset = 0;
            block += 1;
        }
        Ok(buf.len())
    }
    fn size(&self) -> usize {
        self.device.lock().capacity() as usize * 512
    }
    fn flush(&self) {
        let mut device = self.device.lock();
        for (id, cache) in self.cache.lock().iter() {
            device.write_block(*id, &cache.clone()).unwrap();
        }
    }
}
