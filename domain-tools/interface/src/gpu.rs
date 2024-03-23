use crate::devices::DeviceInfo;
use crate::DeviceBase;
use constants::AlienResult;
use rref::RRefVec;
pub trait GpuDomain: DeviceBase {
    fn init(&self, device_info: &DeviceInfo) -> AlienResult<()>;
    fn flush(&self) -> AlienResult<()>;
    fn fill(&self, offset: u32, buf: &RRefVec<u8>) -> AlienResult<usize>;
}
