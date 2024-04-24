#![no_std]

extern crate alloc;

use alloc::{boxed::Box};
use core::{fmt::Debug, ops::Range};

use constants::{AlienError, AlienResult};
use interface::{Basic, DeviceBase, InputDomain};
use ksync::Mutex;
use spin::Once;
use virtio_drivers::device::input::VirtIOInput;
use virtio_drivers::transport::mmio::MmioTransport;
use basic::io::SafeIORegion;
use virtio_mmio_common::{HalImpl, SafeIORW};


static INPUT: Once<Mutex<VirtIOInput<HalImpl,MmioTransport>>> = Once::new();

#[derive(Debug)]
pub struct InputDevDomain;



impl Basic for InputDevDomain {}
impl DeviceBase for InputDevDomain {
    fn handle_irq(&self) -> AlienResult<()> {
        INPUT.get().unwrap().lock().ack_interrupt().unwrap();
        Ok(())
    }
}

impl InputDomain for InputDevDomain {
    fn init(&self, address_range: Range<usize>) -> AlienResult<()> {
        let io_region = SafeIORW(SafeIORegion::from(address_range));
        let transport = MmioTransport::new(Box::new(io_region)).unwrap();
        let input = VirtIOInput::<HalImpl, MmioTransport>::new(transport)
            .expect("failed to create input driver");
        INPUT.call_once(|| Mutex::new(input));
        Ok(())
    }
    fn event_nonblock(&self) -> AlienResult<Option<u64>> {
        match INPUT.get().unwrap().lock().pop_pending_event() {
            Ok(v)=>{
               let val =  v.map(|e|(e.event_type as u64) << 48 | (e.code as u64) << 32 | (e.value) as u64,);
                Ok(val)
            }
            Err(_e) => {
                Err(AlienError::EINVAL)
            }
        }
    }
}

pub fn main() -> Box<dyn InputDomain> {
    Box::new(InputDevDomain)
}
