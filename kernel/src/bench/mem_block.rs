use core::{cmp::min, ops::Range};

use basic::io::SafeIORegion;
use corelib::{AlienError, AlienResult};
use interface::{define_unwind_for_BlkDeviceDomain, Basic, BlkDeviceDomain, DeviceBase};
use ksync::Mutex;
use rref::RRefVec;

#[derive(Debug)]
pub struct MemoryImg {
    data: Mutex<SafeIORegion>,
}

impl Default for MemoryImg {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryImg {
    pub fn new() -> Self {
        MemoryImg {
            data: Mutex::new(SafeIORegion::from(0..0)),
        }
    }

    pub fn read_blocks(&self, block: u64, data: &mut [u8]) -> AlienResult<usize> {
        if data.len() % 512 != 0 {
            return Err(AlienError::EINVAL);
        }
        let start = block as usize * 512;
        let end = start + data.len();
        let datalock = self.data.lock();
        let io_region = datalock.as_bytes();
        let copy_start = min(io_region.len(), start);
        let copy_end = min(io_region.len(), end);
        data[..copy_end - copy_start].copy_from_slice(&io_region[copy_start..copy_end]);
        Ok(copy_end - copy_start)
    }

    pub fn write_blocks(&self, block: u64, data: &[u8]) -> AlienResult<usize> {
        if data.len() % 512 != 0 {
            return Err(AlienError::EINVAL);
        }
        let start = block as usize * 512;
        let end = start + data.len();
        // let io_region = self.data.lock().as_mut_bytes();

        let mut data_lock = self.data.lock();
        let io_region = data_lock.as_mut_bytes();

        let copy_start = min(io_region.len(), start);
        let copy_end = min(io_region.len(), end);
        io_region[copy_start..copy_end].copy_from_slice(&data[..copy_end - copy_start]);
        Ok(copy_end - copy_start)
    }
}

impl DeviceBase for MemoryImg {
    fn handle_irq(&self) -> AlienResult<()> {
        todo!()
    }
}

impl Basic for MemoryImg {
    fn domain_id(&self) -> u64 {
        rref::domain_id()
    }
}

impl BlkDeviceDomain for MemoryImg {
    fn init(&self, device_info: &Range<usize>) -> AlienResult<()> {
        let _region = device_info;
        let io_region = SafeIORegion::from(device_info.clone());
        *self.data.lock() = io_region;
        Ok(())
    }
    fn read_block(&self, block: u32, mut data: RRefVec<u8>) -> AlienResult<RRefVec<u8>> {
        self.read_blocks(block as _, data.as_mut_slice())?;
        Ok(data)
    }
    fn write_block(&self, block: u32, data: &RRefVec<u8>) -> AlienResult<usize> {
        self.write_blocks(block as _, data.as_slice())
    }
    fn get_capacity(&self) -> AlienResult<u64> {
        Ok(self.data.lock().size() as u64)
    }
    fn flush(&self) -> AlienResult<()> {
        Ok(())
    }
}

define_unwind_for_BlkDeviceDomain!(MemoryImg);

pub type UnwindMemoryImg = UnwindWrap;
