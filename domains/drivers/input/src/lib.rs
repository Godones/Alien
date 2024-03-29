#![no_std]

extern crate alloc;
use alloc::sync::Arc;
use core::fmt::Debug;

use constants::AlienResult;
use interface::{Basic, DeviceBase, DeviceInfo, InputDomain};
use ksync::Mutex;
use spin::Once;
use virtio_drivers::device::input::VirtIOInput;

mod input;
use input::{HalImpl, VirtioInputWrapper};

static INPUT: Once<Arc<Mutex<VirtioInputWrapper>>> = Once::new();

pub struct InputDevDomain;

impl Debug for InputDevDomain {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Input Device Domain")
    }
}
impl Basic for InputDevDomain {}
impl DeviceBase for InputDevDomain {
    fn handle_irq(&self) -> AlienResult<()> {
        INPUT.get().unwrap().lock().ack_interrupt();
        Ok(())
    }
}

impl InputDomain for InputDevDomain {
    fn init(&self, device_info: &DeviceInfo) -> AlienResult<()> {
        let input = Arc::new(Mutex::new(VirtioInputWrapper::new(device_info)));
        INPUT.call_once(|| input);
        Ok(())
    }
    fn event_nonblock(&self) -> AlienResult<Option<u64>> {
        match INPUT.get().unwrap().lock().pop_pending_event() {
            Some(e) => Ok(Some(
                (e.event_type as u64) << 48 | (e.code as u64) << 32 | (e.value) as u64,
            )),
            None => Ok(None),
        }
    }
}

pub fn main() -> Arc<dyn InputDomain> {
    Arc::new(InputDevDomain)
}
