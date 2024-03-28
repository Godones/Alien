use constants::AlienResult;

use crate::DeviceBase;

pub trait BufInputDomain: DeviceBase {
    fn init(&self, input_domain_name: &str) -> AlienResult<()>;
    /// Read an input event from the input device
    fn event_block(&self) -> AlienResult<u64>;
    fn event_nonblock(&self) -> AlienResult<Option<u64>>;
    fn have_event(&self) -> AlienResult<bool>;
}
