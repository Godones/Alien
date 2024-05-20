use core::ops::Range;

use downcast_rs::{impl_downcast, DowncastSync};
use gproxy::proxy;

use super::AlienResult;
use crate::{Basic, DeviceBase};

#[proxy(InputDomainProxy,Range<usize>)]
pub trait InputDomain: DeviceBase + Basic + DowncastSync {
    fn init(&self, device_info: Range<usize>) -> AlienResult<()>;
    /// Read an input event from the input device
    fn event_nonblock(&self) -> AlienResult<Option<u64>>;
    // fn event_block(&self) -> AlienResult<u64>;
    // fn have_event(&self) -> AlienResult<bool>;
}

impl_downcast!(sync InputDomain);
