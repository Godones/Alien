use constants::AlienResult;

use crate::{DeviceBase, DeviceInfo};

pub trait InputDomain: DeviceBase {
    fn init(&self, device_info: &DeviceInfo) -> AlienResult<()>;
    /// Read an input event from the input device
    fn event_nonblock(&self) -> AlienResult<Option<u64>>;
    // fn event_block(&self) -> AlienResult<u64>;
    // fn have_event(&self) -> AlienResult<bool>;
}
