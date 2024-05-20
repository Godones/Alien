use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;

use super::AlienResult;
use crate::{Basic, DeviceBase};

#[proxy(BufInputDomainProxy, String)]
pub trait BufInputDomain: DeviceBase + Basic + DowncastSync {
    fn init(&self, input_domain_name: &str) -> AlienResult<()>;
    /// Read an input event from the input device
    fn event_block(&self) -> AlienResult<u64>;
    fn event_nonblock(&self) -> AlienResult<Option<u64>>;
    fn have_event(&self) -> AlienResult<bool>;
}

impl_downcast!(sync BufInputDomain);
