use crate::DeviceBase;
use constants::AlienResult;

pub trait InputDomain: DeviceBase {
    fn init(&self) -> AlienResult<()>;
    /// Read an input event from the input device
    fn event(&self) -> AlienResult<Option<u64>>;
}
