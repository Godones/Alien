use constants::AlienResult;
use gproxy::proxy;

use crate::{Basic, DeviceBase, DeviceInfo};

#[proxy(InputDomainProxy)]
pub trait InputDomain: DeviceBase + Basic {
    fn init(&self, device_info: &DeviceInfo) -> AlienResult<()>;
    /// Read an input event from the input device
    fn event_nonblock(&self) -> AlienResult<Option<u64>>;
    // fn event_block(&self) -> AlienResult<u64>;
    // fn have_event(&self) -> AlienResult<bool>;
}
