#![no_std]
extern crate alloc;
extern crate malloc;

use alloc::sync::Arc;
use core::ops::Range;
use interface::{Basic, DeviceBase, DeviceInfo, UartDomain};
use region::SafeIORegion;
use rref::{RRef, RRefVec, RpcResult};

#[derive(Debug)]
pub struct UartDomainImpl {
    region: SafeIORegion,
}

impl UartDomainImpl {
    pub fn new(uart_addr: usize, size: usize) -> Self {
        Self {
            region: SafeIORegion::new(uart_addr, size).unwrap(),
        }
    }
}

impl Basic for UartDomainImpl {}

impl DeviceBase for UartDomainImpl {
    fn handle_irq(&self) -> RpcResult<()> {
        todo!()
    }
}

impl UartDomain for UartDomainImpl {
    fn putc(&self, _ch: u8) -> RpcResult<()> {
        todo!()
    }

    fn getc(&self) -> RpcResult<Option<u8>> {
        todo!()
    }

    fn have_data_to_get(&self) -> bool {
        todo!()
    }
}

pub fn main() -> Arc<dyn UartDomain> {
    let devices_domain = libsyscall::get_devices_domain().unwrap();
    let name = RRefVec::from_slice("uart".as_bytes());

    let info = RRef::new(DeviceInfo {
        address_range: Default::default(),
        irq: RRef::new(0),
        compatible: RRefVec::new(0, 64),
    });

    let info = devices_domain.get_device(name, info).unwrap();
    let region = &info.address_range;
    libsyscall::println!("uart_addr: {:#x}-{:#x}", region.start, region.end);
    Arc::new(UartDomain::new(region.start, region.end - region.start))
}
