#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::boxed::Box;
use core::{fmt::Debug, ops::Range};

use basic::{io::SafeIORegion, println};
use constants::AlienResult;
use interface::{Basic, DeviceBase, UartDomain};
use raw_uart16550::{Uart16550, Uart16550IO};
use spin::Once;

static UART: Once<Uart16550> = Once::new();

#[derive(Debug)]
pub struct SafeIORegionWrapper(SafeIORegion);

impl Uart16550IO for SafeIORegionWrapper {
    fn read_at(&self, offset: usize) -> AlienResult<u8> {
        self.0.read_at(offset)
    }

    fn write_at(&self, offset: usize, value: u8) -> AlienResult<()> {
        self.0.write_at(offset, value)
    }
}

#[derive(Debug)]
struct UartDomainImpl;

impl DeviceBase for UartDomainImpl {
    fn handle_irq(&self) -> AlienResult<()> {
        todo!()
    }
}

impl Basic for UartDomainImpl {}

impl UartDomain for UartDomainImpl {
    fn init(&self, address_range: Range<usize>) -> AlienResult<()> {
        let region = &address_range;
        println!("uart_addr: {:#x}-{:#x}", region.start, region.end);
        let io_region = SafeIORegion::from(region.clone());
        let uart = Uart16550::new(Box::new(SafeIORegionWrapper(io_region)));
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
