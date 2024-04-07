#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::boxed::Box;
use core::fmt::Debug;

use basic::{io::SafeIORegion, println};
use constants::AlienResult;
use interface::{Basic, DeviceBase, DeviceInfo, UartDomain};
use spin::Once;
use uart16550::Uart16550;

static UART: Once<Uart16550> = Once::new();

#[derive(Debug)]
struct UartDomainImpl;

impl DeviceBase for UartDomainImpl {
    fn handle_irq(&self) -> AlienResult<()> {
        todo!()
    }
}

impl Basic for UartDomainImpl {}

impl UartDomain for UartDomainImpl {
    fn init(&self, device_info: &DeviceInfo) -> AlienResult<()> {
        let region = &device_info.address_range;
        println!("uart_addr: {:#x}-{:#x}", region.start, region.end);
        let io_region = SafeIORegion::from(region.clone())?;
        let uart = Uart16550::new(Box::new(io_region));
        uart.enable_receive_interrupt()?;
        UART.call_once(|| uart);
        Ok(())
    }

    fn putc(&self, ch: u8) -> AlienResult<()> {
        UART.get().unwrap().putc(ch)
    }

    fn getc(&self) -> AlienResult<Option<u8>> {
        UART.get().unwrap().getc()
    }

    fn have_data_to_get(&self) -> AlienResult<bool> {
        UART.get().unwrap().have_data_to_get()
    }

    fn enable_receive_interrupt(&self) -> AlienResult<()> {
        UART.get().unwrap().enable_receive_interrupt()
    }

    fn disable_receive_interrupt(&self) -> AlienResult<()> {
        UART.get().unwrap().disable_receive_interrupt()
    }
}

pub fn main() -> Box<dyn UartDomain> {
    Box::new(UartDomainImpl)
}
